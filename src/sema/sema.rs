use crate::{control::{IdentyId, Module}, diagnostic::Summary};



pub struct Sema<'f, 'a, 'd:'a> {
  pub mol: &'f mut Module<'a,'d>,
  pub sum: Summary<'a>,
  pub visitors: Vec<IdentyId>,
}


impl<'f,'a,'d> Sema<'f,'a,'d> {

  pub fn new(mol: &'f mut Module<'a,'d>) -> Self {
    let sum = Summary::new();

    Self{mol, sum, visitors: Vec::new()}
  }


  pub fn check(&mut self) -> Summary<'a> {
    if !self.mol.list_decl.is_empty() {
      let root_id = crate::control::IdentyId::new(crate::control::IdentyKind::Decl, 0, 0);
      if let Err(e) = self.check_decl(root_id) {
        self.sum.add(e);
      }
    }

    self.sum.clone()
  }

  pub fn check_attributes(&mut self, id: crate::control::identy::IdentyId) -> Result<(), crate::diagnostic::Message<'a>> {
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
      mmap: "fun main() -> sys::void {}".to_string(),
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
      mmap: "fun main() -> sys::void { unknown_var = 5; }".to_string(), // undeclared variable
      kind: ModuleKind::Regular,
    };
    
    mol.add_dep(&sys.mol);
    let mut front = Front::new(&mut mol, &mfd).unwrap();
    front.parse(&mi);
    
    let sum = Sema::new(&mut mol).check();
    assert!(sum.sumerr() > 0);
  }

}
