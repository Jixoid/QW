use crate::{ast, diagnostic::Message, hgen::{GenContext, TypeGen}, hir};


pub struct DeclGen;

impl DeclGen {

  pub fn low(ctx: &mut GenContext, id: ast::DeclId) -> Result<hir::TypeId, Message> {
    let it = ctx.ast.get_decl(id);

    match &it.vari {
      ast::DeclVari::Fun{kind, blok} => Self::low_fun(ctx, kind, blok),
      ast::DeclVari::Using{kind} => Self::low_using(ctx, kind),
      
      _ => todo!("unknown decl: {:?}", it.vari),
    }
  }


  fn low_fun(ctx: &mut GenContext, kind: &ast::TypeId, _blok: &Option<ast::ExprId>) -> Result<hir::TypeId, Message> {
    println!("DECL.FUN");

    let _ = TypeGen::low(ctx, *kind)?;

    panic!()
  }

  fn low_using(ctx: &mut GenContext, kind: &ast::TypeId) -> Result<hir::TypeId, Message> {
    println!("DECL.USING {}", kind);

    let _a = TypeGen::low(ctx, *kind)?;

    panic!()
  }

}
