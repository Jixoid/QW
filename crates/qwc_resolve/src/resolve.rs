use qwc_arena::Files;
use qwc_diagnostic::{Message, Span};
use qwc_ast as ast;

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
  far: &'ast Files,
  current: &'ast Scope,
}


impl<'ast> Resolver<'ast> {

  pub fn new(scp: &'ast ScopeMap, far: &'ast Files, current: &'ast Scope) -> Self {
    Self { scp, far, current }
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
      /* Me Scope */
      let me_scope = match current_target.kind {
        ScopeKind::Module(id) => self.scp.get(&id.to_any()),
        _ => panic!("This object does not have a scope"),
      };
      let me_scope = me_scope.unwrap_or_else(|| panic!());

      /* Search */
      let finded = match me_scope.get(&seg.sid()) {
        Some(v) => v,
        None => return Err(Message::error(seg, "`{}` not found in `{}` scope", &[seg.str(self.far), segments[i].str(self.far)]))
      };

      current_target = LookupResult { kind: finded.0, span: finded.1, in_scope: me_scope }
    }

    Ok(current_target)
  }

  
  fn lookup_in_scope(&self, ident: ast::Ident, scope: &'ast Scope) -> Result<LookupResult<'ast>, Message> {
    // Yerel Kapsam
    if let Some((kind, span)) = scope.get(&ident.sid()) {
      return Ok(LookupResult{kind: *kind, in_scope: scope, span: *span});
    }

    // Üst Kapsam Fallback
    if let Some(parent_id) = scope.parent {
      let parent = self.scp.get(&parent_id).unwrap();

      return self.lookup_in_scope(ident, parent);
    }

    // Hiçbir yerde bulunamadı
    Err(Message::error(ident, "unresolved identifier: `{}`", &[ident.str(self.far)]))
  }

}
