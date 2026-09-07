use qwc_diagnostic::Summary;
use qwc_hir as hir;
use qwc_mir as mir;
use qwc_string_interner::{Sid, StrInterner};

use crate::SymbLow;


pub struct Ctx<'ast, 'hir, 'mir> {
  pub src: &'hir hir::Krate,
  pub sin: &'ast StrInterner,
  pub cre: &'mir mut mir::Krate,
  pub sum: &'mir mut Summary,
  pub mgr: &'mir mut Vec<Sid>,
}


#[macro_export]
macro_rules! ctx {
  ($ctx:expr => $cre:ident, $sum:ident, $src:ident, $sin:ident, $mgr:ident) => {
    #[allow(unused_variables)]
    let Ctx{$cre, $sum, $src, $sin, $mgr} = $ctx;
  };
  
  ($cre:ident, $sum:ident, $src:ident, $sin:ident, $mgr:ident) => {
    &mut Ctx{$cre, $sum, $src, $sin, $mgr}
  };
}



pub struct MGen;

impl<'ast, 'hir, 'mir> MGen {

  pub fn low(src: &'hir hir::Krate, sin: &'ast StrInterner) -> (Option<mir::Krate>, Summary) {
    let mut cre = mir::Krate::new();
    let mut sum = Summary::new();

    let root = src.root().unwrap();

    let _ = SymbLow::low(
      &mut Ctx{cre: &mut cre, sum: &mut sum, src, sin, mgr: &mut vec![]},
      root
    ).map_err(|msg| sum.add(msg));

    (Some(cre), sum)
  }

}
