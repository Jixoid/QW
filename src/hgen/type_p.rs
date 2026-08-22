use std::assert_matches;

use crate::{ast::{self, AstKind, Rng}, diagnostic::Message, hgen::GenContext, hir, lexer::Span};


pub struct TypeGen;

impl TypeGen {

  pub fn low(ctx: &mut GenContext, id: ast::TypeId) -> Result<hir::TypeId, Message> {
    let it = ctx.ast.get_type(id);

    match &it {
      ast::Type::Nick  {pos, ..} => Self::low_nick(ctx, pos),
      ast::Type::Struct{vars, bases} => Self::low_struct(ctx, vars, bases),
      ast::Type::Array {sub, ext} => Self::low_array(ctx, sub, ext),
      ast::Type::Fun   {args, ret, attr} => Self::low_fun(ctx, args, ret, attr),
      
      _ => todo!("unknown type: {:?}", it),
    }
  }


  fn low_nick(ctx: &mut GenContext, pos: &Span) -> Result<hir::TypeId, Message> {
    println!("TYPE.NICK {:?}", pos.str(ctx.far));

    Err(Message::error(*pos, "unkown nick: {}", vec![ pos.string(ctx.far) ]))
  }

  fn low_struct(ctx: &mut GenContext, vars: &Rng, _bases: &Option<Rng>) -> Result<hir::TypeId, Message> {
    println!("TYPE.STRUCT [{:?}]", vars);

    let exda = ctx.ast.get_extra(vars.clone());
    
    for y in exda {
      assert_matches!(y.kind, AstKind::Thing);

      let z = ctx.ast.get_thing(ast::ThingId::new_from(*y));

      assert_matches!(z, ast::Thing::NamedTypeVis(..));

      if let ast::Thing::NamedTypeVis(w, v, k) = z {
        println!("INVAR: {:?} {} {}", v, w.str(ctx.far), k);

        let _a = TypeGen::low(ctx, *k)?;
      }
    }

    panic!();
    //Err(Message::error(*pos, "cannot low struct: {}", vec![]))
  }

  fn low_array(ctx: &mut GenContext, sub: &ast::TypeId, ext: &Option<Rng>) -> Result<hir::TypeId, Message> {
    println!("TYPE.ARRAY {} [{:?}]", sub, ext);

    let _a = TypeGen::low(ctx, *sub)?;

    if let Some(ext) = ext { 
      let exda = ctx.ast.get_extra(ext.clone());
      
      for x in exda {
        assert_matches!(x.kind, AstKind::Expr);

        // Check Expr
      }
    }

    panic!();
    //Err(Message::error(*pos, "cannot low array: {}", vec![]))
  }

  fn low_fun(_ctx: &mut GenContext, args: &Rng, ret: &Option<ast::TypeId>, _attr: &u8) -> Result<hir::TypeId, Message> {
    println!("TYPE.FUN args[{:?}] {:?}", args, ret);

    panic!();
    //Err(Message::error(*pos, "cannot crated fun: {}", vec![]))
  }

}
