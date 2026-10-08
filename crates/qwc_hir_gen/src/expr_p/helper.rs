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


pub fn find_duo_res_ty(ctx: &mut Ctx, pos: Span, blok: hir::ExprId, elsb: Option<hir::ExprId>) -> Result<hir::TypeId, Message> {
  let ty_blok = (ctx.cre.get(blok) as &hir::Expr).ety;
  let opt_ty_elsb = elsb.map(|id| (ctx.cre.get(id) as &hir::Expr).ety);

  let ty_unit = ctx.prims.ty_unit;
  let ty_never = ctx.prims.ty_never;

  let it = match opt_ty_elsb {
    None => ty_blok,
      
    Some(ty_elsb) => {
      match () {
        // Durum 1: Blok '!' döndürüyor (örn. hep panic atıyor). Else'in tipini al.
        _ if ty_blok == ty_never => ty_elsb,

        // Durum 2: Else '!' döndürüyor. Bloğun tipini al.
        _ if ty_elsb == ty_never => ty_blok,
        
        // Durum 3: İkisi de aynı tip.
        _ if ty_blok == ty_elsb => ty_blok,
        
        // Durum 4: Blok hiçbir şey döndürmüyor, Else T döndürüyor -> ?T
        _ if ty_blok == ty_unit => {
          let lay = ctx.get_type(ty_elsb).layout;
          ctx.tin.ty_option(ctx.cre, ty_elsb, lay)
        }

        // Durum 5: Blok T döndürüyor, Else hiçbir şey döndürmüyor -> ?T
        _ if ty_elsb == ty_unit => {
          let lay = ctx.get_type(ty_blok).layout;
          ctx.tin.ty_option(ctx.cre, ty_blok, lay)
        }

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
