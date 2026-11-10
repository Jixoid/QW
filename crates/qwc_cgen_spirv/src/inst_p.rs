/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir::{self as mir, Expr, Terminator};
use rspirv::dr::Builder;
use rspirv::spirv::{StorageClass, Word};
use rustc_hash::FxHashMap;

use crate::{type_p::TypeLow, value_p::{FunCtx, ValueLow}};


pub struct InstLow;

impl InstLow {
  pub fn low_inst(
    b: &mut Builder,
    cre: &mir::Krate,
    ty_low: &mut TypeLow,
    symbols: &FxHashMap<mir::SymbId, Word>,
    fctx: &mut FunCtx,
    id: mir::InstId,
  ) -> Result<(), String> {
    let it = cre.get(id);

    let res_word = match it.kind {
      Expr::StackAlloca { kind } => {
        let ptr_ty = ty_low.pointer_type(b, cre, kind, StorageClass::Function)?;
        let var_id = b.variable(ptr_ty, None, StorageClass::Function, None);
        Some(var_id)
      }

      Expr::HeapStore { target, kind, value } => {
        let target_ptr = ValueLow::low(b, cre, ty_low, symbols, fctx, &target, None)?;
        let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &value, Some(kind))?;
        b.store(target_ptr, val_w, None, vec![]).map_err(|e| e.to_string())?;
        None
      }

      Expr::HeapLoad { target, kind } => {
        let target_ptr = ValueLow::low(b, cre, ty_low, symbols, fctx, &target, None)?;
        let res_ty = ty_low.low(b, cre, kind)?;
        let loaded = b.load(res_ty, None, target_ptr, None, vec![]).map_err(|e| e.to_string())?;
        Some(loaded)
      }

      Expr::Call { callee, args } => {
        let callee_id = ValueLow::low(b, cre, ty_low, symbols, fctx, &callee, None)?;
        let mut arg_words = Vec::new();
        for arg_id in cre.extra_get(args) {
          let arg_val = cre.get(arg_id);
          let arg_w = ValueLow::low(b, cre, ty_low, symbols, fctx, arg_val, None)?;
          arg_words.push(arg_w);
        }

        let ret_ty = match callee {
          mir::Value::GlobalRef(sym_id) => {
            let sym = cre.get(sym_id);
            if let mir::TypeKind::Fun { ret, .. } = cre.get(sym.ety).kind {
              ty_low.low(b, cre, ret)?
            } else {
              b.type_void()
            }
          }
          _ => b.type_void(),
        };

        let call_res = b.function_call(ret_ty, None, callee_id, arg_words).map_err(|e| e.to_string())?;
        Some(call_res)
      }

      Expr::Gep { target: _, kind: _, idx: _ } => {
        return Err("SPIR-V: GEP not yet implemented".to_string());
      }

      Expr::Select { cond, then_val, else_val, kind } => {
        let cond_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &cond, None)?;
        let then_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &then_val, Some(kind))?;
        let else_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &else_val, Some(kind))?;
        let res_ty = ty_low.low(b, cre, kind)?;
        let sel = b.select(res_ty, None, cond_w, then_w, else_w).map_err(|e| e.to_string())?;
        Some(sel)
      }

      Expr::IntArithmetic { op, flg: _, flg2, kind, lhs, rhs } => {
        let lhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &lhs, Some(kind))?;
        let rhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &rhs, Some(kind))?;
        let res_ty = ty_low.low(b, cre, kind)?;

        use mir::IntArithmeticOp::*;
        let res = match op {
          Add => b.i_add(res_ty, None, lhs_w, rhs_w),
          Sub => b.i_sub(res_ty, None, lhs_w, rhs_w),
          Mul => b.i_mul(res_ty, None, lhs_w, rhs_w),
          Div => match flg2 {
            mir::IntArithmeticFlg2::Signed => b.s_div(res_ty, None, lhs_w, rhs_w),
            mir::IntArithmeticFlg2::Unsigned => b.u_div(res_ty, None, lhs_w, rhs_w),
          },
          Rem => match flg2 {
            mir::IntArithmeticFlg2::Signed => b.s_rem(res_ty, None, lhs_w, rhs_w),
            mir::IntArithmeticFlg2::Unsigned => b.u_mod(res_ty, None, lhs_w, rhs_w),
          },
        }.map_err(|e| e.to_string())?;

        Some(res)
      }

      Expr::IntCondition { op, flg2, kind, lhs, rhs } => {
        let lhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &lhs, Some(kind))?;
        let rhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &rhs, Some(kind))?;
        let bool_ty = b.type_bool();

        use mir::IntConditionOp::*;
        let res = match op {
          Eq => b.i_equal(bool_ty, None, lhs_w, rhs_w),
          Ne => b.i_not_equal(bool_ty, None, lhs_w, rhs_w),
          Gt => match flg2 {
            mir::IntConditionFlg2::Signed => b.s_greater_than(bool_ty, None, lhs_w, rhs_w),
            mir::IntConditionFlg2::Unsigned => b.u_greater_than(bool_ty, None, lhs_w, rhs_w),
          },
          GtEq => match flg2 {
            mir::IntConditionFlg2::Signed => b.s_greater_than_equal(bool_ty, None, lhs_w, rhs_w),
            mir::IntConditionFlg2::Unsigned => b.u_greater_than_equal(bool_ty, None, lhs_w, rhs_w),
          },
          Lt => match flg2 {
            mir::IntConditionFlg2::Signed => b.s_less_than(bool_ty, None, lhs_w, rhs_w),
            mir::IntConditionFlg2::Unsigned => b.u_less_than(bool_ty, None, lhs_w, rhs_w),
          },
          LtEq => match flg2 {
            mir::IntConditionFlg2::Signed => b.s_less_than_equal(bool_ty, None, lhs_w, rhs_w),
            mir::IntConditionFlg2::Unsigned => b.u_less_than_equal(bool_ty, None, lhs_w, rhs_w),
          },
        }.map_err(|e| e.to_string())?;

        Some(res)
      }

      Expr::IntLogic { op, kind, lhs, rhs } => {
        let lhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &lhs, Some(kind))?;
        let rhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &rhs, Some(kind))?;
        let res_ty = ty_low.low(b, cre, kind)?;

        let res = match op {
          mir::IntLogicOp::And => b.bitwise_and(res_ty, None, lhs_w, rhs_w),
          mir::IntLogicOp::Or => b.bitwise_or(res_ty, None, lhs_w, rhs_w),
          mir::IntLogicOp::Xor => b.bitwise_xor(res_ty, None, lhs_w, rhs_w),
        }.map_err(|e| e.to_string())?;

        Some(res)
      }

      Expr::IntUnary { op, kind, val } => {
        let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &val, Some(kind))?;
        let res_ty = ty_low.low(b, cre, kind)?;

        let res = match op {
          mir::IntUnaryOp::Not => b.not(res_ty, None, val_w),
          mir::IntUnaryOp::Neg => b.s_negate(res_ty, None, val_w),
        }.map_err(|e| e.to_string())?;

        Some(res)
      }

      Expr::IntCast { kind, val, signed: _ } => {
        let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &val, None)?;
        let res_ty = ty_low.low(b, cre, kind)?;
        let res = b.bitcast(res_ty, None, val_w).map_err(|e| e.to_string())?;
        Some(res)
      }

      Expr::FloatArithmetic { op, kind, lhs, rhs } => {
        let lhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &lhs, Some(kind))?;
        let rhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &rhs, Some(kind))?;
        let res_ty = ty_low.low(b, cre, kind)?;

        use mir::FloatArithmeticOp::*;
        let res = match op {
          Add => b.f_add(res_ty, None, lhs_w, rhs_w),
          Sub => b.f_sub(res_ty, None, lhs_w, rhs_w),
          Mul => b.f_mul(res_ty, None, lhs_w, rhs_w),
          Div => b.f_div(res_ty, None, lhs_w, rhs_w),
          Rem => b.f_rem(res_ty, None, lhs_w, rhs_w),
        }.map_err(|e| e.to_string())?;

        Some(res)
      }

      Expr::FloatCondition { op, flg: _, kind, lhs, rhs } => {
        let lhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &lhs, Some(kind))?;
        let rhs_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &rhs, Some(kind))?;
        let bool_ty = b.type_bool();

        use mir::FloatConditionOp::*;
        let res = match op {
          Eq => b.f_ord_equal(bool_ty, None, lhs_w, rhs_w),
          Ne => b.f_ord_not_equal(bool_ty, None, lhs_w, rhs_w),
          Gt => b.f_ord_greater_than(bool_ty, None, lhs_w, rhs_w),
          GtEq => b.f_ord_greater_than_equal(bool_ty, None, lhs_w, rhs_w),
          Lt => b.f_ord_less_than(bool_ty, None, lhs_w, rhs_w),
          LtEq => b.f_ord_less_than_equal(bool_ty, None, lhs_w, rhs_w),
        }.map_err(|e| e.to_string())?;

        Some(res)
      }

      Expr::FloatUnary { op: _, kind, val } => {
        let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &val, Some(kind))?;
        let res_ty = ty_low.low(b, cre, kind)?;
        let res = b.f_negate(res_ty, None, val_w).map_err(|e| e.to_string())?;
        Some(res)
      }

      Expr::FloatCast { kind, val } => {
        let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &val, None)?;
        let res_ty = ty_low.low(b, cre, kind)?;
        let res = b.bitcast(res_ty, None, val_w).map_err(|e| e.to_string())?;
        Some(res)
      }

      Expr::IntToFloatCast { kind, val, signed } => {
        let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &val, None)?;
        let res_ty = ty_low.low(b, cre, kind)?;
        let res = if signed {
          b.convert_s_to_f(res_ty, None, val_w)
        } else {
          b.convert_u_to_f(res_ty, None, val_w)
        }.map_err(|e| e.to_string())?;
        Some(res)
      }

      Expr::FloatToIntCast { kind, val, signed } => {
        let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, &val, None)?;
        let res_ty = ty_low.low(b, cre, kind)?;
        let res = if signed {
          b.convert_f_to_s(res_ty, None, val_w)
        } else {
          b.convert_f_to_u(res_ty, None, val_w)
        }.map_err(|e| e.to_string())?;
        Some(res)
      }
    };

    if let (Some(dest), Some(word)) = (it.dest, res_word) {
      fctx.ssa_map.insert(dest, word);
    }

    Ok(())
  }

  pub fn low_term(
    b: &mut Builder,
    cre: &mir::Krate,
    ty_low: &mut TypeLow,
    symbols: &FxHashMap<mir::SymbId, Word>,
    fctx: &FunCtx,
    block_map: &FxHashMap<mir::BlokId, Word>,
    term: &Terminator,
  ) -> Result<(), String> {
    match term {
      Terminator::Jump(to) => {
        let target_label = block_map.get(to).copied().ok_or_else(|| {
          format!("SPIR-V: undefined jump target block {:?}", to)
        })?;
        b.branch(target_label).map_err(|e| e.to_string())?;
      }

      Terminator::Branch { cond, then_bb, else_bb } => {
        let cond_w = ValueLow::low(b, cre, ty_low, symbols, fctx, cond, None)?;
        let then_label = block_map.get(then_bb).copied().ok_or_else(|| {
          format!("SPIR-V: undefined then block {:?}", then_bb)
        })?;
        let else_label = block_map.get(else_bb).copied().ok_or_else(|| {
          format!("SPIR-V: undefined else block {:?}", else_bb)
        })?;
        b.branch_conditional(cond_w, then_label, else_label, vec![]).map_err(|e| e.to_string())?;
      }

      Terminator::Return(ret_val) => {
        match ret_val {
          None => {
            b.ret().map_err(|e| e.to_string())?;
          }
          Some(v) => {
            let val_w = ValueLow::low(b, cre, ty_low, symbols, fctx, v, None)?;
            b.ret_value(val_w).map_err(|e| e.to_string())?;
          }
        }
      }

      Terminator::Unreachable => {
        b.unreachable().map_err(|e| e.to_string())?;
      }
    }

    Ok(())
  }
}
