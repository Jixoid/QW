
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WK {
  // Bases
  Number,
  String,
  Word,
  EOF,
  Unknown,

  // Shift

  /// <<=
  AssignmentLeftShift,
  /// >>=
  AssignmentRighShift,

  /// <<
  ShiftLeft,
  /// >>
  ShiftRigh,

  // Arithmetic

  /// +=
  AssignmentAdd,
  /// -=
  AssignmentSub,
  /// *=
  AssignmentMul,
  /// /=
  AssignmentDiv,
  /// %=
  AssignmentRem,

  // Logical

  /// &&=
  AssignmentLogicalAnd,
  /// ^^=
  AssignmentLogicalXor,
  /// ||=
  AssignmentLogicalOr,
  
  /// &&
  LogicalAnd,
  /// ^^
  LogicalXor,
  /// ||
  LogicalOr,

  // Bitwise

  /// &=
  AssignmentBitwiseAnd,
  /// ^=
  AssignmentBitwiseXor,
  /// |=
  AssignmentBitwiseOr,
  
  /// &
  BitwiseAnd,
  /// ^
  BitwiseXor,
  /// |
  BitwiseOr,

  // Brackets

  /// [
  SquareBracketBeg,
  /// ]
  SquareBracketEnd,

  /// {
  CurlyBracketBeg,
  /// }
  CurlyBracketEnd,

  /// (
  ParenBeg,
  /// )
  ParenEnd,

  /// <
  AngleBeg,
  /// >
  AngleEnd,

  // Equalities

  /// ==
  Equal,
  /// !=
  NotEqual,
  /// >=
  BiggerEqual,
  /// <=
  SmallerEqual,

  // Punctuation

  /// ::
  Scope,
  /// :
  Colon,
  /// ;
  Semicolon,
  /// ,
  Comma,
  /// .
  Dot,
  /// ..
  Dot2,
  /// #
  Hash,
  /// @
  At,
  /// ?
  Question,
  /// ~
  Tilde,
  /// `
  Backtick,

  // Assignment

  /// =
  Assign,

  // Arithmetic (single)

  /// +
  Add,
  /// -
  Sub,
  /// *
  Mul,
  /// /
  Div,
  /// %
  Rem,

  // Directives / Special

  /// #[
  Directive,
  /// ![
  Attribute,
  /// !
  Bang,

  /// _
  Underscore,

  /// <-
  ArrowLeft,
  /// ->
  ArrowRigh,
  /// =>
  FatArrow,

  /// <<|
  RotateLeft,
  /// |>>
  RotateRigh,

  // Keyword
  If, Ef, Else, Match, Loop, While, For, In, Let, Var,
  Using, Struct, Iface, Trait, Enum, Flags, Variant, Fun, Init, Fini, Generic, Mod, Use, Impl,
  Pub, Priv, Prot, Crate, Super, Mut, Imm,
  Ret, Break, Continue, Die, Unsafe, Relaxed,
  True, False, Undef, Unreachable,
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum CharKind {
  Ignored,
  Symbol,
  Numeral,
  String,
  Word,
}

const fn create_char_lut() -> [CharKind; 256] {
  let mut lut = [CharKind::Word; 256];

  // Whitespace
  let whitespace = [b' ', b'\n', b'\r', b'\t'];
  let mut i = 0;
  while i < whitespace.len() {
    lut[whitespace[i] as usize] = CharKind::Ignored;
    i += 1;
  }

  // Numeral ('0'..='9')
  let mut c = b'0';
  while c <= b'9' {
    lut[c as usize] = CharKind::Numeral;
    c += 1;
  }

  // String
  lut[b'\'' as usize] = CharKind::String;
  lut[b'"' as usize] = CharKind::String;

  // Symbols
  let symbols = [
    b'#', b'{', b'}', b'.', b':', b';', b',', b'=', b'(', b')', 
    b'<', b'>', b'[', b']', b'-', b'+', b'/', b'%', b'*', b'^', 
    b'~', b'&', b'|', b'@', b'?', b'!', b'`',
  ];
  let mut j = 0;
  while j < symbols.len() {
    lut[symbols[j] as usize] = CharKind::Symbol;
    j += 1;
  }

  lut
}

pub(crate) static CHAR_LUT: [CharKind; 256] = create_char_lut();
