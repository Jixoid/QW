use crate::diagnostic::{Message, MsgKind};
use crate::sema::scopemng::ScopeManager;
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
      Path(Vec<(u32, crate::lexer::Word<'a>)>),
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
        ExprVari::Path(p) => Extracted::Path(p.iter().map(|n| (n.idx, n.pos)).collect()),
        ExprVari::Number(n) => Extracted::Number(n.pos),
      }
    };

    
    match extracted {
      Extracted::Block(ctn) => {
        self.visitors.push(id);
        for s_id in &ctn {
          self.check_stmt(*s_id)?;
        }
        self.visitors.pop();
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
        if let Some(did) = ScopeManager::find_local(self.mol, &self.visitors, &name) {
          let stmt = self.mol.get_stmt(did);
          if let crate::ast::StmtVari::Let(l) = &stmt.vari {
            computed_ty = l.kind;
          } else {
            return Err(Message::new(MsgKind::Error, pos, "expected variable".to_string(), vec![name]));
          }
        } else if let Some(did) = ScopeManager::find(self.mol, self.mol.get_mod(), &[name.clone()]) {
          let decl = self.mol.get_decl(did);
          if let crate::ast::DeclVari::Var(v) = &decl.vari {
            computed_ty = v.kind;
          } else if let crate::ast::DeclVari::Fun(f) = &decl.vari {
            computed_ty = f.kind; // Use the function's type
          } else {
            return Err(Message::new(MsgKind::Error, pos, "expected variable or function".to_string(), vec![name]));
          }
        } else {
          return Err(Message::new(MsgKind::Error, pos, "unknown variable".to_string(), vec![name]));
        }
      }
      
      Extracted::Path(path) => {
        let path_strs: Vec<String> = path.iter().map(|(idx, _)| self.mol.nick_map[*idx as usize].clone()).collect();
        if let Some(ty_id) = ScopeManager::find(self.mol, self.mol.get_mod(), &path_strs) {
          // If ScopeManager resolved this, it might be an Enum Variant's type id or just a direct type id
          computed_ty = ty_id;
        } else {
          let err_str = path_strs.join("::");
          return Err(Message::new(MsgKind::Error, path[0].1, format!("unknown path `{}`", err_str), vec![]));
        }
      }

      Extracted::Number(_pos) => {
        if let Some(ty) = ScopeManager::find(self.mol, self.mol.get_mod(), &["sys".to_string(), "i32".to_string()]) {
          computed_ty = ty;
        }
      }
    
    }

    self.mol.get_mut_expr(id).ty = computed_ty;
    Ok(computed_ty)
  }

}
