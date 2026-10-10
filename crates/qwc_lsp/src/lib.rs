/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use qwc_arena::Files;
use qwc_ast as ast;
use qwc_hir as hir;
use qwc_diagnostic::{Label, Message, Summary, msg::*};
use qwc_hir_gen::HGen;
use qwc_parse::Parse;
use qwc_resolve::{ExportMap, Imod, ImplFor, ScopeCollector, ScopeMap};
use qwc_string_interner::StrInterner;
use qwc_vfs::Vfs;



#[derive(Clone)]
pub struct CoreUnit {
  pub base_interner: StrInterner,
  pub core_exp: ExportMap,
  pub core_name_sid: qwc_string_interner::Sid,
  pub core_cid: hir::CID,
  pub core_cre: hir::Krate,
  pub core_prims: hir::PrimTypes,
}

impl CoreUnit {
  pub fn new() -> Self {
    let mut sin = StrInterner::new();
    let hir_layinfo = hir::LayoutInfo {
      str_lay:  hir::Layout::new_dst(hir::LayoutBy::SYS),
      bool_lay: hir::Layout::new_static(hir::LayoutBy::SYS),
      i8_lay:   hir::Layout::new_static(hir::LayoutBy::SYS),
      i16_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
      i32_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
      i64_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
      i128_lay: hir::Layout::new_static(hir::LayoutBy::SYS),
      bf16_lay: hir::Layout::new_static(hir::LayoutBy::SYS),
      f16_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
      f32_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
      f64_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
      f128_lay: hir::Layout::new_static(hir::LayoutBy::SYS),
      ptr_size: hir::Layout::new_static(hir::LayoutBy::SYS),
    };
    let mut deps = hir::Deps::new();
    let core_cid = deps.get_next_id();
    let (core_cre, core_exp, core_prims) = qwc_intrinsic::new_core(core_cid, &mut sin, &hir_layinfo);
    let core_name_sid = sin.sid("core");

    Self { base_interner: sin, core_exp, core_name_sid, core_cid, core_cre, core_prims }
  }
}

pub struct AnalysisState {
  pub scope_map: ScopeMap,
  pub impl_for: Vec<ImplFor>,
  pub krate: ast::Krate,
  pub interner: StrInterner,
  pub files: Files,
  pub target_fid: u16,
  pub hir_cid: Option<hir::CID>,
  pub hir_deps: hir::Deps,
}

#[derive(Clone)]
pub struct Backend {
  pub client: Client,
  pub vfs: Arc<RwLock<Vfs>>,
  pub analysis: Arc<RwLock<Option<AnalysisState>>>,
  pub core: Arc<CoreUnit>,
}

impl Backend {
  fn load_module_file(fpath: &Path, cre: &mut ast::Krate, sin: &mut StrInterner, far: &mut Files, vfs: &Vfs) -> (ast::Item, Summary) {
    let fid = match vfs.get_by_path(fpath) {
      Some(vfile) => far.add_buffer(fpath, vfile.content().to_vec()),
      None => {
        if fpath.exists() { far.add(fpath) }
        else {
          return (
            ast::Item {
              pos: qwc_diagnostic::Span::new(0, std::num::NonZeroU16::MIN, 0),
              vis: ast::Visibility::Inherited,
              name: None,
              kind: ast::ItemKind::Krate(ast::Rng::empty()),
            },
            Summary::new(),
          );
        }
      }
    };

    let fi = far.get(fid);
    let (start, rng, mut sum) = Parse::parse(cre, sin, far, fi);

    let mut submods_to_load = Vec::new();
    for id in cre.extra_get(rng) {
      let it = cre.get(id);
      
      if let ast::ItemKind::ModuleUnloaded = it.kind {
        let name = sin.str(it.name.unwrap().sid()).to_string();
        let parent_dir = fpath.parent().unwrap_or(Path::new("."));
        let path1 = parent_dir.join(name.clone() + ".qw");
        let path2 = parent_dir.join(&name).join("mod.qw");

        let path = if vfs.exists(&path1) { path1 }
        else if vfs.exists(&path2) { path2 }
        else {
          sum.add(Message::error(
            COULD_NOT_FIND_MODULE_FILE,
            Label::new(
              it.name.unwrap(),
              NOT_FOUND_IN_OR2.args(&[
                path1.to_str().unwrap_or(""),
                path2.to_str().unwrap_or(""),
              ]),
            ),
          ));
          continue;
        };

        submods_to_load.push((id, path));
      }
    }

    for (id, sub_path) in submods_to_load {
      let (it, ssum) = Self::load_module_file(&sub_path, cre, sin, far, vfs);
      let sub_rng = if let ast::ItemKind::Krate(rng) = it.kind {
        rng
      } else {
        ast::Rng::empty()
      };
      let this: &mut ast::Item = cre.get_mut(id);
      this.kind = ast::ItemKind::ModuleFile(sub_rng, it.pos.fid());
      sum += ssum;
    }

    let this = ast::Item {
      pos: start,
      vis: ast::Visibility::Inherited,
      name: None,
      kind: ast::ItemKind::Krate(rng),
    };

    (this, sum)
  }

  async fn validate_document(&self, uri: &Url) {
    let Ok(path) = uri.to_file_path() else { return };

    let vfs = self.vfs.read().await;
    if !vfs.exists(&path) { return }

    let mut far = Files::new();
    let mut cre = ast::Krate::new();
    let mut sin = self.core.base_interner.clone();

    // Parse AST and load submodules using VFS
    let (root_item, mut summary) = Self::load_module_file(&path, &mut cre, &mut sin, &mut far, &vfs);
    let target_fid = root_item.pos.fid();

    let root_id = cre.push(root_item);
    cre.set_root(root_id);

    // Run ScopeCollector with pre-created core dependency
    let imods = [Imod::new(self.core.core_name_sid, &self.core.core_exp)];
    let scope_res = (summary.sumerr() == 0).then(|| ScopeCollector::collect(&cre, &sin, &imods));

    if let Some(Err(scope_sum)) = scope_res.as_ref() {
      summary += scope_sum.clone();
    }

    // Run HGen (HIR lowering & type checking) if scope collection succeeded
    let mut hir_state = None;
    if let Some(Ok((scope_map, impl_for))) = scope_res {
      if summary.sumerr() == 0 {
        let mut deps = hir::Deps::new();
        deps.set_prims(self.core.core_prims);
        deps.add(self.core.core_cre.clone());
        let imod_cids = [self.core.core_cid];

        let (hir_cid, hir_sum) = HGen::low(&cre, &sin, &far, &scope_map, impl_for.clone(), &imod_cids, &mut deps);

        summary += hir_sum;
        hir_state = Some((scope_map, impl_for, hir_cid, deps));
      } else {
        hir_state = Some((scope_map, impl_for, None, hir::Deps::new()));
      }
    }

    // Publish diagnostics (Syntax + Scope + Type checking)
    let diagnostics = summary.to_lsp(&far, target_fid);
    self.client
      .publish_diagnostics(uri.clone(), diagnostics, None)
      .await;

    // Save state for Go to Definition & Hover
    if let Some((scope_map, impl_for, hir_cid, deps)) = hir_state {
      let mut analysis = self.analysis.write().await;
      *analysis = Some(AnalysisState { scope_map, impl_for, krate: cre, interner: sin, files: far, target_fid, hir_cid, hir_deps: deps });
    }
  }

  async fn build_completions(&self) -> Vec<CompletionItem> {
    use std::collections::HashSet;
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    // Snippets and declarations
    let snippets = [
      ("fun", "fun ${1:name}(${2:params}) {\n\t$0\n}", "Function definition"),
      ("let", "let ${1:name} = ${2:value};", "Immutable variable declaration"),
      ("var", "var ${1:name} = ${2:value};", "Mutable variable declaration"),
      ("struct", "struct ${1:Name} {\n\t$0\n}", "Struct definition"),
      ("enum", "enum ${1:Name} {\n\t$0\n}", "Enum definition"),
      ("trait", "trait ${1:Name} {\n\t$0\n}", "Trait definition"),
      ("iface", "iface ${1:Name} {\n\t$0\n}", "Interface definition"),
      ("impl", "impl ${1:Type} {\n\t$0\n}", "Implementation block"),
      ("if", "if ${1:condition} {\n\t$0\n}", "If condition"),
      ("ef", "ef ${1:condition} {\n\t$0\n}", "Else-if condition"),
      ("else", "else {\n\t$0\n}", "Else block"),
      ("while", "while ${1:condition} {\n\t$0\n}", "While loop"),
      ("for", "for ${1:item} in ${2:iter} {\n\t$0\n}", "For-in loop"),
      ("loop", "loop {\n\t$0\n}", "Infinite loop"),
      ("match", "match ${1:expr} {\n\t$0\n}", "Pattern matching"),
      ("mod", "mod ${1:name};", "Module declaration"),
      ("use", "use ${1:path};", "Import declaration"),
      ("return", "return ${1:expr};", "Return statement"),
      ("ret", "ret ${1:expr};", "Return statement"),
    ];

    for (label, snippet, doc) in snippets {
      seen.insert(label.to_string());
      items.push(CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(doc.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("0_{}", label)),
        ..Default::default()
      });
    }

    // Control flow, visibility and modifiers
    let keywords = [
      "break", "continue", "pub", "priv", "prot", "mut", "imm",
      "const", "static", "task", "init", "fini", "using", "requires",
      "generic", "flags", "die", "crate", "super", "self", "Self", "true", "false", "null", "nil",
    ];

    for kw in keywords {
      if seen.insert(kw.to_string()) {
        items.push(CompletionItem {
          label: kw.to_string(),
          kind: Some(CompletionItemKind::KEYWORD),
          detail: Some("keyword".to_string()),
          sort_text: Some(format!("1_{}", kw)),
          ..Default::default()
        });
      }
    }

    // Primitive and standard types
    let types = [
      "i8", "i16", "i32", "i64", "i128", "isize",
      "u8", "u16", "u32", "u64", "u128", "usize",
      "f16", "f32", "f64", "f128", "fsize", "bf16",
      "bool", "str", "char", "void", "never", "type",
    ];

    for ty in types {
      if seen.insert(ty.to_string()) {
        items.push(CompletionItem {
          label: ty.to_string(),
          kind: Some(CompletionItemKind::TYPE_PARAMETER),
          detail: Some("primitive type".to_string()),
          sort_text: Some(format!("2_{}", ty)),
          ..Default::default()
        });
      }
    }

    // User symbols from current AnalysisState
    if let Some(analysis) = self.analysis.read().await.as_ref() {
      for (_id, scope) in analysis.scope_map.iter() {
        for (sid, (kind, _span)) in scope.iter() {
          let name = analysis.interner.str(*sid).to_string();
          if seen.insert(name.clone()) {
            let item_kind = match kind {
              qwc_resolve::ScopeKind::Ast(qwc_resolve::ScopeKindAst::Type(_) | qwc_resolve::ScopeKindAst::GenericType(..)) => CompletionItemKind::STRUCT,
              qwc_resolve::ScopeKind::Ast(qwc_resolve::ScopeKindAst::Expr(_) | qwc_resolve::ScopeKindAst::GenericExpr(..)) => CompletionItemKind::FUNCTION,
              qwc_resolve::ScopeKind::Ast(qwc_resolve::ScopeKindAst::Module(_)) => CompletionItemKind::MODULE,
              qwc_resolve::ScopeKind::Ast(qwc_resolve::ScopeKindAst::Local(_)) => CompletionItemKind::VARIABLE,
              qwc_resolve::ScopeKind::Ast(qwc_resolve::ScopeKindAst::TypeParam(..)) => CompletionItemKind::TYPE_PARAMETER,
              qwc_resolve::ScopeKind::Ast(qwc_resolve::ScopeKindAst::ExprParam(..)) => CompletionItemKind::VARIABLE,
              qwc_resolve::ScopeKind::Hir(qwc_resolve::ScopeKindHir::Type(_)) => CompletionItemKind::STRUCT,
              qwc_resolve::ScopeKind::Hir(qwc_resolve::ScopeKindHir::Expr(_, _)) => CompletionItemKind::FUNCTION,
              qwc_resolve::ScopeKind::Hir(qwc_resolve::ScopeKindHir::Module(_)) => CompletionItemKind::MODULE,
            };
            items.push(CompletionItem {
              label: name,
              kind: Some(item_kind),
              detail: Some("symbol".to_string()),
              sort_text: Some("0_symbol".to_string()),
              ..Default::default()
            });
          }
        }
      }
    }

    // Core library exports
    for (_id, export) in self.core.core_exp.iter() {
      for (sid, kind) in export.iter() {
        let name = self.core.base_interner.str(*sid).to_string();
        if seen.insert(name.clone()) {
          let item_kind = match kind {
            qwc_resolve::ExportKind::Type(_) => CompletionItemKind::STRUCT,
            qwc_resolve::ExportKind::Expr(_, _) => CompletionItemKind::FUNCTION,
            qwc_resolve::ExportKind::NameSpace(_) => CompletionItemKind::MODULE,
          };
          items.push(CompletionItem {
            label: name.clone(),
            kind: Some(item_kind),
            detail: Some("core".to_string()),
            sort_text: Some(format!("3_{}", name)),
            ..Default::default()
          });
        }
      }
    }

    items
  }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
          TextDocumentSyncKind::FULL,
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
          resolve_provider: Some(false),
          trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    })
  }

  async fn initialized(&self, _: InitializedParams) {
    self.client.log_message(MessageType::INFO, "QW Language Server initialized!").await;
  }

  async fn shutdown(&self) -> Result<()> {
    Ok(())
  }

  async fn did_open(&self, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri;
    let text = params.text_document.text;
    let version = params.text_document.version;

    if let Ok(path) = uri.to_file_path() {
      {
        let mut vfs = self.vfs.write().await;
        vfs.set_overlay(&path, text.into_bytes(), version);
      }
      self.validate_document(&uri).await;
    }
  }

  async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    let version = params.text_document.version;

    if let Some(change) = params.content_changes.into_iter().last() {
      if let Ok(path) = uri.to_file_path() {
        {
          let mut vfs = self.vfs.write().await;
          vfs.set_overlay(&path, change.text.into_bytes(), version);
        }
        self.validate_document(&uri).await;
      }
    }
  }

  async fn did_close(&self, params: DidCloseTextDocumentParams) {
    let uri = params.text_document.uri;
    if let Ok(path) = uri.to_file_path() {
      let mut vfs = self.vfs.write().await;
      vfs.remove_overlay(&path);
    }
  }

  async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let items = self.build_completions().await;
    Ok(Some(CompletionResponse::Array(items)))
  }

  async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
    Ok(None)
  }

  async fn goto_definition(&self, _params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    Ok(None)
  }

  async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
    let mut actions = Vec::new();

    for diag in params.context.diagnostics {
      if let Some(ref data) = diag.data {
        if let Ok(fixes) = serde_json::from_value::<Vec<serde_json::Value>>(data.clone()) {
          for fix in fixes {
            let title = fix.get("title").and_then(|v| v.as_str()).unwrap_or("Apply suggestion");
            let replacement = fix.get("replacement").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(range_val) = fix.get("range") {
              if let Ok(range) = serde_json::from_value::<Range>(range_val.clone()) {
                let action_title = if !replacement.is_empty() {
                  format!("{}: `{}`", title, replacement)
                } else {
                  title.to_string()
                };

                let mut changes = std::collections::HashMap::new();
                changes.insert(
                  params.text_document.uri.clone(),
                  vec![TextEdit {
                    range,
                    new_text: replacement.to_string(),
                  }],
                );

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                  title: action_title,
                  kind: Some(CodeActionKind::QUICKFIX),
                  diagnostics: Some(vec![diag.clone()]),
                  edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                  }),
                  is_preferred: Some(true),
                  ..Default::default()
                }));
              }
            }
          }
        }
      }
    }

    if actions.is_empty() {
      Ok(None)
    } else {
      Ok(Some(actions))
    }
  }
}

pub fn start() -> std::result::Result<(), String> {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .map_err(|_| String::from("runtime cannot started"))?
    .block_on(async {
      let stdin = tokio::io::stdin();
      let stdout = tokio::io::stdout();

      let core = Arc::new(CoreUnit::new());
      let vfs = Arc::new(RwLock::new(Vfs::new()));
      let analysis = Arc::new(RwLock::new(None));
      let (service, socket) = LspService::new(|client| Backend { client, vfs, analysis, core });
      
      Server::new(stdin, stdout, socket).serve(service).await;
    }
  );

  Ok(())
}
