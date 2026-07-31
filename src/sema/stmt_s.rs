use crate::diagnostic::Message;
use super::Sema;
use crate::control::identy::IdentyId;
use crate::ast::StmtVari;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_stmt(&mut self, id: IdentyId) -> Result<(), Message<'a>> {
    self.check_attributes(id)?;
    let vari = &self.mol.get_stmt(id).vari;

    match vari {
      StmtVari::Let(l) => {
        let kind = l.kind;
        let init = l.init;
        
        let mut resolved_type = kind;

        if kind.kind() != crate::control::IdentyKind::Null {
          self.check_type(kind)?;
        }
        if let Some(expr_id) = init {
          let expr_ty = self.check_expr(expr_id)?;
          if kind.kind() == crate::control::IdentyKind::Null {
            resolved_type = expr_ty;
          } else {

          }
        }
        
        if let StmtVari::Let(ml) = &mut self.mol.get_mut_stmt(id).vari {
          ml.kind = resolved_type;
        }
      },
      StmtVari::Ret(r) => {
        let val = r.val;
        self.check_expr(val)?;
      },
      StmtVari::Expr(e) => {
        let expr = e.expr;
        self.check_expr(expr)?;
      }
    }

    Ok(())
  }

}
