/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast as ast;
use qwc_hir::{self as hir, ExprCategory};

use crate::hgen::Ctx;


pub fn find_duo_res_ty(ctx: &mut Ctx, pos: Span, blok: hir::TypeId, elsb: Option<hir::TypeId>) -> Result<hir::TypeId, Message> {
  let ty_unit = ctx.prims.ty_unit;
  let ty_never = ctx.prims.ty_never;

  let it = match elsb {
    None => blok,
      
    Some(elsb) => {
      match () {
        // Durum 1: Blok '!' döndürüyor (örn. hep panic atıyor). Else'in tipini al.
        _ if blok == ty_never => elsb,

        // Durum 2: Else '!' döndürüyor. Bloğun tipini al.
        _ if elsb == ty_never => blok,
        
        // Durum 3: İkisi de aynı tip.
        _ if blok == elsb => blok,
        
        // Durum 4: Blok hiçbir şey döndürmüyor, Else T döndürüyor -> ?T
        _ if blok == ty_unit => ctx.tin.ty_option(ctx.cre, ctx.ideps, elsb),

        // Durum 5: Blok T döndürüyor, Else hiçbir şey döndürmüyor -> ?T
        _ if elsb == ty_unit => ctx.tin.ty_option(ctx.cre, ctx.ideps, blok),

        // Durum 6: Birbiriyle alakasız iki tip (örn: i32 ve String)
        _ => return Err(Message::error(LOOP_BRANCHES_HAVE_INCOMPATIBLE_TYPE, Label::new_pos(pos)))
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
