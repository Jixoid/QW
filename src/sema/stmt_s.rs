use crate::diagnostic::Message;
use super::Sema;
use crate::control::identy::AstId;
use crate::ast::StmtVari;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_stmt(&mut self, id: AstId) -> Result<(), Message<'a>> {
    self.check_attributes(id)?;
    enum ExtractedStmt<'a> {
      Let(crate::lexer::Word<'a>, AstId, Option<AstId>),
      Ret(AstId),
      Expr(AstId),
    }

    let extracted = {
      let stmt = self.mol.get_stmt(id);
      match &stmt.vari {
        StmtVari::Let(l) => ExtractedStmt::Let(l.name, l.kind, l.init),
        StmtVari::Ret(r) => ExtractedStmt::Ret(r.val),
        StmtVari::Expr(e) => ExtractedStmt::Expr(e.expr),
      }
    };

    match extracted {
      ExtractedStmt::Let(name, kind, init) => {
        let mut resolved_type = kind;

        if kind.kind() != crate::control::IdentyKind::Null {
          self.check_type(kind)?;
        }
        if let Some(expr_id) = init {
          let expr_ty = self.check_expr(expr_id)?;
          if kind.kind() == crate::control::IdentyKind::Null {
            resolved_type = expr_ty;
          } else {
            if !self.is_type_equal(kind, expr_ty, None) {
              return Err(Message::error(name, format!("type mismatch in variable initialization of `{}`", name.str()), vec![]));
            }
          }
        }
        
        if let StmtVari::Let(ml) = &mut self.mol.get_mut_stmt(id).vari {
          ml.kind = resolved_type;
        }
      },
      ExtractedStmt::Ret(val) => {
        let val_ty = self.check_expr(val)?;
        if let Some(expected_ret_ty) = self.current_fn_ret_type() {
          if !self.is_type_equal(expected_ret_ty, val_ty, None) {
            let pos_opt = match &self.mol.get_expr(val).vari {
              crate::ast::ExprVari::Nick(n) => Some(n.pos),
              crate::ast::ExprVari::Path(p) => p.first().map(|n| n.pos),
              crate::ast::ExprVari::Number(n) => Some(n.pos),
              _ => None,
            };
            if let Some(pos) = pos_opt {
              return Err(Message::error(pos, "return type mismatch: returned value does not match function return type".to_string(), vec![]));
            } else {
              let fn_id = self.visitors.iter().rev().find(|&&id| id.kind() == crate::control::IdentyKind::Decl).cloned();
              if let Some(fn_id) = fn_id {
                if let Some(pos) = self.mol.get_decl(fn_id).name.pos().cloned() {
                  return Err(Message::error(pos, "return type mismatch: returned value does not match function return type".to_string(), vec![]));
                }
              }
            }
          }
        }
      },
      ExtractedStmt::Expr(expr) => {
        self.check_expr(expr)?;
      }
    }

    Ok(())
  }

}
