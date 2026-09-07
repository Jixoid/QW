use qwc_diagnostic::Span;

use crate::{ExprId, Ident, PattId, Rng, TypeId};



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
  Add, Sub, Mul, Div, Rem,

  Eq, Ne, Lt, Gt, Lte, Gte,
  
  And, Or, Xor, Shl, Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
  Neg, Poz, Not, Ref, Addr, Deref
}


#[derive(Debug, Copy, Clone)]
pub enum ExprKind {
  // Access
  Nick   (Ident),
  Path   (Rng /* ExprId */),
  Member (Rng /* ExprId */),

  // Lit
  Unit,
  Bool   (Span, bool),
  Number (Span),
  String (Span),
  
  // Intrinsic
  SelfB(),
  SelfS(),

  // Data
  Tuple (Rng /* ExprId */),
  Array (Rng /* ExprId */),
  Propagate (Rng /* ExprId */, ExprId),

  // Block
  Block {label: Option<Span>, rng: Rng /* ExprId */, expr: Option<ExprId>},

  // Branch
  If    {cond: ExprId, then: ExprId, elsb: Option<ExprId>},
  Match {cond: ExprId, arms: Rng /* MatchArm */},
  
  // Loop
  While {cond: ExprId, blok: ExprId, elsb: Option<ExprId>},
  Loop  {blok: ExprId, elsb: Option<ExprId>},
  ForIn {vars: PattId, iter: ExprId, blok: ExprId, elsb: Option<ExprId>},
  
  // Operator
  Unary  {op: UnaryOp, val: ExprId},
  Binary {op: BinaryOp, lhs: ExprId, rhs: ExprId},

  Assign   {lhs: ExprId, rhs: ExprId},
  AssignOp {op: BinaryOp, lhs: ExprId, rhs: ExprId},
  
  Exchange {lhs: ExprId, rhs: ExprId},

  // Call
  Call  {callee: ExprId, args: Rng /* ExprId */},
  Index {callee: ExprId, args: Rng /* ExprId */},

  // Variable
  Let {item: PattId, kind: Option<TypeId>, init: Option<ExprId>, ism: bool},

  // Manage
  Return   {label: Option<Span>, val: Option<ExprId>},
  Break    {label: Option<Span>, val: Option<ExprId>},
  Continue {label: Option<Span>},
  Die      {lvar: Span},

  // Try
  Try (ExprId),
  Unwrap (ExprId),
  
  // Scope
  Unsafe (ExprId),
  Relaxed (ExprId),
  
  // Specialize
  Spec{callee: ExprId, args: Rng /* TypeId | ExprId */},
}


#[derive(Debug, Copy, Clone)]
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
