/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, Value};

use crate::{Ctx, ExprLow, FunBuilder, SymbLow};


pub fn low_cast_to_iface_ref(ctx: &mut Ctx, bbld: &mut FunBuilder, ref_of_expr: hir::ExprId, ref_of_type: hir::TypeId, target_iface: hir::TypeId) -> Result<Value, Message> {
  let ref_of_expr = ExprLow::low(ctx, bbld, ref_of_expr)?.unwrap();


  // Get VMT
  let vmt = Value::GlobalRef(SymbLow::get_or_low_vmt(ctx, ref_of_type, target_iface)?);
  
  // Get Types
  let ptr = ctx.tin.ty_ptr();
  let fatptr = ctx.tin.ty_fatptr();
  
  // Create
  let slot = bbld.build_alloca(fatptr);

  let f0 = bbld.emit(mir::Expr::Gep { target: slot, kind: fatptr, idx: 0 }).unwrap();
  bbld.emit(mir::Expr::Store { target: f0.into(), kind: ptr, value: ref_of_expr });
  let f1 = bbld.emit(mir::Expr::Gep { target: slot, kind: fatptr, idx: 1 }).unwrap();
  bbld.emit(mir::Expr::Store { target: f1.into(), kind: ptr, value: vmt });

  let fat_val = bbld.emit(mir::Expr::Load { target: slot, kind: fatptr }).unwrap();
  
  Ok(fat_val.into())
}

pub fn low_cast_to_trait(ctx: &mut Ctx, bbld: &mut FunBuilder, expr: hir::ExprId, _target_trait: hir::TypeId) -> Result<Option<Value>, Message> {
  ExprLow::low(ctx, bbld, expr)
}
