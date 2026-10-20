/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


// The lexer expects its input to be valid UTF-8 (the loader guarantees it),
// so every test feeds it `&str`.

use std::{path::Path, sync::mpsc, thread, time::Duration};

use qwc_arena::Files;
use qwc_diagnostic::Summary;
use qwc_lexer::{Lexer, WK};


/// Runs `f` on its own thread so a lexer hang fails the test instead of blocking the suite.
fn with_timeout<T: Send + 'static>(secs: u64, f: impl FnOnce() -> T + Send + 'static) -> T {
  let (tx, rx) = mpsc::channel();

  thread::spawn(move || { tx.send(f()).ok(); });

  rx.recv_timeout(Duration::from_secs(secs)).expect("lexer hung or panicked")
}


fn lex_all(src: &str) -> Vec<(WK, String)> {
  let mut files = Files::new();
  let fid = files.add_buffer(Path::new("t.qw"), src.as_bytes().to_vec());
  let mut lx = Lexer::new(files.get(fid));

  let mut out = vec![];
  while let Some(w) = lx.get_safe() {
    out.push((w.kind(), lx.str(w).to_string()));
  }
  out
}

fn lex(src: &str) -> Vec<(WK, String)> {
  let src = src.to_owned();
  with_timeout(10, move || lex_all(&src))
}

fn kinds(src: &str) -> Vec<WK> {
  lex(src).into_iter().map(|(k, _)| k).collect()
}

fn tok(kind: WK, text: &str) -> (WK, String) {
  (kind, text.to_string())
}


// Keywords
#[test]
fn keywords_table() {
  let table = [
    ("if", WK::If), ("ef", WK::Ef), ("else", WK::Else), ("match", WK::Match),
    ("loop", WK::Loop), ("while", WK::While), ("for", WK::For), ("in", WK::In),
    ("let", WK::Let), ("var", WK::Var), ("using", WK::Using), ("struct", WK::Struct),
    ("iface", WK::Iface), ("trait", WK::Trait), ("enum", WK::Enum), ("flags", WK::Flags),
    ("variant", WK::Variant), ("fun", WK::Fun), ("task", WK::Task), ("init", WK::Init),
    ("fini", WK::Fini), ("generic", WK::Generic), ("mod", WK::Mod), ("use", WK::Use),
    ("impl", WK::Impl), ("pub", WK::Pub), ("priv", WK::Priv), ("prot", WK::Prot),
    ("crate", WK::Crate), ("super", WK::Super), ("mut", WK::Mut), ("ret", WK::Ret),
    ("break", WK::Break), ("continue", WK::Continue), ("die", WK::Die),
    ("unsafe", WK::Unsafe), ("relaxed", WK::Relaxed), ("true", WK::True),
    ("false", WK::False), ("undef", WK::Undef), ("unreachable", WK::Unreachable),
    ("as", WK::As), ("self", WK::SelfS), ("Self", WK::SelfB), ("type", WK::Type),
  ];

  for (kw, kind) in table {
    assert_eq!(lex(kw), vec![tok(kind, kw)], "keyword `{kw}`");
  }
}

#[test]
fn keyword_prefixes_are_words() {
  for src in ["ifx", "if_", "let_var", "true_val", "iff", "Selfie", "selfish", "types"] {
    assert_eq!(lex(src), vec![tok(WK::Word, src)], "`{src}`");
  }
}


// Symbols
#[test]
fn longest_match() {
  let table = [
    ("<<=", WK::Lt2Eq), ("<<", WK::Lt2), ("<=>", WK::LtEqGt), ("<=", WK::LtEq),
    (">>=", WK::Gt2Eq), (">>", WK::Gt2), (">=", WK::GtEq),
    ("&&=", WK::Amp2Eq), ("&&", WK::Amp2), ("||=", WK::Pipe2Eq), ("||", WK::Pipe2),
    ("^^=", WK::Caret2Eq), ("^^", WK::Caret2),
    ("..=", WK::Dot2Eq), ("...", WK::Dot3), ("..", WK::Dot2), ("::", WK::Colon2),
    ("==", WK::Eq2), ("!=", WK::BangEq), ("!!", WK::Bang2), ("![", WK::BangAttr),
    ("#[", WK::HashAttr), ("->", WK::ArrowRight), ("<-", WK::ArrowLeft), ("=>", WK::FatArrow),
  ];

  for (src, kind) in table {
    assert_eq!(lex(src), vec![tok(kind, src)], "`{src}`");
  }
}

#[test]
fn arrow_left_wins_over_less_than_minus() {
  // Pins current behavior: `a<-1` is a swap, not `a < -1`
  assert_eq!(kinds("a<-1"), vec![WK::Word, WK::ArrowLeft, WK::Number]);
  assert_eq!(kinds("a < -1"), vec![WK::Word, WK::Lt, WK::Sub, WK::Number]);
}


// Numbers
#[test]
fn numbers() {
  assert_eq!(lex("1.5"), vec![tok(WK::NumberPoint, "1.5")]);
  assert_eq!(lex("12abc"), vec![tok(WK::Number, "12abc")]);
  assert_eq!(lex("1.2.3"), vec![tok(WK::NumberPoint, "1.2"), tok(WK::Dot, "."), tok(WK::Number, "3")]);

  // A range is not a float
  assert_eq!(lex("1..2"), vec![tok(WK::Number, "1"), tok(WK::Dot2, ".."), tok(WK::Number, "2")]);
}


// Comments & whitespace
#[test]
fn comments_are_skipped() {
  assert_eq!(lex("// no newline at the end"), vec![]);
  assert_eq!(lex("/* /* nested */ */"), vec![]);
  assert_eq!(kinds("1 /* a /* b /* c */ */ */ + 2"), vec![WK::Number, WK::Add, WK::Number]);
  assert_eq!(kinds("a // x\nb"), vec![WK::Word, WK::Word]);
}

#[test]
fn whitespace_is_skipped() {
  assert_eq!(kinds("a\r\nb\t c"), vec![WK::Word, WK::Word, WK::Word]);
}


// Strings
#[test]
fn strings() {
  assert_eq!(lex("\"\""), vec![tok(WK::String, "\"\"")]);
  assert_eq!(lex("''"), vec![tok(WK::String, "''")]);
  assert_eq!(lex("\"abc\""), vec![tok(WK::String, "\"abc\"")]);
  assert_eq!(lex("\"a\\\"b\""), vec![tok(WK::String, "\"a\\\"b\"")]);
  assert_eq!(kinds("\"a\" \"b\""), vec![WK::String, WK::String]);
}


// peek / get / bump
#[test]
fn peek_get_bump_and_parse_token() {
  let mut files = Files::new();
  let fid = files.add_buffer(Path::new("t.qw"), b"a && b".to_vec());
  let mut lx = Lexer::new(files.get(fid));
  let mut sum = Summary::new();

  // peek does not advance
  assert_eq!(lx.peek(&mut sum).unwrap().kind(), WK::Word);
  assert_eq!(lx.peek(&mut sum).unwrap().kind(), WK::Word);

  let a = lx.get(&mut sum).unwrap();
  assert_eq!(lx.str(a), "a");

  // `&&` is split into two `&`
  lx.split_token(&mut sum).unwrap();

  let amp1 = lx.get(&mut sum).unwrap();
  assert_eq!((amp1.kind(), lx.str(amp1)), (WK::Amp, "&"));

  let amp2 = lx.get(&mut sum).unwrap();
  assert_eq!((amp2.kind(), lx.str(amp2)), (WK::Amp, "&"));

  lx.bump(&mut sum).unwrap();

  assert!(lx.get(&mut sum).is_fail());
  assert!(lx.peek(&mut sum).is_fail());
  assert!(lx.get_safe().is_none());
}


// peek / get / bump
#[test]
fn parse_nested_generic() {
  let mut files = Files::new();
  let fid = files.add_buffer(Path::new("t.qw"), b"Add<Vec2<i32>>".to_vec());
  let mut lx = Lexer::new(files.get(fid));
  let mut sum = &mut Summary::new();

  // test
  let out_s = lx.get(sum).unwrap();
  assert_eq!(out_s.kind(), WK::Word);
  assert_eq!(lx.str(out_s), "Add");
  
  assert_eq!(lx.get(sum).unwrap().kind(), WK::Lt);
  
  let in_s = lx.get(sum).unwrap();
  assert_eq!(in_s.kind(), WK::Word);
  assert_eq!(lx.str(in_s), "Vec2");
  
  assert_eq!(lx.get(sum).unwrap().kind(), WK::Lt);
  
  let t_s = lx.get(sum).unwrap();
  assert_eq!(t_s.kind(), WK::Word);
  assert_eq!(lx.str(t_s), "i32");
  
  // normal return `>>`
  assert_eq!(lx.peek(sum).unwrap().kind(), WK::Gt2);
  
  // split `>>` into two `>`
  lx.split_token(&mut sum).unwrap();
  
  assert_eq!(lx.get(sum).unwrap().kind(), WK::Gt);
  assert_eq!(lx.get(sum).unwrap().kind(), WK::Gt);

  // EOF
  assert!(lx.get(sum).is_fail());
}


// Unknown input: the lexer must always make progress
#[test]
fn stray_ascii_is_unknown() {
  for src in ["$", "\\", "\0", "\x0b", "\x0c", "\x7f"] {
    assert_eq!(lex(src), vec![tok(WK::Unknown, src)], "{src:?}");
  }

  assert_eq!(
    lex("a $ b"),
    vec![tok(WK::Word, "a"), tok(WK::Unknown, "$"), tok(WK::Word, "b")],
  );
}

#[test]
fn non_letter_characters_are_one_unknown_token_each() {
  // Not letters, so they cannot be part of a name: one Unknown token per character
  for src in ["€", "¥", "©", "§", "🔥", "→"] {
    assert_eq!(lex(src), vec![tok(WK::Unknown, src)], "{src}");
  }

  assert_eq!(lex("©§"), vec![tok(WK::Unknown, "©"), tok(WK::Unknown, "§")]);
  assert_eq!(lex("🔥🚀"), vec![tok(WK::Unknown, "🔥"), tok(WK::Unknown, "🚀")]);
  assert_eq!(lex("a🔥b"), vec![tok(WK::Word, "a"), tok(WK::Unknown, "🔥"), tok(WK::Word, "b")]);
  assert_eq!(lex("x€"), vec![tok(WK::Word, "x"), tok(WK::Unknown, "€")]);
}


// Names support UTF-8
#[test]
fn identifiers_are_utf8() {
  let names = [
    // Turkish
    "değişken", "özellik", "_özellik", "şart_123", "çocuk", "yaş", "ışık", "İnt", "ÇĞIŞÖÜ_çğışöü",
    // Other scripts
    "λ", "αβγ", "Δx", "переменная", "тест_1", "変数", "名前", "café", "über", "año",
  ];

  for name in names {
    assert_eq!(lex(name), vec![tok(WK::Word, name)], "{name}");
  }

  assert_eq!(
    lex("let değişken = öğe_1 + 2;"),
    vec![
      tok(WK::Let, "let"), tok(WK::Word, "değişken"), tok(WK::Eq, "="),
      tok(WK::Word, "öğe_1"), tok(WK::Add, "+"), tok(WK::Number, "2"), tok(WK::Semicolon, ";"),
    ],
  );
}

#[test]
fn utf8_names_are_not_keywords() {
  for name in ["ifade", "ile", "mutlu", "for_döngüsü", "ret_değeri", "impl_özellik", "var_mı", "ifş"] {
    assert_eq!(lex(name), vec![tok(WK::Word, name)], "{name}");
  }
}

#[test]
fn utf8_names_end_at_a_non_name_character() {
  assert_eq!(lex("foo🔥bar"), vec![tok(WK::Word, "foo"), tok(WK::Unknown, "🔥"), tok(WK::Word, "bar")]);
  assert_eq!(lex("şöyle.böyle"), vec![tok(WK::Word, "şöyle"), tok(WK::Dot, "."), tok(WK::Word, "böyle")]);
  assert_eq!(lex("a+ş"), vec![tok(WK::Word, "a"), tok(WK::Add, "+"), tok(WK::Word, "ş")]);
}

#[test]
fn utf8_name_rules_for_digits_and_underscore() {
  // A name cannot start with a digit, ASCII or not, but can contain one
  assert_eq!(lex("٣"), vec![tok(WK::Unknown, "٣")]);
  assert_eq!(lex("x٣"), vec![tok(WK::Word, "x٣")]);

  // A number takes the trailing name characters, like `12abc`
  assert_eq!(lex("12ş"), vec![tok(WK::Number, "12ş")]);

  // `_` alone is Underscore, but starts a name when a name character follows
  assert_eq!(kinds("_ _ş"), vec![WK::Underscore, WK::Word]);
}

#[test]
fn decomposed_letters_are_one_name() {
  // `s` + U+0327 COMBINING CEDILLA is the same `ş` as U+015F and must not split the name
  assert_eq!(lex("s\u{327}x"), vec![tok(WK::Word, "s\u{327}x")]);
}

#[test]
fn non_ascii_inside_strings_and_comments_is_kept() {
  assert_eq!(lex("\"Türkçe 🔥\""), vec![tok(WK::String, "\"Türkçe 🔥\"")]);
  assert_eq!(lex("'ğüş'"), vec![tok(WK::String, "'ğüş'")]);
  assert_eq!(kinds("// ç ğ ı\nlet"), vec![WK::Let]);
  assert_eq!(kinds("/* 🔥 /* ş */ ö */ let"), vec![WK::Let]);
}


// Unterminated constructs reach the end of the file as a single Unknown token
#[test]
fn unterminated_string() {
  assert_eq!(lex("\"abc"), vec![tok(WK::Unknown, "\"abc")]);
  assert_eq!(lex("'abc"), vec![tok(WK::Unknown, "'abc")]);
  assert_eq!(lex("\"abc\\"), vec![tok(WK::Unknown, "\"abc\\")]);
  assert_eq!(lex("let s = \"abc"), vec![
    tok(WK::Let, "let"), tok(WK::Word, "s"), tok(WK::Eq, "="), tok(WK::Unknown, "\"abc"),
  ]);
}

#[test]
fn unterminated_block_comment() {
  assert_eq!(lex("/* never closed"), vec![tok(WK::Unknown, "/* never closed")]);
  assert_eq!(lex("x /* a /* b */"), vec![tok(WK::Word, "x"), tok(WK::Unknown, "/* a /* b */")]);
}


// Tokens that do not fit in a `Word` (u16 length)
#[test]
fn token_length_limit() {
  let max = u16::MAX as usize;

  // Exactly at the limit: still a normal token
  let fits = format!("\"{}\"", "a".repeat(max - 2));
  assert_eq!(lex(&fits), vec![(WK::String, fits.clone())]);

  // One byte over: Unknown, truncated to the limit
  let over = format!("\"{}\"", "a".repeat(max - 1));
  assert_eq!(lex(&over), vec![(WK::Unknown, over[..max].to_string())]);

  // Long identifier and unterminated string behave the same way
  let ident = "a".repeat(70_000);
  assert_eq!(lex(&ident), vec![(WK::Unknown, ident[..max].to_string())]);

  let open = format!("\"{}", "a".repeat(70_000));
  assert_eq!(lex(&open), vec![(WK::Unknown, open[..max].to_string())]);
}

#[test]
fn truncated_token_ends_on_a_char_boundary() {
  // The limit (65535) must never land in the middle of a character.
  // `lex` reads the text back as `&str`, which would panic on a split character.
  // The padding shifts the characters so every alignment is covered.
  for (chr, count) in [("é", 40_000), ("€", 30_000), ("🔥", 20_000)] {
    for pad in ["", "a", "ab", "abc"] {
      let src = format!("\"{pad}{}\"", chr.repeat(count));
      let toks = lex(&src);

      assert_eq!(toks.len(), 1, "{chr} / pad {pad:?}");
      assert_eq!(toks[0].0, WK::Unknown, "{chr} / pad {pad:?}");
      assert!(toks[0].1.len() <= u16::MAX as usize, "{chr} / pad {pad:?}");
      assert!(toks[0].1.len() >= u16::MAX as usize - 3, "{chr} / pad {pad:?}");
      assert!(src.starts_with(&toks[0].1), "{chr} / pad {pad:?}");
    }
  }

  // Same for a name made of multi-byte letters
  for chr in ["é", "変", "𝒜"] {
    for pad in ["a", "ab", "abc"] {
      let src = format!("{pad}{}", chr.repeat(40_000));
      let toks = lex(&src);

      assert_eq!(toks.len(), 1, "name {chr} / pad {pad:?}");
      assert_eq!(toks[0].0, WK::Unknown, "name {chr} / pad {pad:?}");
      assert!(toks[0].1.len() <= u16::MAX as usize, "name {chr} / pad {pad:?}");
      assert!(toks[0].1.len() >= u16::MAX as usize - 3, "name {chr} / pad {pad:?}");
      assert!(src.starts_with(&toks[0].1), "name {chr} / pad {pad:?}");
    }
  }
}

#[test]
fn file_span_is_clamped_on_a_char_boundary() {
  let mut files = Files::new();
  let fid = files.add_buffer(Path::new("t.qw"), "é".repeat(50_000).into_bytes());
  let lx = Lexer::new(files.get(fid));

  let span = lx.pos_extend_file();

  assert!(span.range().len() <= u16::MAX as usize);
  assert!(span.range().len() >= u16::MAX as usize - 3);

  // Would panic if the span ended inside a character
  let _ = span.str(&files);
}


// Robustness
#[test]
fn random_utf8_always_terminates() {
  const POOL: &[&str] = &[
    "a", "Z", "_", "0", "9", " ", "\n", "\t", "\"", "'", "\\", "/", "*", "//", "/*", "*/",
    "<", ">", "=", "!", "#", "[", "]", ".", "$", "\0", "é", "ş", "€", "🔥",
  ];

  with_timeout(30, || {
    let mut state: u64 = 0x1234_5678_9abc_def0;
    let mut next = move || {
      state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
      (state >> 33) as usize
    };

    for _ in 0..500 {
      let count = next() % 64;
      let src: String = (0..count).map(|_| POOL[next() % POOL.len()]).collect();

      let total: usize = lex_all(&src).iter().map(|(_, text)| {
        assert!(!text.is_empty(), "empty token for {src:?}");
        text.len()
      }).sum();

      assert!(total <= src.len(), "tokens longer than the source for {src:?}");
    }
  });
}

#[test]
fn non_ascii_input_is_lexed_in_linear_time() {
  // Regression guard: validating the rest of the file at every non-ASCII
  // character made this quadratic (minutes for ~1 MB).
  let src = "let değişken = 1;\n".repeat(60_000);

  let count = with_timeout(20, move || lex_all(&src).len());

  // let, değişken, =, 1, ;
  assert_eq!(count, 60_000 * 5);
}
