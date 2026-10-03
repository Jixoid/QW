use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast as ast;
use qwc_hir::{self as hir, ExprCategory};

use crate::hgen::Ctx;


pub fn find_duo_res_ty(ctx: &mut Ctx, pos: Span, blok: hir::ExprId, elsb: Option<hir::ExprId>) -> Result<hir::TypeId, Message> {
  let ty_blok = (ctx.cre.get(blok) as &hir::Expr).ety;
  let opt_ty_elsb = elsb.map(|id| (ctx.cre.get(id) as &hir::Expr).ety);

  let ty_unit = ctx.tin.ty_unit();
  let ty_never = ctx.tin.ty_never();

  let it = match opt_ty_elsb {
    None => ty_blok,
      
    Some(ty_elsb) => {
      if ty_blok == ty_never {
        // Durum 1: Blok '!' döndürüyor (örn. hep panic atıyor). Else'in tipini al.
        ty_elsb
      } else if ty_elsb == ty_never {
        // Durum 2: Else '!' döndürüyor. Bloğun tipini al.
        ty_blok
      } else if ty_blok == ty_elsb {
        // Durum 3: İkisi de aynı tip.
        ty_blok
      } else if ty_blok == ty_unit {
        // Durum 4: Blok hiçbir şey döndürmüyor, Else T döndürüyor -> ?T
        ctx.tin.ty_option(ctx.cre, ty_elsb)
      } else if ty_elsb == ty_unit {
        // Durum 5: Blok T döndürüyor, Else hiçbir şey döndürmüyor -> ?T
        ctx.tin.ty_option(ctx.cre, ty_blok)
      } else {
        // Durum 6: Birbiriyle alakasız iki tip (örn: i32 ve String)
        return Err(Message::error(LOOP_BRANCHES_HAVE_INCOMPATIBLE_TYPE, Label::new_pos(pos)));
      }
    }
  };

  Ok(it)
}

pub fn verify_assignable(ctx: &Ctx, lhs_ast: ast::ExprId, lhs_hir: hir::ExprId, op_span: Span) -> Result<(), Message> {
  let lhs_ast = ctx.src.get(lhs_ast);
  let lhs_hir = ctx.cre.get(lhs_hir);


  match lhs_hir.category {
    ExprCategory::RValue => Err(Message::error(CANNOT_ASSIGN_RVALUE, Label::new_pos(op_span))
      .add(Label::new(lhs_ast.pos, CANNOT_ASSIGN_TO_THIS_EXPRESSION))
    ),
    
    ExprCategory::LValueImm => Err(Message::error(CANNOT_ASSIGN_IMMUTABLE, Label::new_pos(op_span))
      .add(Label::new(lhs_ast.pos, CANNOT_ASSIGN_TO_THIS_EXPRESSION))
    ),

    ExprCategory::LValueMut => Ok(())
  }
}
