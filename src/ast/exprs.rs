use crate::{ast::{self, AccessKind, ExprId, PattId, Rng, TypeId}, lexer::Span};


pub struct MatchArm {
  pub pat: ast::ExprId,
  pub body: ast::ExprId,
}

#[derive(Debug)]
pub enum BinaryOp {
  Add, Sub,
  Mul, Div, Mod,
  Eq, Neq,
  Lt, Gt,
  Lte, Gte,
  And, Or,
  Assign,
}

#[derive(Debug)]
pub enum UnaryOp {
  Neg, Poz, Not, Ref, Addr
}


pub enum Expr {
  Nick(Span),
  Path(Vec<ExprId>),
  Member(Vec<ExprId>),

  Tuple(Vec<ExprId>),

  Number(Span),
  String(Span),

  Block{label: Option<Span>, rng: Rng, expr: Option<ExprId>},

  If   {cond: ExprId, then: ExprId, elsb: Option<ExprId>},
  Match{cond: ExprId, arms: Vec<MatchArm>},
  While{cond: ExprId, blok: ExprId, elsb: Option<ExprId>},
  Loop {blok: ExprId, elsb: Option<ExprId>},
  ForIn{vars: PattId, iter: ExprId, blok: ExprId, elsb: Option<ExprId>},
  
  Binary{op: BinaryOp, lhs: ExprId, rhs: ExprId},
  Unary {op: UnaryOp, val: ExprId},
  
  Call {callee: ExprId, args: Vec<ExprId>},
  Index{callee: ExprId, args: Vec<ExprId>},

  Let{item: PattId, kind: Option<TypeId>, init: Option<ExprId>, acck: AccessKind},

  Return  {label: Option<Span>, val: Option<ExprId>},
  Break   {label: Option<Span>, val: Option<ExprId>},
  Continue{label: Option<Span>},

  Try(ExprId),
  Unwrap(ExprId),
  Unsafe(ExprId),
  Specialize{callee: ExprId, args: Vec<TypeId>},
}


impl Expr {
  pub fn vari_is_if(&self) -> bool { matches!(self, Expr::If{..} | Expr::Match{..}) }
}
