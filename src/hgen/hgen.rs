use crate::{ast, diagnostic::Message, hgen::{ItemGen, scope}, hir, route::build::FileArena};


pub struct GenContext<'a,'h,'d> {
  pub ast: &'a ast::Crate,
  pub hir: &'h mut hir::Crate<'d>,
  pub far: &'a FileArena,
  pub self_type: Option<ast::TypeId>,
  
  pub(super) local: scope::Local,
  pub(super) global: scope::Global,

  pub type_cache: std::collections::HashMap<ast::TypeId, hir::TypeId>,
  pub decl_cache: std::collections::HashMap<ast::DeclId, hir::TypeId>,
  pub type_bounds: std::collections::HashMap<String, Vec<hir::TypeId>>,
  pub type_to_ast: std::collections::HashMap<hir::TypeId, ast::TypeId>,
}


pub struct HGen;

impl HGen {

  pub fn lower<'a,'d>(ast: &'a ast::Crate, far: &'a FileArena) -> Result<hir::Crate<'d>, Message> {
    let mut ret = hir::Crate::new();
    
    let mut gctx = GenContext {
      ast,
      hir: &mut ret,
      far,
      self_type: None,

      local: scope::Local::new(),
      global: scope::Global::new(),

      type_cache: std::collections::HashMap::new(),
      decl_cache: std::collections::HashMap::new(),
      type_bounds: std::collections::HashMap::new(),
      type_to_ast: std::collections::HashMap::new(),
    };

    ret.root = ItemGen::low_item(&mut gctx, ast.root)?;

    Ok(ret)
  }

}
