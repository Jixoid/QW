use qwc_diagnostic::{Applicability, Label, Message, Span, Suggestion, msg::*};
use qwc_ast as ast;
use qwc_hir::{self as hir, CID, Deps};
use qwc_string_interner::StrInterner;
use strsim::damerau_levenshtein;

use crate::{LocalScopeManager, Scope, ScopeKind, ScopeKindAst, ScopeKindHir, ScopeMap};


#[derive(Copy, Clone)]
pub struct LookupResult<'ast> {
  pub kind: ScopeKind,
  pub span: Span,
  pub in_scope: &'ast Scope,
}

impl<'ast> LookupResult<'ast> {
  pub fn get_k(self) -> (ScopeKind, &'ast Scope, Span) {
    (self.kind, self.in_scope, self.span)
  }
}


pub struct Resolver<'ast, 'loc, 'imod> {
  scp: &'ast ScopeMap,
  sin: &'ast StrInterner,
  current: &'ast Scope,
  loc: Option<&'loc LocalScopeManager>,

  ideps: &'imod Deps,
  _imods: &'imod [CID],
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


impl<'ast, 'loc, 'imod> Resolver<'ast, 'loc, 'imod> {

  pub fn new(scp: &'ast ScopeMap, sin: &'ast StrInterner, current: &'ast Scope, ideps: &'imod Deps, imods: &'imod [CID]) -> Self {
    Self { scp, sin, current, loc: None, _imods: imods, ideps }
  }

  pub fn with_locals(scp: &'ast ScopeMap, sin: &'ast StrInterner, current: &'ast Scope, ideps: &'imod Deps, imods: &'imod [CID], loc: Option<&'loc LocalScopeManager>) -> Self {
    Self { scp, sin, current, loc, _imods: imods, ideps }
  }


  pub fn lookup(&self, ident: ast::Ident) -> Result<LookupResult<'ast>, Message> {
    self.lookup_in_scope(ident, self.current)
  }

  pub fn resolve_path(&self, segments: &[ast::Ident]) -> Result<LookupResult<'ast>, Message> {
    if segments.is_empty() { panic!(); }

    // İlk segmenti ara
    let mut current_target = self.lookup(segments[0])?;

    // Kalan segmentleri eklemeli ara
    for (i, &seg) in segments[1..].iter().enumerate() {
      match current_target.kind {
        ScopeKind::Ast(ScopeKindAst::Module(id)) => {
          let me_scope = self.scp.get(&id.to_any()).unwrap();

          let finded = match me_scope.get(&seg.sid()) {
            Some(v) => v,
            None => {
              let sugg = {
                let mut pool = vec![];
                
                for (&id, &(_, s)) in &me_scope.map {
                  pool.push((self.sin.str(id), s));
                }
                
                find_did_you_mean(self.sin.str(seg.sid()), &pool).map(|(str, span)| {
                  Suggestion::new(DID_YOU_MEAN.args(&[str]), Applicability::MaybeIncorrect, str.to_string(),
                    Label::new(seg, YOU_SAID.args(&[
                      self.sin.str(seg.sid())
                    ])))
                    .add(Label::new(span, X_DEFINED_HERE.args(&[str])))
                })
              };

              return Err(Message::error(NOT_FOUND_IN_SCOPE
                .args(&[
                  self.sin.str(seg.sid()),
                  self.sin.str(segments[i].sid())
                ]),
                Label::new_pos(seg))
                .add(Label::new(current_target.span, X_DEFINED_HERE.args(&[self.sin.str(segments[i].sid())])))
                .add_if(sugg)
              );
            }
          };

          current_target = LookupResult { kind: finded.0, span: finded.1, in_scope: me_scope };
        }

        ScopeKind::Hir(ScopeKindHir::Module(item_id)) => {
          let krate = self.ideps.get(item_id.cid());
          let item: &hir::Item = krate.get(item_id);
          let rng = match item.kind {
            hir::ItemKind::RootNS{rng} | hir::ItemKind::NameSpace{rng, ..} => rng,
            _ => panic!("This object does not have a scope"),
          };

          let mut found = None;
          for child in krate.extra_get(rng) {
            let child_id = hir::ItemId::new_from(child);
            let child_item: &hir::Item = krate.get(child_id);
            match child_item.kind {
              hir::ItemKind::NameSpace{name, ..} if name == seg.sid() => {
                found = Some(ScopeKind::Hir(ScopeKindHir::Module(child_id)));
                break;
              }
              hir::ItemKind::Using{name, kind} if name == seg.sid() => {
                found = Some(ScopeKind::Hir(ScopeKindHir::Type(kind)));
                break;
              }
              hir::ItemKind::Variable{name, kind: ty, ..} | hir::ItemKind::Function{name, kind: ty, ..} if name == seg.sid() => {
                found = Some(ScopeKind::Hir(ScopeKindHir::Expr(child_id, ty)));
                break;
              }
              _ => {}
            }
          }

          let next_kind = match found {
            Some(k) => k,
            None => {
              return Err(Message::error(NOT_FOUND_IN_SCOPE
                .args(&[
                  self.sin.str(seg.sid()),
                  self.sin.str(segments[i].sid())
                ]),
                Label::new_pos(seg))
                .add(Label::new(current_target.span, X_DEFINED_HERE.args(&[self.sin.str(segments[i].sid())])))
              );
            }
          };

          current_target = LookupResult {
            kind: next_kind,
            span: seg.into(),
            in_scope: current_target.in_scope,
          };
        }

        _ => panic!("This object does not have a scope"),
      }
    }

    Ok(current_target)
  }

  
  fn lookup_in_scope(&self, ident: ast::Ident, scope: &'ast Scope) -> Result<LookupResult<'ast>, Message> {
    // Local scope check (only when searching the starting/current scope)
    if std::ptr::eq(scope, self.current) {
      if let Some(loc) = self.loc {
        if let Some(local_id) = loc.lookup(&ident.sid()) {
          let local = loc.get_local(local_id);
          return Ok(LookupResult {
            kind: ScopeKind::Ast(ScopeKindAst::Local(local_id)),
            in_scope: scope,
            span: local.span,
          });
        }
      }
    }

    // Yerel Kapsam
    if let Some(&(kind, span)) = scope.get(&ident.sid()) {
      return Ok(LookupResult { kind, in_scope: scope, span });
    }

    // Üst Kapsam Fallback
    if let Some(parent_id) = scope.parent {
      let parent = self.scp.get(&parent_id).unwrap();

      return self.lookup_in_scope(ident, parent);
    }

    // Suggestion
    let sugg = {
      let mut pool = vec![];

      if let Some(loc) = self.loc {
        for local in &loc.locals {
          pool.push((self.sin.str(local.name), local.span));
        }
      }
      
      for (&id, &(_, s)) in &scope.map {
        pool.push((self.sin.str(id), s));
      }
      
      find_did_you_mean(self.sin.str(ident.sid()), &pool).map(|(str, span)| {
        Suggestion::new(DID_YOU_MEAN.args(&[str]), Applicability::MaybeIncorrect, str.to_string(),
          Label::new(ident, YOU_SAID.args(&[
            self.sin.str(ident.sid())
          ])))
          .add(Label::new(span, X_DEFINED_HERE.args(&[str])))
      })
    };

    // Hiçbir yerde bulunamadı
    Err(Message::error(CANNOT_FIND_X_IN_SCOPE.args(&[self.sin.str(ident.sid())]), Label::new_pos(ident)).add_if(sugg))
  }

}
