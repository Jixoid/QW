/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, Const};


pub struct Evaluator<'a> {
  hir: &'a hir::Krate,
}

impl<'a> Evaluator<'a> {
  pub fn new(hir: &'a hir::Krate) -> Self {
    Self { hir }
  }
}


impl<'a> Evaluator<'a> {
  pub fn eval(&self, sum: &mut Summary, id: hir::ExprId) -> Status<Const> {
    let mut id = id;

    'ml: loop {
      macro_rules! next {
        ($expr:expr) => { id = $expr; continue 'ml; };
      }

      let it = self.hir.get(id);
      
      let val = match it.kind {
        hir::ExprKind::Const(v) => v,
        
        hir::ExprKind::Block {stmt, expr} => {
          if !stmt.is_empty() { ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE)) }
          
          let Some(expr) = expr else { ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE)) };
          
          next!(expr);
        }
        
        hir::ExprKind::IntArithmetic {op, flg, lhs, rhs} => {
          let lhs = self.eval(sum, lhs)?.get_u64(sum)?;
          let rhs = self.eval(sum, rhs)?.get_u64(sum)?;

          let a = match op {
            hir::IntArithmeticOp::Add => match flg {
              hir::IntArithmeticFlg::Overflow => lhs.wrapping_add(rhs),
              hir::IntArithmeticFlg::Saturating => lhs.saturating_add(rhs),
              hir::IntArithmeticFlg::Checked => ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE)),
            },
            hir::IntArithmeticOp::Sub => match flg {
              hir::IntArithmeticFlg::Overflow => lhs.wrapping_sub(rhs),
              hir::IntArithmeticFlg::Saturating => lhs.saturating_sub(rhs),
              hir::IntArithmeticFlg::Checked => ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE)),
            },
            hir::IntArithmeticOp::Mul => match flg {
              hir::IntArithmeticFlg::Overflow => lhs.wrapping_mul(rhs),
              hir::IntArithmeticFlg::Saturating => lhs.saturating_mul(rhs),
              hir::IntArithmeticFlg::Checked => ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE)),
            },
            hir::IntArithmeticOp::Div => match flg {
              hir::IntArithmeticFlg::Overflow => lhs.wrapping_div(rhs),
              hir::IntArithmeticFlg::Saturating => lhs.saturating_div(rhs),
              hir::IntArithmeticFlg::Checked => ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE)),
            },
            hir::IntArithmeticOp::Rem => match flg {
              hir::IntArithmeticFlg::Overflow => lhs.wrapping_rem(rhs),
              hir::IntArithmeticFlg::Saturating | hir::IntArithmeticFlg::Checked => ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE)),
            },
          };

          Const::U64(a)
        }

        hir::ExprKind::GenericExpr {..} => ret_err!(sum, Message::error(FIXED_VALUE_CANNOT_BE_EVALUATED, Label::new_pos(it.pos))),

        _ => ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE).add(ICE_MESSAGE.args(&[&format!("evaluator kind: {it:#?}")])))
      };
      
      return Status::Success(val);
    }
  }
}



pub trait GetConst {
  fn get_u64(self, sum: &mut Summary) -> Status<u64>;
}

impl GetConst for Const {
  fn get_u64(self, sum: &mut Summary) -> Status<u64> {
    let val = match self {
      Const::U8(v) => v as u64,
      Const::U16(v) => v as u64,
      Const::U32(v) => v as u64,
      Const::U64(v) => v,
      
      Const::I8(v) if v >= 0 => v as u64,
      Const::I16(v) if v >= 0 => v as u64,
      Const::I32(v) if v >= 0 => v as u64,
      Const::I64(v) if v >= 0 => v as u64,
      
      _ => ret_err!(sum, Message::fatal(ICE_UNSUPPORTED_FEATURE))
    };

    Status::Success(val)
  }
}
