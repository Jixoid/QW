/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir as mir;

use crate::{adapter::{low_type, low_val}, cgen::Ctx};


pub struct InstLow;

impl InstLow {
  pub fn low(ctx: &mut Ctx, id: mir::InstId) -> Result<(), String> {
    let it = ctx.src.get(id);

    use mir::Expr::*;
    match it.kind {
      StackAlloca{..} | HeapStore{..} | HeapLoad{..} => {
        return Err("memory operations (stack/heap alloca/load/store) are not supported in CIRCT".to_string());
      }

      IntArithmetic { op, flg: _, flg2, kind, lhs, rhs } => {
        let dest = it.dest.ok_or_else(|| "arithmetic instruction without destination".to_string())?;
        let ty_str = low_type(ctx.src, kind)?;
        let lhs_str = low_val(&lhs)?;
        let rhs_str = low_val(&rhs)?;

        let op_str = match op {
          mir::IntArithmeticOp::Add => "comb.add",
          mir::IntArithmeticOp::Sub => "comb.sub",
          mir::IntArithmeticOp::Mul => "comb.mul",
          mir::IntArithmeticOp::Div => match flg2 {
            mir::IntArithmeticFlg2::Signed => "comb.divs",
            mir::IntArithmeticFlg2::Unsigned => "comb.divu",
          },
          mir::IntArithmeticOp::Rem => match flg2 {
            mir::IntArithmeticFlg2::Signed => "comb.mods",
            mir::IntArithmeticFlg2::Unsigned => "comb.modu",
          },
        };

        writeln!(ctx.adap.stream(), "  %{} = {} {}, {} : {}", dest.get(), op_str, lhs_str, rhs_str, ty_str).unwrap();
      }

      IntCondition { op, flg2, kind, lhs, rhs } => {
        let dest = it.dest.ok_or_else(|| "condition instruction without destination".to_string())?;
        let ty_str = low_type(ctx.src, kind)?;
        let lhs_str = low_val(&lhs)?;
        let rhs_str = low_val(&rhs)?;

        let pred = match op {
          mir::IntConditionOp::Eq => "eq",
          mir::IntConditionOp::Ne => "ne",
          mir::IntConditionOp::Gt => match flg2 {
            mir::IntConditionFlg2::Signed => "sgt",
            mir::IntConditionFlg2::Unsigned => "ugt",
          },
          mir::IntConditionOp::GtEq => match flg2 {
            mir::IntConditionFlg2::Signed => "sge",
            mir::IntConditionFlg2::Unsigned => "uge",
          },
          mir::IntConditionOp::Lt => match flg2 {
            mir::IntConditionFlg2::Signed => "slt",
            mir::IntConditionFlg2::Unsigned => "ult",
          },
          mir::IntConditionOp::LtEq => match flg2 {
            mir::IntConditionFlg2::Signed => "sle",
            mir::IntConditionFlg2::Unsigned => "ule",
          },
        };

        writeln!(ctx.adap.stream(), "  %{} = comb.icmp {} {}, {} : {}", dest.get(), pred, lhs_str, rhs_str, ty_str).unwrap();
      }

      IntLogic { op, kind, lhs, rhs } => {
        let dest = it.dest.ok_or_else(|| "logic instruction without destination".to_string())?;
        let ty_str = low_type(ctx.src, kind)?;
        let lhs_str = low_val(&lhs)?;
        let rhs_str = low_val(&rhs)?;

        let op_str = match op {
          mir::IntLogicOp::And => "comb.and",
          mir::IntLogicOp::Or => "comb.or",
          mir::IntLogicOp::Xor => "comb.xor",
        };

        writeln!(ctx.adap.stream(), "  %{} = {} {}, {} : {}", dest.get(), op_str, lhs_str, rhs_str, ty_str).unwrap();
      }

      Select { cond, then_val, else_val, kind } => {
        let dest = it.dest.ok_or_else(|| "select instruction without destination".to_string())?;
        let ty_str = low_type(ctx.src, kind)?;
        let cond_str = low_val(&cond)?;
        let then_str = low_val(&then_val)?;
        let else_str = low_val(&else_val)?;

        writeln!(ctx.adap.stream(), "  %{} = comb.mux {}, {}, {} : {}", dest.get(), cond_str, then_str, else_str, ty_str).unwrap();
      }

      c => return Err(format!("unsupported expression in CIRCT: {:?}", c)),
    }

    Ok(())
  }
}
