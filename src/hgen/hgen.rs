use crate::{ast, diagnostic::Message, hgen::ItemGen, hir, route::build::FileArena};


pub struct GenContext<'a,'h,'d> {
  pub ast: &'a ast::Crate,
  pub hir: &'h mut hir::Crate<'d>,
  pub far: &'a FileArena,
}


pub struct HGen;

impl HGen {

  pub fn lower<'a,'d>(ast: &'a ast::Crate, far: &'a FileArena) -> Result<hir::Crate<'d>, Message> {
    let mut ret = hir::Crate::new();
    
    let mut gctx = GenContext {ast, hir: &mut ret, far};

    let _a = ItemGen::low(&mut gctx, ast.root)?;

    Ok(ret)
  }

}
