/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Span;

use crate::{ExprId, ExprRng, Ident, PattId, ThingRng, TypeId, id::AnyRng};



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
  Add, Sub, Mul, Div, Rem,

  Eq, Ne, Lt, Gt, LtEq, GtEq,
  
  And, Or, Xor,
  
  Shl, Shr,

  Pipe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
  Neg, Poz,
  
  Not,
  
  Ref, Addr, Deref,
  
  Try, Unwrap,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
  // Access
  Nick   (Ident),
  Path   (ExprRng),
  Member (ExprRng),

  // Lit
  Unit,
  Bool   (Span, bool),
  Number (Span),
  String (Span),
  
  // Intrinsic
  SelfB(),
  SelfS(),

  // Data
  Tuple (ExprRng),
  Array (ExprRng),
  Propagate (ExprRng, ExprId),

  // Block
  Block {label: Option<Span>, rng: ExprRng, expr: Option<ExprId>},

  // Branch
  If    {cond: ExprId, then: ExprId, elsb: Option<ExprId>},
  Match {cond: ExprId, arms: ThingRng},
  
  // Loop
  While {cond: ExprId, blok: ExprId, elsb: Option<ExprId>},
  Loop  {blok: ExprId, elsb: Option<ExprId>},
  ForIn {vars: PattId, iter: ExprId, blok: ExprId, elsb: Option<ExprId>},
  
  // Operator
  Unary  {op: UnaryOp, val: ExprId},
  Binary {op: BinaryOp, lhs: ExprId, rhs: ExprId},
  
  Assign   {lhs: ExprId, rhs: ExprId, op_span: Span},
  AssignOp {op: BinaryOp, lhs: ExprId, rhs: ExprId, op_span: Span},
  
  Exchange {lhs: ExprId, rhs: ExprId},

  // Field Create
  FieldCreate {lhs: ExprId, fields: ThingRng, brace_span: Span},

  // Call
  Call  {callee: ExprId, args: ExprRng},
  Index {callee: ExprId, args: ExprRng},

  // Variable
  Let {item: PattId, kind: Option<TypeId>, init: Option<ExprId>, ism: bool},

  // Manage
  Return   {label: Option<Span>, val: Option<ExprId>},
  Break    {label: Option<Span>, val: Option<ExprId>},
  Continue {label: Option<Span>},
  Die      {lvar: Span},

  // Scope
  Unsafe (ExprId),
  Relaxed (ExprId),
  
  // Specialize
  Spec{callee: ExprId, args: AnyRng},

  // Cast
  Cast { expr: ExprId, kind: TypeId },
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
  pub pos: Span,
  pub kind: ExprKind,
}

impl Expr {
  pub fn is_like_blok(&self) -> bool {
    matches!(self.kind,
      ExprKind::Block{..} |
      ExprKind::If{..} | ExprKind::Match{..} |
      ExprKind::While{..} | ExprKind::Loop{..} | ExprKind::ForIn{..} |
      ExprKind::Unsafe{..} | ExprKind::Relaxed{..}
    )
  }
}
