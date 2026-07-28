use crate::diagnostic::Message;
use super::Sema;
use crate::control::identy::IdentyId;
use crate::ast::DeclVari;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_decl(&mut self, id: IdentyId) -> Result<(), Message<'a>> {
    self.check_attributes(id)?;
    let decl = self.mol.get_decl(id);
    let vari = &decl.vari;
    let name_str = decl.name.to_string();

    match vari {
      DeclVari::Using(ty_id) => {
        let ty_id = *ty_id;
        self.scp.add_type(name_str, ty_id);
        self.check_type(ty_id)?;
      }
      DeclVari::Module(m) => {
        let decls = m.decls.clone();
        for did in decls {
          self.check_decl(did)?;
        }
      }
      DeclVari::Fun(f) => {
        let kind = f.kind;
        let blok = f.blok;
        self.check_type(kind)?;
        self.check_expr(blok)?;
      }
      DeclVari::Var(v) => {
        let kind = v.kind;
        let init = v.init;
        self.check_type(kind)?;
        if let Some(i) = init {
          self.check_expr(i)?;
        }
      }
    }

    Ok(())
  }

}
