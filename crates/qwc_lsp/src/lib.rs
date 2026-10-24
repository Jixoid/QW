/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::collections::HashSet;
use std::path::{Path, PathBuf};
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
  pub core_prim_op: hir::PrimOps,
}

impl CoreUnit {
  pub fn new() -> Self {
    let hir_layinfo = hir::LayoutInfo {
      str_lay:  hir::Layout::DST,
      bool_lay: hir::Layout::STATIC,
      i8_lay:   hir::Layout::STATIC,
      i16_lay:  hir::Layout::STATIC,
      i32_lay:  hir::Layout::STATIC,
      i64_lay:  hir::Layout::STATIC,
      i128_lay: hir::Layout::STATIC,
      bf16_lay: hir::Layout::STATIC,
      f16_lay:  hir::Layout::STATIC,
      f32_lay:  hir::Layout::STATIC,
      f64_lay:  hir::Layout::STATIC,
      f128_lay: hir::Layout::STATIC,
      ptr_size: hir::Layout::STATIC,
    };

    let mut sin = StrInterner::default();
    let mut deps = hir::Deps::default();
    let core_cid = deps.get_next_id();
    let (core_cre, core_exp, core_prims, core_prim_op) = qwc_intrinsic::new_core(core_cid, &mut sin, &hir_layinfo);
    //deps.add(core_cre);
    deps.set_prims(core_prims);
    deps.set_prim_op(core_prim_op);
    let core_name_sid = sin.sid("core");

    Self { base_interner: sin, core_exp, core_name_sid, core_cid, core_cre, core_prims, core_prim_op }
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
  pub workspace_root: Arc<RwLock<Option<PathBuf>>>,
  pub published_files: Arc<RwLock<HashSet<Url>>>,
}

fn find_project_root(start_path: &Path, ws_root: Option<&Path>) -> PathBuf {
  let abs_start = start_path.canonicalize().unwrap_or_else(|_| start_path.to_path_buf());

  if let Some(ws) = ws_root {
    let abs_ws = ws.canonicalize().unwrap_or_else(|_| ws.to_path_buf());
    if abs_start.starts_with(&abs_ws) && (abs_ws.join("qw.conf").exists() || abs_ws.join("src/main.qw").exists() || abs_ws.join("src/lib.qw").exists()) {
      return abs_ws;
    }
  }

  let mut cur = if abs_start.is_file() {
    abs_start.parent().unwrap_or(Path::new(".")).to_path_buf()
  } else {
    abs_start.clone()
  };

  let mut candidate = None;
  loop {
    if cur.join("qw.conf").exists() || cur.join("src/main.qw").exists() || cur.join("src/lib.qw").exists() {
      candidate = Some(cur.clone());
    }
    match cur.parent() {
      Some(parent) if parent != cur => cur = parent.to_path_buf(),
      _ => break,
    }
  }

  if let Some(c) = candidate {
    c
  } else if let Some(ws) = ws_root {
    ws.canonicalize().unwrap_or_else(|_| ws.to_path_buf())
  } else {
    abs_start
  }
}

fn path_to_uri(path: &Path) -> Option<Url> {
  if path.is_absolute() {
    Url::from_file_path(path).ok()
  } else if let Ok(abs) = path.canonicalize() {
    Url::from_file_path(abs).ok()
  } else if let Ok(cwd) = std::env::current_dir() {
    Url::from_file_path(cwd.join(path)).ok()
  } else {
    None
  }
}

impl Backend {
  fn load_module_file(fpath: &Path, base_path: ast::DefPathId, cre: &mut ast::Krate, sin: &mut StrInterner, far: &mut Files, vfs: &Vfs) -> (ast::Item, Summary) {
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
              path: base_path,
            },
            Summary::new(),
          );
        }
      }
    };

    let fi = far.get(fid);
    let (start, rng, mut sum) = Parse::parse(cre, base_path, sin, far, fi);

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
      let mod_path = cre.get(id).path;
      let (it, ssum) = Self::load_module_file(&sub_path, mod_path, cre, sin, far, vfs);
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
      path: base_path,
    };

    (this, sum)
  }

  async fn analyze_workspace(&self, trigger_uri: Option<&Url>) {
    let ws_root = self.workspace_root.read().await.clone();
    let target_path = trigger_uri.and_then(|u| u.to_file_path().ok());

    let project_root = target_path
      .as_deref()
      .map(|p| find_project_root(p, ws_root.as_deref()))
      .or_else(|| ws_root.clone())
      .unwrap_or_else(|| PathBuf::from("."));

    let main_entry = if project_root.is_file() {
      project_root.clone()
    } else if project_root.join("src/main.qw").exists() {
      project_root.join("src/main.qw")
    } else if project_root.join("src/lib.qw").exists() {
      project_root.join("src/lib.qw")
    } else {
      project_root.clone()
    };

    let vfs = self.vfs.read().await;

    let mut far = Files::default();
    let mut sin = self.core.base_interner.clone();
    let mut summary = Summary::new();

    let mut deps = hir::Deps::default();
    deps.set_prims(self.core.core_prims);
    deps.set_prim_op(self.core.core_prim_op);
    let core_cid = deps.add(self.core.core_cre.clone());

    let mut dep_exps = Vec::new();
    let mut imod_cids_for_main = vec![core_cid];

    // Check qw.conf for dependencies
    if project_root.join("qw.conf").exists() {
      let mut conf_far = Files::default();
      if let Ok(conf) = qwc_config::read_conf(&project_root, &mut conf_far) {
        if !conf.deps.is_empty() {
          let mut dep_sum = Summary::new();
          if let qwc_diagnostic::Status::Success(dep_list) = qwc_deps::find_deps(&mut dep_sum, &project_root, &conf.deps) {
            for (dep_name, dep_dir) in dep_list {
              let lib_path = dep_dir.join("src/lib.qw");
              let main_path = dep_dir.join("src/main.qw");
              let dep_entry = if vfs.exists(&lib_path) {
                lib_path
              } else if vfs.exists(&main_path) {
                main_path
              } else {
                dep_dir.clone()
              };

              if vfs.exists(&dep_entry) {
                let mut dep_cre = ast::Krate::default();
                let dep_root_path = dep_cre.push(ast::DefPath::Root(sin.sid(&dep_name)));
                let (dep_root_item, dep_ssum) = Self::load_module_file(&dep_entry, dep_root_path, &mut dep_cre, &mut sin, &mut far, &vfs);
                let dep_clean = dep_ssum.sumerr() == 0;
                summary += dep_ssum;
                let dep_root_id = dep_cre.push(dep_root_item);
                dep_cre.set_root(dep_root_id);

                if dep_clean {
                  let dep_imods = [Imod::new(self.core.core_name_sid, &self.core.core_exp)];
                  if let Ok((dep_scope_map, dep_impl_for)) = ScopeCollector::collect(&dep_cre, &mut sin, &dep_imods) {
                    let dep_imod_cids = [core_cid];
                    let (dep_hir_cid, dep_hir_sum) = HGen::low(&dep_cre, &sin, &far, &dep_scope_map, dep_impl_for, &dep_imod_cids, &mut deps);
                    summary += dep_hir_sum;
                    if let Some(dep_cid) = dep_hir_cid {
                      let dep_hir_cre = deps.get(dep_cid);
                      if let Ok(dep_exp) = ExportMap::visit(dep_hir_cre, &deps, &sin) {
                        let dep_sid = sin.sid(&dep_name);
                        dep_exps.push((dep_sid, dep_exp, dep_cid));
                      }
                    }
                  }
                }
              }
            }
          }
          summary += dep_sum;
        }
      }
    }

    let mut imods_for_main = Vec::with_capacity(1 + dep_exps.len());
    imods_for_main.push(Imod::new(self.core.core_name_sid, &self.core.core_exp));
    for (dep_sid, dep_exp, dep_cid) in &dep_exps {
      imods_for_main.push(Imod::new(*dep_sid, dep_exp));
      imod_cids_for_main.push(*dep_cid);
    }

    // Parse AST for Main Crate
    let mut main_cre = ast::Krate::default();
    let main_krate_name = qwc_config::read_conf(&project_root, &mut Files::default())
      .ok()
      .map(|c| c.name)
      .unwrap_or_else(|| {
        project_root.file_name().and_then(|s| s.to_str()).unwrap_or("main").to_string()
      });
    let main_root_path = main_cre.push(ast::DefPath::Root(sin.sid(&main_krate_name)));
    let (main_root_item, main_ssum) = Self::load_module_file(&main_entry, main_root_path, &mut main_cre, &mut sin, &mut far, &vfs);
    summary += main_ssum;
    let main_root_id = main_cre.push(main_root_item);
    main_cre.set_root(main_root_id);

    // Scope for Main Crate
    let scope_res = (summary.sumerr() == 0).then(|| ScopeCollector::collect(&main_cre, &mut sin, &imods_for_main));
    if let Some(Err(scope_sum)) = scope_res.as_ref() {
      summary += scope_sum.clone();
    }

    let mut hir_state = None;
    if let Some(Ok((scope_map, impl_for))) = scope_res {
      if summary.sumerr() == 0 {
        let (hir_cid, hir_sum) = HGen::low(&main_cre, &sin, &far, &scope_map, impl_for.clone(), &imod_cids_for_main, &mut deps);
        summary += hir_sum;
        hir_state = Some((scope_map, impl_for, hir_cid, deps));
      } else {
        hir_state = Some((scope_map, impl_for, None, deps));
      }
    }

    // Publish diagnostics to ALL files in far
    let mut current_diag_files = HashSet::new();

    for fid in 0..far.len() as u16 {
      let file_ref = far.get(fid);
      let fpath = Path::new(file_ref.fpath());
      let Some(uri) = path_to_uri(fpath) else { continue };

      let diags = summary.to_lsp(&far, fid);
      current_diag_files.insert(uri.clone());
      self.client.publish_diagnostics(uri, diags, None).await;
    }

    // Clear diagnostics for files that had errors before but now don't
    let mut published = self.published_files.write().await;
    for old_uri in published.difference(&current_diag_files) {
      self.client.publish_diagnostics(old_uri.clone(), vec![], None).await;
    }
    *published = current_diag_files;

    // Save state for Go to Definition & Hover
    if let Some((scope_map, impl_for, hir_cid, deps)) = hir_state {
      let mut analysis = self.analysis.write().await;
      *analysis = Some(AnalysisState {
        scope_map,
        impl_for,
        krate: main_cre,
        interner: sin,
        files: far,
        target_fid: 0,
        hir_cid,
        hir_deps: deps,
      });
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
              qwc_resolve::ScopeKind::Ast(qwc_resolve::ScopeKindAst::TypeParam(..) | qwc_resolve::ScopeKindAst::AssocType(..)) => CompletionItemKind::TYPE_PARAMETER,
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
      for (name, kind) in export.iter() {
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
  async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
    if let Some(folders) = params.workspace_folders {
      if let Some(first) = folders.first() {
        if let Ok(path) = first.uri.to_file_path() {
          *self.workspace_root.write().await = Some(path);
        }
      }
    } else if let Some(uri) = params.root_uri {
      if let Ok(path) = uri.to_file_path() {
        *self.workspace_root.write().await = Some(path);
      }
    }

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
    self.analyze_workspace(None).await;
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
      self.analyze_workspace(Some(&uri)).await;
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
        self.analyze_workspace(Some(&uri)).await;
      }
    }
  }

  async fn did_close(&self, params: DidCloseTextDocumentParams) {
    let uri = params.text_document.uri;
    if let Ok(path) = uri.to_file_path() {
      let mut vfs = self.vfs.write().await;
      vfs.remove_overlay(&path);
    }
    self.analyze_workspace(Some(&uri)).await;
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
      let workspace_root = Arc::new(RwLock::new(None));
      let published_files = Arc::new(RwLock::new(HashSet::new()));
      let (service, socket) = LspService::new(|client| Backend { client, vfs, analysis, core, workspace_root, published_files });
      
      Server::new(stdin, stdout, socket).serve(service).await;
    }
  );

  Ok(())
}
