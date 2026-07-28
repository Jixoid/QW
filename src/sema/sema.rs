use crate::{control::Module, diagnostic::Summary};



pub struct Sema<'f, 'a, 'd:'a> {
  pub mol: &'f mut Module<'a,'d>,
  pub sum: Summary<'a>,
  pub scp: super::scopemng::ScopeManager,
}


impl<'f,'a,'d> Sema<'f,'a,'d> {

  pub fn new(mol: &'f mut Module<'a,'d>) -> Self {
    let sum = Summary::new();
    let scp = super::scopemng::ScopeManager::new();

    Self{mol, sum, scp}
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
            if v != "bare" && v != "itanium" {
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
