
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WK {
  // Bases
  Number,
  String,
  Word,
  Unknown,

  // < >
    /// <
    Lt,
    /// >
    Gt,

    /// <=
    LtEq,
    /// >=
    GtEq,

    /// <<
    Lt2,
    /// >>
    Gt2,

    /// <<=
    Lt2Eq,
    /// >>=
    Gt2Eq,

    // <>
    LtGt,
    // ><
    GtLt,

    // <=>
    LtEqGt,
    // >=<
    GtEqLt,


  // & | ^
    /// &
    Amp,
    /// |
    Pipe,
    /// ^
    Caret,
    
    /// &&
    Amp2,
    /// ||
    Pipe2,
    /// ^^
    Caret2,

    /// &&=
    Amp2Eq,
    /// ||=
    Pipe2Eq,
    /// ^^=
    Caret2Eq,
  

  // + - * / %
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
    
    /// +=
    AddEq,
    /// -=
    SubEq,
    /// *=
    MulEq,
    /// /=
    DivEq,
    /// %=
    RemEq,

    /// +|
    AddPipe,
    /// -|
    SubPipe,
    /// *|
    MulPipe,


  // () [] {}
    /// (
    ParenL,
    /// )
    ParenR,

    /// [
    BracketL,
    /// ]
    BracketR,

    /// {
    BraceL,
    /// }
    BraceR,


  // =
    /// =
    Eq,
    /// ==
    Eq2,


  // . , : ; 
    /// .
    Dot,
    /// ,
    Comma,
    /// :
    Colon,
    /// ;
    Semicolon,
    
    /// ::
    Colon2,
    /// ..
    Dot2,
    /// ..=
    Dot2Eq,
    /// ...
    Dot3,
  

  // !
    /// !
    Bang,
    /// !!
    Bang2,
    /// !=
    BangEq,
    /// ![
    BangAttr,


  // #
    /// #
    Hash,
    /// #[
    HashAttr,

  
  // @ ? ~ ` _
    /// @
    At,
    /// ?
    Question,
    /// ~
    Tilde,
    /// `
    Backtick,
    /// _
    Underscore,

  
  // Arrow
    /// <-
    ArrowLeft,
    /// ->
    ArrowRight,
    /// =>
    FatArrow,


  // Keyword
  If, Ef, Else, Match, Loop, While, For, In, Let, Var,
  Using, Struct, Iface, Trait, Enum, Flags, Variant, Fun, Init, Fini, Generic, Mod, Use, Impl,
  Pub, Priv, Prot, Crate, Super, Mut,
  Ret, Break, Continue, Die, Unsafe, Relaxed,
  True, False, Undef, Unreachable,
  
  /// Self
  SelfB,
  /// self
  SelfS,

  /// type
  Type,
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
