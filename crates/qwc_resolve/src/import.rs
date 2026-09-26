use qwc_ast::{AnyId, Ident, Krate};
use qwc_diagnostic::{Applicability, CodedMsg, Label, Message, Span, Suggestion, Summary, msg::*};
use qwc_string_interner::{Sid, StrInterner};
use strsim::damerau_levenshtein;

use crate::{ExportKind, Imod, ImportDef, ImportSegment, ImportStatus, InsertResult, ScopeKind, ScopeKindAst, ScopeKindHir, ScopeMap};

const EMPTY_IMPORT_PATH:        CodedMsg = CodedMsg::new_str("empty import path");
const CANNOT_USE_SUPER_AT_ROOT: CodedMsg = CodedMsg::new_str("cannot use `super` at the root module");
const EXPECTED_MODULE_FOR_GLOB: CodedMsg = CodedMsg::new_str("expected a module for glob import");
const EXPECTED_MODULE_IN_PATH:  CodedMsg = CodedMsg::new_str("expected module in import path");
const MODULE_SCOPE_NOT_FOUND:   CodedMsg = CodedMsg::new_str("module scope not found");



pub fn resolve_imports(scp: &mut ScopeMap, cre: &Krate, sin: &StrInterner, imods: &[Imod<'_>], sum: &mut Summary) {
  let root_id = match cre.root() {
    Some(id) => id.to_any(),
    None => return,
  };

  ImportResolver {
    sin,
    root_id,
    imods,
  }
  .resolve(scp, sum);
}


struct ImportResolver<'a, 'imod> {
  sin: &'a StrInterner,
  root_id: AnyId,
  imods: &'a [Imod<'imod>],
}

enum ResolveResult {
  Specific { name: Sid, kind: ScopeKind, span: Span },
  Glob { items: Vec<(Sid, ScopeKind, Span)>, is_fully_solved: bool },
  Pending,
  Failed,
}

enum ImportAction {
  BindSpecific { scope_id: AnyId, import_idx: usize, name: Sid, kind: ScopeKind, span: Span },
  BindGlob { scope_id: AnyId, import_idx: usize, items: Vec<(Sid, ScopeKind, Span)>, is_fully_solved: bool },
}


fn find_did_you_mean<'a, S: Copy>(input: &str, candidates: &[(&'a str, S)]) -> Option<(&'a str, S)> {
  let max_distance = (input.chars().count() / 3).max(1);

  candidates
    .iter()
    .map(|&(candidate, span)| ((candidate, span), damerau_levenshtein(input, candidate)))
    .filter(|&(_, dist)| dist <= max_distance)
    .min_by_key(|&(_, dist)| dist)
    .map(|(matched, _)| matched)
}

impl<'a, 'imod> ImportResolver<'a, 'imod> {

  fn resolve(&self, scp: &mut ScopeMap, sum: &mut Summary) {
    let scope_ids: Vec<AnyId> = scp.map.keys().copied().collect();
    let mut progress = true;
    let mut pass = 0;
    const MAX_PASSES: usize = 256;

    while progress && pass < MAX_PASSES {
      progress = false;
      pass += 1;

      let mut actions = Vec::new();

      for &scope_id in &scope_ids {
        let scope = match scp.get(&scope_id) {
          Some(s) => s,
          None => continue,
        };

        for (idx, import) in scope.import.iter().enumerate() {
          if import.is_solved() {
            continue;
          }

          match self.try_resolve_import(scp, scope_id, import) {
            ResolveResult::Specific { name, kind, span } => {
              actions.push(ImportAction::BindSpecific {
                scope_id,
                import_idx: idx,
                name,
                kind,
                span,
              });
            }
            ResolveResult::Glob { items, is_fully_solved } => {
              if !items.is_empty() || is_fully_solved {
                actions.push(ImportAction::BindGlob {
                  scope_id,
                  import_idx: idx,
                  items,
                  is_fully_solved,
                });
              }
            }
            ResolveResult::Pending => {}
            ResolveResult::Failed => {}
          }
        }
      }

      for action in actions {
        match action {
          ImportAction::BindSpecific { scope_id, import_idx, name, kind, span } => {
            let scope = scp.get_mut(&scope_id).unwrap();
            match scope.insert_import(name, kind, span) {
              InsertResult::Inserted => {
                progress = true;
              }
              InsertResult::AlreadyPresentSame => {}
              InsertResult::Conflict(first_span) => {
                let import_span = scope.import[import_idx].span;
                sum.add(
                  Message::error(DUPLICATE_IDENTIFIER, Label::new_pos(import_span))
                    .add(Label::new(first_span, FIRST_DEFINITION_HERE)),
                );
              }
            }
            scope.import[import_idx].status = ImportStatus::Solved;
          }
          ImportAction::BindGlob { scope_id, import_idx, items, is_fully_solved } => {
            let scope = scp.get_mut(&scope_id).unwrap();
            for (name, kind, span) in items {
              if !scope.contains_key(&name) {
                scope.insert_import(name, kind, span);
                progress = true;
              }
            }
            if is_fully_solved {
              scope.import[import_idx].status = ImportStatus::Solved;
            }
          }
        }
      }
    }

    // Error reporting for any remaining unsolved imports
    for &scope_id in &scope_ids {
      let scope = scp.get(&scope_id).unwrap();
      for (idx, import) in scope.import.iter().enumerate() {
        if !import.is_solved() {
          self.report_unresolved_import(scp, scope_id, idx, sum);
        }
      }
    }
  }

  fn find_in_scope_or_ancestors(&self, scp: &ScopeMap, start_scope_id: AnyId, sid: Sid) -> Option<(ScopeKind, Span)> {
    let mut cur = Some(start_scope_id);
    while let Some(scope_id) = cur {
      if let Some(scope) = scp.get(&scope_id) {
        if let Some(&target) = scope.get(&sid) {
          return Some(target);
        }
        cur = scope.parent;
      } else {
        break;
      }
    }
    None
  }

  fn resolve_imod_import(&self, imod: &Imod<'_>, mut current_item: qwc_hir::ItemId, segments: &[ImportSegment], glob: bool, span: Span) -> ResolveResult {
    for (idx, seg) in segments.iter().enumerate() {
      let seg_ident = match seg {
        ImportSegment::Name(ident) => *ident,
        _ => return ResolveResult::Failed,
      };

      let is_last_segment = idx == segments.len() - 1;

      let exports = match imod.expmap.get(&current_item.to_any()) {
        Some(e) => e,
        None => return ResolveResult::Failed,
      };

      let export_kind = match exports.get(&seg_ident.sid()) {
        Some(k) => *k,
        None => return ResolveResult::Failed,
      };

      if is_last_segment && !glob {
        let hir_kind = match export_kind {
          ExportKind::Type(ty) => ScopeKindHir::Type(ty),
          ExportKind::Expr(item, ty) => ScopeKindHir::Expr(item, ty),
          ExportKind::NameSpace(item) => ScopeKindHir::Module(item),
        };
        return ResolveResult::Specific {
          name: seg_ident.sid(),
          kind: ScopeKind::Hir(hir_kind),
          span,
        };
      } else {
        match export_kind {
          ExportKind::NameSpace(sub_item) => {
            current_item = sub_item;
          }
          _ => return ResolveResult::Failed,
        }
      }
    }

    if glob {
      let exports = match imod.expmap.get(&current_item.to_any()) {
        Some(e) => e,
        None => return ResolveResult::Failed,
      };

      let mut items = Vec::new();
      for (&name_sid, &export_kind) in exports.iter() {
        let hir_kind = match export_kind {
          ExportKind::Type(ty) => ScopeKindHir::Type(ty),
          ExportKind::Expr(item, ty) => ScopeKindHir::Expr(item, ty),
          ExportKind::NameSpace(item) => ScopeKindHir::Module(item),
        };
        items.push((name_sid, ScopeKind::Hir(hir_kind), span));
      }

      ResolveResult::Glob {
        items,
        is_fully_solved: true,
      }
    } else {
      ResolveResult::Specific {
        name: imod.name,
        kind: ScopeKind::Hir(ScopeKindHir::Module(current_item)),
        span,
      }
    }
  }

  fn try_resolve_import(&self, scp: &ScopeMap, importing_scope_id: AnyId, import: &ImportDef) -> ResolveResult {
    if import.segments.is_empty() {
      return ResolveResult::Failed;
    }

    let first_seg = import.segments[0];
    let mut next_seg_idx = 1;

    let cur_scope_id = match first_seg {
      ImportSegment::Crate => self.root_id,
      ImportSegment::Super => {
        let mut cur = match scp.get(&importing_scope_id).and_then(|s| s.parent) {
          Some(p) => p,
          None => return ResolveResult::Failed,
        };

        while next_seg_idx < import.segments.len() {
          if let ImportSegment::Super = import.segments[next_seg_idx] {
            match scp.get(&cur).and_then(|s| s.parent) {
              Some(p) => cur = p,
              None => return ResolveResult::Failed,
            }
            next_seg_idx += 1;
          } else {
            break;
          }
        }
        cur
      }
      ImportSegment::Name(first_ident) => {
        let first_target = self.find_in_scope_or_ancestors(scp, importing_scope_id, first_ident.sid());
        let (kind, span) = match first_target {
          Some(t) => t,
          None => {
            // Check if first_ident is an imod!
            if let Some(imod) = self.imods.iter().find(|im| im.name == first_ident.sid()) {
              let root_id = match imod.expmap.root() {
                Some(r) => r,
                None => return ResolveResult::Failed,
              };
              return self.resolve_imod_import(imod, root_id, &import.segments[1..], import.glob, import.span);
            }
            return ResolveResult::Pending;
          }
        };

        if next_seg_idx == import.segments.len() {
          if import.glob {
            match kind {
              ScopeKind::Ast(ScopeKindAst::Module(mod_id)) => mod_id.to_any(),
              ScopeKind::Hir(ScopeKindHir::Module(item_id)) => {
                if let Some(imod) = self.imods.iter().find(|im| im.expmap.cid() == item_id.cid()) {
                  return self.resolve_imod_import(imod, item_id, &[], true, import.span);
                }
                return ResolveResult::Failed;
              }
              _ => return ResolveResult::Failed,
            }
          } else {
            return ResolveResult::Specific {
              name: first_ident.sid(),
              kind,
              span,
            };
          }
        } else {
          match kind {
            ScopeKind::Ast(ScopeKindAst::Module(mod_id)) => mod_id.to_any(),
            ScopeKind::Hir(ScopeKindHir::Module(item_id)) => {
              if let Some(imod) = self.imods.iter().find(|im| im.expmap.cid() == item_id.cid()) {
                return self.resolve_imod_import(imod, item_id, &import.segments[1..], import.glob, import.span);
              }
              return ResolveResult::Failed;
            }
            _ => return ResolveResult::Failed,
          }
        }
      }
    };

    let mut current_scope_id = cur_scope_id;

    for (idx, seg) in import.segments[next_seg_idx..].iter().enumerate() {
      let seg_ident = match seg {
        ImportSegment::Name(ident) => *ident,
        _ => return ResolveResult::Failed,
      };

      let is_last_segment = (next_seg_idx + idx) == (import.segments.len() - 1);

      let scope = match scp.get(&current_scope_id) {
        Some(s) => s,
        None => return ResolveResult::Pending,
      };

      match scope.get(&seg_ident.sid()) {
        Some(&(item_kind, item_span)) => {
          if is_last_segment && !import.glob {
            return ResolveResult::Specific {
              name: seg_ident.sid(),
              kind: item_kind,
              span: item_span,
            };
          } else {
            match item_kind {
              ScopeKind::Ast(ScopeKindAst::Module(sub_mod_id)) => {
                current_scope_id = sub_mod_id.to_any();
              }
              ScopeKind::Hir(ScopeKindHir::Module(item_id)) => {
                if let Some(imod) = self.imods.iter().find(|im| im.expmap.cid() == item_id.cid()) {
                  let remaining = &import.segments[(next_seg_idx + idx + 1)..];
                  return self.resolve_imod_import(imod, item_id, remaining, import.glob, import.span);
                }
                return ResolveResult::Failed;
              }
              _ => return ResolveResult::Failed,
            }
          }
        }
        None => {
          return ResolveResult::Pending;
        }
      }
    }

    if import.glob {
      let target_scope = match scp.get(&current_scope_id) {
        Some(s) => s,
        None => return ResolveResult::Pending,
      };

      let mut items = Vec::new();
      for (&name_sid, &(item_kind, item_span)) in target_scope.iter() {
        items.push((name_sid, item_kind, item_span));
      }

      let is_fully_solved = target_scope.import.iter().all(|imp| imp.is_solved());

      ResolveResult::Glob {
        items,
        is_fully_solved,
      }
    } else {
      ResolveResult::Pending
    }
  }

  fn report_unresolved_imod_import(&self, imod: &Imod<'_>, segments: &[ImportSegment], glob: bool, _span: Span, sum: &mut Summary) {
    let mut current_item = match imod.expmap.root() {
      Some(r) => r,
      None => return,
    };

    for (idx, seg) in segments.iter().enumerate() {
      let seg_ident = match seg {
        ImportSegment::Name(ident) => *ident,
        _ => return,
      };

      let is_last_segment = idx == segments.len() - 1;

      let exports = match imod.expmap.get(&current_item.to_any()) {
        Some(e) => e,
        None => return,
      };

      match exports.get(&seg_ident.sid()) {
        Some(&export_kind) => {
          if is_last_segment {
            if glob {
              match export_kind {
                ExportKind::NameSpace(_) => {}
                _ => {
                  sum.add(Message::error(
                    EXPECTED_MODULE_FOR_GLOB,
                    Label::new_pos(seg_ident),
                  ));
                  return;
                }
              }
            }
          } else {
            match export_kind {
              ExportKind::NameSpace(sub_item) => {
                current_item = sub_item;
              }
              _ => {
                sum.add(Message::error(
                  EXPECTED_MODULE_IN_PATH,
                  Label::new_pos(seg_ident),
                ));
                return;
              }
            }
          }
        }
        None => {
          sum.add(Message::error(
            CANNOT_FIND_X_IN_SCOPE.args(&[self.sin.str(seg_ident.sid())]),
            Label::new_pos(seg_ident),
          ));
          return;
        }
      }
    }
  }

  fn report_unresolved_import(&self, scp: &ScopeMap, importing_scope_id: AnyId, import_idx: usize, sum: &mut Summary) {
    let scope = scp.get(&importing_scope_id).unwrap();
    let import = &scope.import[import_idx];

    if import.segments.is_empty() {
      sum.add(Message::error(
        EMPTY_IMPORT_PATH,
        Label::new_pos(import.span),
      ));
      return;
    }

    let mut next_seg_idx = 1;
    let cur_scope_id = match import.segments[0] {
      ImportSegment::Crate => self.root_id,
      ImportSegment::Super => {
        let mut cur = match scp.get(&importing_scope_id).and_then(|s| s.parent) {
          Some(p) => p,
          None => {
            sum.add(Message::error(
              CANNOT_USE_SUPER_AT_ROOT,
              Label::new_pos(import.span),
            ));
            return;
          }
        };

        while next_seg_idx < import.segments.len() {
          if let ImportSegment::Super = import.segments[next_seg_idx] {
            match scp.get(&cur).and_then(|s| s.parent) {
              Some(p) => cur = p,
              None => {
                sum.add(Message::error(
                  CANNOT_USE_SUPER_AT_ROOT,
                  Label::new_pos(import.span),
                ));
                return;
              }
            }
            next_seg_idx += 1;
          } else {
            break;
          }
        }
        cur
      }
      ImportSegment::Name(first_ident) => {
        if let Some(imod) = self.imods.iter().find(|im| im.name == first_ident.sid()) {
          self.report_unresolved_imod_import(imod, &import.segments[1..], import.glob, import.span, sum);
          return;
        }

        match self.find_in_scope_or_ancestors(scp, importing_scope_id, first_ident.sid()) {
          Some((item, _)) => {
            match item {
              ScopeKind::Ast(kind) => {
                if next_seg_idx == import.segments.len() {
                  if import.glob {
                    match kind {
                      ScopeKindAst::Module(mod_id) => mod_id.to_any(),
                      _ => {
                        sum.add(Message::error(
                          EXPECTED_MODULE_FOR_GLOB,
                          Label::new_pos(first_ident),
                        ));
                        return;
                      }
                    }
                  } else {
                    return;
                  }
                } else {
                  match kind {
                    ScopeKindAst::Module(mod_id) => mod_id.to_any(),
                    _ => {
                      sum.add(Message::error(
                        EXPECTED_MODULE_IN_PATH,
                        Label::new_pos(first_ident),
                      ));
                      return;
                    }
                  }
                }
              }
              ScopeKind::Hir(kind) => {
                if let ScopeKindHir::Module(item_id) = kind {
                  if let Some(imod) = self.imods.iter().find(|im| im.expmap.cid() == item_id.cid()) {
                    let mut cur = item_id;
                    for seg in &import.segments[next_seg_idx..] {
                      if let ImportSegment::Name(id) = seg {
                        if let Some(exp) = imod.expmap.get(&cur.to_any()) {
                          if let Some(&ExportKind::NameSpace(ns)) = exp.get(&id.sid()) {
                            cur = ns;
                            continue;
                          }
                        }
                      }
                    }
                    return;
                  }
                }
                return;
              }
            }
          }
          None => {
            let sugg = self.build_suggestion(scp, importing_scope_id, first_ident);
            sum.add(
              Message::error(
                CANNOT_FIND_X_IN_SCOPE.args(&[self.sin.str(first_ident.sid())]),
                Label::new_pos(first_ident),
              )
              .add_if(sugg),
            );
            return;
          }
        }
      }
    };

    let mut current_scope_id = cur_scope_id;

    for (idx, seg) in import.segments[next_seg_idx..].iter().enumerate() {
      let seg_ident = match seg {
        ImportSegment::Name(ident) => *ident,
        _ => {
          sum.add(Message::error(
            UNKNOWN_USE_SEGMENT,
            Label::new_pos(import.span),
          ));
          return;
        }
      };

      let is_last_segment = (next_seg_idx + idx) == (import.segments.len() - 1);

      let scope = match scp.get(&current_scope_id) {
        Some(s) => s,
        None => {
          sum.add(Message::error(
            MODULE_SCOPE_NOT_FOUND,
            Label::new_pos(seg_ident),
          ));
          return;
        }
      };

      match scope.get(&seg_ident.sid()) {
        Some(&(item_kind, _)) => {
          if is_last_segment {
            if import.glob {
              match item_kind {
                ScopeKind::Ast(ScopeKindAst::Module(_)) | ScopeKind::Hir(ScopeKindHir::Module(..)) => {}
                _ => {
                  sum.add(Message::error(
                    EXPECTED_MODULE_FOR_GLOB,
                    Label::new_pos(seg_ident),
                  ));
                  return;
                }
              }
            }
          } else {
            match item_kind {
              ScopeKind::Ast(ScopeKindAst::Module(sub_mod_id)) => {
                current_scope_id = sub_mod_id.to_any();
              }
              ScopeKind::Hir(..) => {
                return;
              }
              _ => {
                sum.add(Message::error(
                  EXPECTED_MODULE_IN_PATH,
                  Label::new_pos(seg_ident),
                ));
                return;
              }
            }
          }
        }
        None => {
          let sugg = self.build_suggestion(scp, current_scope_id, seg_ident);
          sum.add(
            Message::error(
              NOT_FOUND_IN_SCOPE.args(&[
                self.sin.str(seg_ident.sid()),
                "module",
              ]),
              Label::new_pos(seg_ident),
            )
            .add_if(sugg),
          );
          return;
        }
      }
    }
  }

  fn build_suggestion(&self, scp: &ScopeMap, scope_id: AnyId, ident: Ident) -> Option<Suggestion> {
    let scope = scp.get(&scope_id)?;
    let mut pool = Vec::new();
    for (&sid, &(_, span)) in scope.iter() {
      pool.push((self.sin.str(sid), span));
    }

    let input = self.sin.str(ident.sid());
    find_did_you_mean(input, &pool).map(|(matched, span)| {
      Suggestion::new(
        DID_YOU_MEAN.args(&[matched]),
        Applicability::MaybeIncorrect,
        matched.to_string(),
        Label::new(ident, YOU_SAID.args(&[input])),
      )
      .add(Label::new(span, X_DEFINED_HERE.args(&[matched])))
    })
  }

}
