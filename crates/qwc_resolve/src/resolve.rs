use qwc_diagnostic::{Applicability, Label, Message, Span, Suggestion, msg::*};
use qwc_ast as ast;
use qwc_string_interner::StrInterner;
use strsim::damerau_levenshtein;

use crate::{Scope, ScopeKind, ScopeMap};


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


pub struct Resolver<'ast> {
  scp: &'ast ScopeMap,
  sin: &'ast StrInterner,
  current: &'ast Scope,
}


fn find_did_you_mean<'a, Span: Copy>(input: &str, candidates: &[(&'a str, Span)]) -> Option<(&'a str, Span)> {
  let max_distance = (input.chars().count() / 3).max(1);

  candidates
    .iter()
    // (aday, span) ikilisini koruyup yanına hesaplanan mesafeyi ekliyoruz
    .map(|&(candidate, span)| ((candidate, span), damerau_levenshtein(input, candidate)))
    // Yalnızca izin verilen mesafedeki adayları filtreliyoruz
    .filter(|&(_, dist)| dist <= max_distance)
    // Mesafeye göre en küçüğünü seçiyoruz
    .min_by_key(|&(_, dist)| dist)
    // Mesafeyi atıp geriye ((candidate, span)) ikilisini döndürüyoruz
    .map(|(matched, _)| matched)
}


impl<'ast> Resolver<'ast> {

  pub fn new(scp: &'ast ScopeMap, sin: &'ast StrInterner, current: &'ast Scope) -> Self {
    Self { scp, sin, current }
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
      // Me Scope
      let me_scope = match current_target.kind {
        ScopeKind::Module(id) => self.scp.get(&id.to_any()).unwrap(),
        _ => panic!("This object does not have a scope"),
      };


      // Search
      let finded = match me_scope.get(&seg.sid()) {
        Some(v) => v,
        None => {
          // Suggestion
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
          )
        }
      };

      current_target = LookupResult { kind: finded.0, span: finded.1, in_scope: me_scope }
    }

    Ok(current_target)
  }

  
  fn lookup_in_scope(&self, ident: ast::Ident, scope: &'ast Scope) -> Result<LookupResult<'ast>, Message> {
    // Yerel Kapsam
    if let Some(&(kind, span)) = scope.get(&ident.sid()) {
      return Ok(LookupResult{kind, in_scope: scope, span});
    }

    // Üst Kapsam Fallback
    if let Some(parent_id) = scope.parent {
      let parent = self.scp.get(&parent_id).unwrap();

      return self.lookup_in_scope(ident, parent);
    }

    // Suggestion
    let sugg = {
      let mut pool = vec![];
      
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
