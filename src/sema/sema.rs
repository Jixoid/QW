use crate::{control::{AstId, Module}, diagnostic::Summary};



pub struct Sema<'f, 'a, 'd:'a> {
  pub mol: &'f mut Module<'a,'d>,
  pub sum: Summary<'a>,
  pub visitors: Vec<AstId>,
}


impl<'f,'a,'d> Sema<'f,'a,'d> {

  pub fn new(mol: &'f mut Module<'a,'d>) -> Self {
    let sum = Summary::new();

    Self{mol, sum, visitors: Vec::new()}
  }


  pub fn check(&mut self) -> Summary<'a> {
    if !self.mol.list_decl.is_empty() {
      let root_id = crate::control::AstId::new(crate::control::IdentyKind::Decl, 0, 0);
      if let Err(e) = self.check_decl(root_id) {
        self.sum.add(e);
      }
    }

    self.sum.clone()
  }

  pub fn check_attributes(&mut self, id: crate::control::identy::AstId) -> Result<(), crate::diagnostic::Message<'a>> {
    if let Some(attrs) = self.mol.map_attr.get(&id) {
      for attr in attrs {
        let name = attr.key.str();
        if name == "mangle" {
          if let Some(val) = &attr.val {
            let v = val.str();
            if v != "bare" && v != "itanium" && v != "qw" {
              return Err(crate::diagnostic::Message::error(attr.key, format!("invalid value for mangle attribute: {}", v), vec![]));
            }
          } else {
            return Err(crate::diagnostic::Message::error(attr.key, "mangle attribute requires a value (e.g. mangle:bare)".to_string(), vec![]));
          }
        } else if name == "weak" {
          if attr.val.is_some() {
            return Err(crate::diagnostic::Message::error(attr.key, "weak attribute does not take a value".to_string(), vec![]));
          }
        } else {
          return Err(crate::diagnostic::Message::error(attr.key, format!("unknown attribute: {}", name), vec![]));
        }
      }
    }
    Ok(())
  }

}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::control::module::{ModuleFile, ModuleKind};
  use crate::front::Front;
  use crate::sys::SysFile;
  use crate::route::build::{FileArena, ModInjection};

  #[test]
  fn test_sema_valid() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: "fun main() -> void {}".to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    for m in sum.msgs() { println!("{}", m); } assert_eq!(sum.sumerr(), 0);
  }

  #[test]
  fn test_sema_invalid() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: "fun main() -> void { unknown_var = 5; }".to_string(), // undeclared variable
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }

  #[test]
  fn test_sema_intrinsics_is_int_is_float() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: "fun main() -> bool { let a: bool = sys::is_int<i32>(); let b: bool = sys::is_float<f32>(); ret a; }".to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    for m in sum.msgs() { println!("{}", m); }
    assert_eq!(sum.sumerr(), 0);
  }

  #[test]
  fn test_sema_intrinsics_missing_type_arg_err() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: "fun main() -> bool { let a: bool = sys::is_int(42); ret a; }".to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }

  #[test]
  fn test_sema_intrinsics_with_args_err() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: "fun main() -> bool { let a: bool = sys::is_int<i32>(42); ret a; }".to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }

  #[test]
  fn test_sema_all_type_query_intrinsics() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"fun main() -> bool {
        let c1: bool = sys::is_int<i32>();
        let c2: bool = sys::is_float<f64>();
        let c3: bool = sys::is_bool<bool>();
        let c4: bool = sys::is_char<char>();
        let c5: bool = sys::is_ptr<ptr>();
        let c6: bool = sys::is_void<void>();
        let c7: bool = sys::is_null<null_t>();
        let c8: bool = sys::is_signed<i64>();
        let c9: bool = sys::is_unsigned<u32>();
        ret c1;
      }"#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    for m in sum.msgs() { println!("{}", m); }
    assert_eq!(sum.sumerr(), 0);
  }

  #[test]
  fn test_sema_is_extended_intrinsic() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"
        struct Foo {}
        trait Bar { fun bar() -> void; }
        extend Foo : Bar { fun bar() -> void {} }

        fun main() -> bool {
          let check: bool = sys::is_extended<Foo, Bar>();
          ret check;
        }
      "#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    for m in sum.msgs() { println!("{}", m); }
    assert_eq!(sum.sumerr(), 0);
  }

  
  #[test]
  fn test_sema_assignment_type_equality_pass() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"
        fun main() -> void {
          let a: bool = true;
          var b: bool = false;
          b = a;
        }
      "#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    for m in sum.msgs() { println!("{}", m); }
    assert_eq!(sum.sumerr(), 0);
  }

  #[test]
  fn test_sema_assignment_type_mismatch_fail() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"
        fun main() -> void {
          let a: bool = true;
          var b: i32 = 0;
          b = a;
        }
      "#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }

  
  #[test]
  fn test_sema_return_type_pass() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"
        fun get_flag() -> bool {
          ret true;
        }
      "#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    for m in sum.msgs() { println!("{}", m); }
    assert_eq!(sum.sumerr(), 0);
  }

  #[test]
  fn test_sema_return_type_mismatch_fail() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"
        fun get_val() -> i32 {
          ret true;
        }
      "#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }

  #[test]
  fn test_sema_invalid_index_target_fail() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"
        fun main() -> void {
          let x: i32 = 10;
          let y: i32 = x[0];
        }
      "#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }

  #[test]
  fn test_sema_array_index_out_of_bounds_fail() {
    let sys = SysFile::new().unwrap();
    let farena = FileArena::new();
    let mi = ModInjection { sys: &sys, farena: &farena };

    let mut mol = Module::new_rtl("test".to_string()).unwrap();
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: r#"
        fun main() -> void {
          let a: [u32, 8];
          let b: u32 = a[10];
        }
      "#.to_string(),
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }
}
