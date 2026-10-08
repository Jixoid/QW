/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir::{self as mir, Expr, SSA};

use crate::{FunCtx, TypeLow, ValueLow, cgen::{CtxI, CtxM}};


pub struct InstLow;

impl InstLow {

  pub fn low_result<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>, fctx: &mut FunCtx<'ctx>, inst_kind: &Expr, dest: SSA) {
    match *inst_kind {
      // Alloca
      Expr::Alloca { kind } => {
        let kind = TypeLow::low_basic_cached(uctx, ictx, kind);

        let a = ictx.builder.build_alloca(kind, "").unwrap();

        fctx.ssa_map.insert(dest, a.into());
      }

      // Memory
      Expr::Store{..} => panic!(),

      Expr::Load{target, kind} => {
        let target = ValueLow::low(uctx, ictx, fctx, &target).into_pointer_value();
        let kind = TypeLow::low_basic_cached(uctx, ictx, kind);
        
        let loaded = ictx.builder.build_load(kind, target, "").unwrap();
        
        fctx.ssa_map.insert(dest, loaded);
      }


      // Call
      Expr::Call{callee, args} => {
        let callee = ValueLow::low_fun(uctx, ictx, fctx, &callee).into_function_value();

        let args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = {
          let mut vec = vec![];

          for id in ictx.cre.extra_get(args) {
            let it = ictx.cre.get(id);
            let it = ValueLow::low(uctx, ictx, fctx, it);
            
            vec.push(it.into());
          }

          vec
        };

        let call_res = ictx.builder.build_direct_call(callee, &args, "").unwrap();
        let val = match call_res.try_as_basic_value() {
          inkwell::values::ValueKind::Basic(v) => v,
          inkwell::values::ValueKind::Instruction(_) => ictx.ctx.const_struct(&[], false).into(),
        };
        fctx.ssa_map.insert(dest, val);
      }



      Expr::Gep { target, kind, idx } => {
        let target_ptr = ValueLow::low(uctx, ictx, fctx, &target).into_pointer_value();
        let struct_ty = TypeLow::low_basic_cached(uctx, ictx, kind).into_struct_type();
        let field_ptr = ictx.builder.build_struct_gep(struct_ty, target_ptr, idx, "").unwrap();
        fctx.ssa_map.insert(dest, field_ptr.into());
      }


      // Binary
      Expr::IntArithmetic{op, flg: _, flg2, kind: _, lhs, rhs} => {
        let lhs = ValueLow::low(uctx, ictx, fctx, &lhs).into_int_value();
        let rhs = ValueLow::low(uctx, ictx, fctx, &rhs).into_int_value();

        use mir::IntArithmeticFlg2::*;

        match op {
          mir::IntArithmeticOp::Add => fctx.ssa_map.insert(dest, ictx.builder.build_int_add(lhs, rhs, "").unwrap().into()),
          mir::IntArithmeticOp::Sub => fctx.ssa_map.insert(dest, ictx.builder.build_int_sub(lhs, rhs, "").unwrap().into()),
          mir::IntArithmeticOp::Mul => fctx.ssa_map.insert(dest, ictx.builder.build_int_mul(lhs, rhs, "").unwrap().into()),

          mir::IntArithmeticOp::Div => match flg2 {
            Signed   => fctx.ssa_map.insert(dest, ictx.builder.build_int_signed_div(lhs, rhs, "").unwrap().into()),
            Unsigned => fctx.ssa_map.insert(dest, ictx.builder.build_int_unsigned_div(lhs, rhs, "").unwrap().into()),
          }
          
          mir::IntArithmeticOp::Rem => match flg2 {
            Signed   => fctx.ssa_map.insert(dest, ictx.builder.build_int_signed_rem(lhs, rhs, "").unwrap().into()),
            Unsigned => fctx.ssa_map.insert(dest, ictx.builder.build_int_unsigned_rem(lhs, rhs, "").unwrap().into()),
          }
        };
      }
    
      Expr::IntCondition{op, flg2, kind: _, lhs, rhs} => {
        let lhs = ValueLow::low(uctx, ictx, fctx, &lhs).into_int_value();
        let rhs = ValueLow::low(uctx, ictx, fctx, &rhs).into_int_value();

        use mir::IntConditionFlg2::*;

        let op = match op {
          mir::IntConditionOp::GtEq => match flg2 { Signed => inkwell::IntPredicate::SGE, Unsigned => inkwell::IntPredicate::UGE }
          mir::IntConditionOp::LtEq => match flg2 { Signed => inkwell::IntPredicate::SLE, Unsigned => inkwell::IntPredicate::ULE }
          
          mir::IntConditionOp::Gt => match flg2 { Signed => inkwell::IntPredicate::SGT, Unsigned => inkwell::IntPredicate::UGT }
          mir::IntConditionOp::Lt => match flg2 { Signed => inkwell::IntPredicate::SLT, Unsigned => inkwell::IntPredicate::ULT }
          
          mir::IntConditionOp::Eq => inkwell::IntPredicate::EQ,
          mir::IntConditionOp::Ne => inkwell::IntPredicate::NE,
        };

        fctx.ssa_map.insert(dest, ictx.builder.build_int_compare(op, lhs, rhs, "").unwrap().into());
      }
    
      Expr::IntLogic{op, kind: _, lhs, rhs} => {
        let lhs = ValueLow::low(uctx, ictx, fctx, &lhs).into_int_value();
        let rhs = ValueLow::low(uctx, ictx, fctx, &rhs).into_int_value();

        match op {
          mir::IntLogicOp::And => {
            fctx.ssa_map.insert(dest, ictx.builder.build_and(lhs, rhs, "").unwrap().into());
          }

          mir::IntLogicOp::Xor => {
            fctx.ssa_map.insert(dest, ictx.builder.build_xor(lhs, rhs, "").unwrap().into());
          }

          mir::IntLogicOp::Or => {
            fctx.ssa_map.insert(dest, ictx.builder.build_or(lhs, rhs, "").unwrap().into());
          }
        };
      }
    
      Expr::IntUnary{op, kind: _, val} => {
        let val = ValueLow::low(uctx, ictx, fctx, &val).into_int_value();

        match op {
          mir::IntUnaryOp::Not => {
            fctx.ssa_map.insert(dest, ictx.builder.build_not(val, "").unwrap().into());
          }
        };
      }

    }
  }


  pub fn low_sideff<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>, fctx: &mut FunCtx<'ctx>, inst_kind: &Expr) {
    match *inst_kind {
      // Alloca
      Expr::Alloca{..} => panic!(),


      // Memory
      Expr::Store{target, kind: _, value} => {
        let target = ValueLow::low(uctx, ictx, fctx, &target).into_pointer_value();
        let value = ValueLow::low(uctx, ictx, fctx, &value);

        ictx.builder.build_store(target, value).unwrap();
      }

      Expr::Load{..} => panic!(),


      // Call
      Expr::Call{callee, args} => {
        let callee = ValueLow::low_fun(uctx, ictx, fctx, &callee).into_function_value();

        let args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = {
          let mut vec = vec![];

          for id in ictx.cre.extra_get(args) {
            let it = ictx.cre.get(id);
            let it = ValueLow::low(uctx, ictx, fctx, it);
            
            vec.push(it.into());
          }

          vec
        };

        ictx.builder.build_direct_call(callee, &args, "").unwrap();
      }


      // Field
      Expr::Gep{..} => panic!(),

      // Binary
      Expr::IntArithmetic{..} |
      Expr::IntCondition{..} |
      Expr::IntLogic{..} |
      Expr::IntUnary{..} => panic!()
    }
  }

}
