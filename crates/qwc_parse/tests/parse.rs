/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;

use qwc_arena::Files;
use qwc_ast as ast;
use qwc_parse::Parse;
use qwc_string_interner::StrInterner;


fn prepare_parser(file: String) -> ast::ItemRng {
  let cre = &mut ast::Krate::new();
  let sin = &mut StrInterner::new();
  let sid = sin.sid("test");
  let far = &mut Files::new();
  let fi = {
    let fid = far.add_buffer(Path::new("test.qw"), file.into_bytes());
    far.get(fid)
  };

  let (_, rng, sum) = Parse::parse(cre, sid, sin, far, fi);

  if !sum.is_empty() {
    for msg in &sum {
      eprintln!("{}", msg.display_human(far));
    }
    eprintln!("{}", sum);

    panic!()
  }

  rng
}


#[test]
fn nested_generic() {
  prepare_parser(String::from("
    impl Vec3<T>: core::ops::Add<Vec3<T>> {}
    impl Vec3<T>: core::ops::AddAssign<Vec3<T>> {}
  "));
}


#[test]
fn two_ref() {
  prepare_parser(String::from("
    struct A {
      a: &&();
      b: ^^();
    }
  "));
}
