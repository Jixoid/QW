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


fn prepare_parser(file: &str) -> ast::ItemRng {
  let cre = &mut ast::Krate::default();
  let sin = &mut StrInterner::default();
  let sid = sin.sid("test");
  let far = &mut Files::default();
  let fi = {
    let fid = far.add_buffer(Path::new("test.qw"), file.to_string().into_bytes());
    far.get(fid)
  };

  let root_path = cre.push(ast::DefPath::Root(sid));
  let (_, rng, sum) = Parse::parse(cre, root_path, sin, far, fi);

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
  prepare_parser(r#"
    impl Vec3<T>: core::ops::Add<Vec3<T>> {}
    impl Vec3<T>: core::ops::AddAssign<Vec3<T>> {}
  "#);
}


#[test]
fn two_ref() {
  prepare_parser(r#"
    struct A {
      a: &&();
      b: ^^();
    }
  "#);
}

#[test]
fn use_grouped() {
  prepare_parser(r#"
    use math::vec2::{Vec2};
    use math::vec2::{Vec2, Vec3};
    use math::vec2::{Vec2 as V2, Vec3};
    use math::vec2::{self, Vec2};
    use math::{vec2::Vec2, vec3::Vec3};
    use math::{vec2::{Vec2, SubVec}, *};
    use {math::vec2::Vec2, std::string::String};
  "#);
}

