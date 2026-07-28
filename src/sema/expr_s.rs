use crate::diagnostic::{Message, MsgKind};
use super::Sema;
use crate::control::identy::IdentyId;
use crate::ast::ExprVari;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_expr(&mut self, id: IdentyId) -> Result<IdentyId, Message<'a>> {
    self.check_attributes(id)?;
    let mut computed_ty = self.get_ty_void();

    enum Extracted<'a> {
      Block(Vec<IdentyId>),
      If(IdentyId, IdentyId, Option<IdentyId>),
      Match(IdentyId, Vec<(IdentyId, IdentyId)>),
      Binary(IdentyId, IdentyId),
      Unary(IdentyId),
      Nick(u32, crate::lexer::Word<'a>),
      Number(crate::lexer::Word<'a>),
    }

    let extracted = {
      let expr = self.mol.get_expr(id);
      match &expr.vari {
        ExprVari::Block(b) => Extracted::Block(b.ctn.clone()),
        ExprVari::If(i) => Extracted::If(i.cond, i.then_block, i.else_block),
        ExprVari::Match(m) => Extracted::Match(m.val, m.arms.iter().map(|a| (a.pat, a.body)).collect()),
        ExprVari::Binary(b) => Extracted::Binary(b.lhs, b.rhs),
        ExprVari::Unary(u) => Extracted::Unary(u.val),
        ExprVari::Nick(n) => Extracted::Nick(n.idx, n.pos),
        ExprVari::Number(n) => Extracted::Number(n.pos),
      }
    };

    
    match extracted {
      Extracted::Block(ctn) => {
        for s_id in &ctn {
          self.check_stmt(*s_id)?;
        }
        computed_ty = self.get_ty_void();
      }
      Extracted::If(cond, then_block, else_block) => {
        let cond_ty = self.check_expr(cond)?;
        let ty_bool = self.get_ty_bool();
        if cond_ty.index() != ty_bool.index() {}
        let then_ty = self.check_expr(then_block)?;
        if let Some(eb) = else_block {
          let _else_ty = self.check_expr(eb)?;
          computed_ty = then_ty;
        } else {
          computed_ty = self.get_ty_void();
        }
      }
      Extracted::Match(val, arms) => {
        let val_ty = self.check_expr(val)?;
        let mut first_body_ty = None;
        for (pat, body) in arms {
          let is_default = {
            let p_expr = self.mol.get_expr(pat);
            if let ExprVari::Nick(n) = &p_expr.vari {
              self.mol.nick_map[n.idx as usize] == "_"
            } else {
              false
            }
          };
          
          if !is_default {
            let pat_ty = self.check_expr(pat)?;
            if pat_ty.index() != val_ty.index() {}
          }
          let body_ty = self.check_expr(body)?;
          if first_body_ty.is_none() {
            first_body_ty = Some(body_ty);
          }
        }
        computed_ty = first_body_ty.unwrap_or_else(|| self.get_ty_void());
      }
      Extracted::Binary(lhs, rhs) => {
        self.check_expr(lhs)?;
        self.check_expr(rhs)?;
      }
      Extracted::Unary(val) => {
        self.check_expr(val)?;
      }
      Extracted::Nick(idx, pos) => {
        let name = self.mol.nick_map[idx as usize].clone();
        if let Some(var_ty) = self.find_var_global(&name) {
           computed_ty = var_ty; 
        } else {
           return Err(Message::new(MsgKind::Error, pos, "unknown variable".to_string(), vec![name]));
        }
      }
      Extracted::Number(_pos) => {
        if let Some(ty) = self.find_type_global("i32") {
          computed_ty = ty;
        }
      }
    }

    self.mol.get_mut_expr(id).ty = computed_ty;
    Ok(computed_ty)
  }

}
