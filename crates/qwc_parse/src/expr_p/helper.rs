use qwc_ast::{BinaryOp, UnaryOp};
use qwc_lexer::WK;


// Helper
pub fn get_infix_bp(op: WK) -> Option<(u8, u8)> {
  use WK::*;

  match op {
    // Assign, Exchange
    Eq | ArrowLeft => Some((11, 10)),
    
    // Combinated Assign
    Lt2Eq | Gt2Eq |
    Amp2Eq | Pipe2Eq | Caret2Eq |
    AddEq | SubEq | MulEq | DivEq | RemEq => Some((11, 10)),

    // Logical
    Pipe2  => Some((20, 21)),
    Caret2 => Some((22, 23)),
    Amp2   => Some((24, 25)),
    
    // Equality
    Eq2 | BangEq => Some((30, 31)),
    
    // Size Comparisons
    Lt | Gt | LtEq | GtEq => Some((35, 36)),
    
    // Pipe Stream
    Pipe  => Some((40, 41)),
    
    // Range
    Dot2 | Dot2Eq => Some((45, 46)),
    
    // Shift
    Lt2 | Gt2 => Some((50, 51)),
    
    // Arithmetic
    Add | Sub => Some((60, 61)),
    Mul | Div | Rem => Some((70, 71)),
    
    _ => None,
  }
}

pub fn parse_unary_op(op: WK) -> UnaryOp {
  match op {
    // Prefix
    WK::Sub  => UnaryOp::Neg,
    WK::Add  => UnaryOp::Poz,
    WK::Bang => UnaryOp::Not,

    // Postfix
    WK::Question => UnaryOp::Try,
    WK::Bang2    => UnaryOp::Unwrap,
    WK::Amp      => UnaryOp::Ref,
    WK::Caret    => UnaryOp::Deref,

    _ => unreachable!("Unknown unary operator: {:?}", op),
  }
}

pub fn parse_binary_op(op: WK) -> BinOp {
  match op {
    // Arithmetic
    WK::Add => BinOp::Bin(BinaryOp::Add),
    WK::Sub => BinOp::Bin(BinaryOp::Sub),
    WK::Mul => BinOp::Bin(BinaryOp::Mul),
    WK::Div => BinOp::Bin(BinaryOp::Div),
    WK::Rem => BinOp::Bin(BinaryOp::Rem),
    
    WK::AddEq => BinOp::AssignOp(BinaryOp::Add),
    WK::SubEq => BinOp::AssignOp(BinaryOp::Sub),
    WK::MulEq => BinOp::AssignOp(BinaryOp::Mul),
    WK::DivEq => BinOp::AssignOp(BinaryOp::Div),
    WK::RemEq => BinOp::AssignOp(BinaryOp::Rem),

    // Shift
    WK::Lt2 => BinOp::Bin(BinaryOp::Shl),
    WK::Gt2 => BinOp::Bin(BinaryOp::Shr),

    WK::Lt2Eq => BinOp::AssignOp(BinaryOp::Shl),
    WK::Gt2Eq => BinOp::AssignOp(BinaryOp::Shr),

    // Comparison
    WK::Eq2    => BinOp::Bin(BinaryOp::Eq),
    WK::BangEq => BinOp::Bin(BinaryOp::Ne),
    
    WK::Lt => BinOp::Bin(BinaryOp::Lt),
    WK::Gt => BinOp::Bin(BinaryOp::Gt),
    WK::LtEq => BinOp::Bin(BinaryOp::LtEq),
    WK::GtEq => BinOp::Bin(BinaryOp::GtEq),
    
    // Logical
    WK::Amp2   => BinOp::Bin(BinaryOp::And),
    WK::Pipe2  => BinOp::Bin(BinaryOp::Or),
    WK::Caret2 => BinOp::Bin(BinaryOp::Xor),
    
    WK::Amp2Eq   => BinOp::AssignOp(BinaryOp::And),
    WK::Pipe2Eq  => BinOp::AssignOp(BinaryOp::Or),
    WK::Caret2Eq => BinOp::AssignOp(BinaryOp::Xor),

    // Pipe
    WK::Pipe => BinOp::Bin(BinaryOp::Pipe),
    
    // Special
    WK::Eq => BinOp::Assign,
    WK::ArrowLeft => BinOp::Exchange,

    _ => unreachable!("unknown binary operator: {:?}", op),
  }
}


pub enum BinOp {
  Bin(BinaryOp),
  AssignOp(BinaryOp),
  Assign,
  Exchange,
}
