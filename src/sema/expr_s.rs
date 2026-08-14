use crate::diagnostic::{Message, MsgKind};
use crate::sema::scopemng::ScopeManager;
use super::Sema;
use crate::control::identy::AstId;
use crate::ast::ExprVari;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_expr(&mut self, id: AstId) -> Result<AstId, Message> {
    self.check_attributes(id)?;
    let mut computed_ty = self.get_ty_void();

    enum Extracted<'a> {
      Block(Vec<AstId>),
      If(AstId, AstId, Option<AstId>),
      Match(AstId, Vec<(AstId, AstId)>),
      Binary(crate::lexer::WordKind, AstId, AstId),
      Unary(AstId),
      Nick(u32, crate::lexer::Word<'a>),
      Path(Vec<(u32, crate::lexer::Word<'a>)>),
      Number(crate::lexer::Word<'a>),
      Call(AstId, Vec<AstId>, Vec<AstId>),
      Index(AstId, AstId),
    }

    let extracted = {
      let expr = self.mol.get_expr(id);
      match &expr.vari {
        ExprVari::Block(b) => Extracted::Block(b.ctn.clone()),
        ExprVari::If(i) => Extracted::If(i.cond, i.then_block, i.else_block),
        ExprVari::Match(m) => Extracted::Match(m.val, m.arms.iter().map(|a| (a.pat, a.body)).collect()),
        ExprVari::Binary(b) => Extracted::Binary(b.op, b.lhs, b.rhs),
        ExprVari::Unary(u) => Extracted::Unary(u.val),
        ExprVari::Nick(n) => Extracted::Nick(n.idx, n.pos),
        ExprVari::Path(p) => Extracted::Path(p.iter().map(|n| (n.idx, n.pos)).collect()),
        ExprVari::Number(n) => Extracted::Number(n.pos),
        ExprVari::Call(c) => Extracted::Call(c.callee, c.args.clone(), c.generic_args.clone()),
        ExprVari::Index(idx) => Extracted::Index(idx.target, idx.index),
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
      
      Extracted::Binary(op, lhs, rhs) => {
        let lhs_ty = self.check_expr(lhs)?;
        let rhs_ty = self.check_expr(rhs)?;

        if matches!(
          op,
          crate::lexer::WordKind::Assign |
          crate::lexer::WordKind::AssignmentAdd |
          crate::lexer::WordKind::AssignmentSub |
          crate::lexer::WordKind::AssignmentMul |
          crate::lexer::WordKind::AssignmentDiv |
          crate::lexer::WordKind::AssignmentRem |
          crate::lexer::WordKind::AssignmentBitwiseAnd |
          crate::lexer::WordKind::AssignmentBitwiseOr |
          crate::lexer::WordKind::AssignmentBitwiseXor |
          crate::lexer::WordKind::AssignmentLeftShift |
          crate::lexer::WordKind::AssignmentRighShift |
          crate::lexer::WordKind::AssignmentLogicalAnd |
          crate::lexer::WordKind::AssignmentLogicalOr |
          crate::lexer::WordKind::AssignmentLogicalXor
        ) {
          if !self.is_type_equal(lhs_ty, rhs_ty, None) {
            let pos_opt = match &self.mol.get_expr(lhs).vari {
              ExprVari::Nick(n) => Some(n.pos),
              ExprVari::Path(p) => p.first().map(|n| n.pos),
              _ => match &self.mol.get_expr(rhs).vari {
                ExprVari::Nick(n) => Some(n.pos),
                ExprVari::Path(p) => p.first().map(|n| n.pos),
                _ => None,
              },
            };
            if let Some(pos) = pos_opt {
              return Err(Message::error(pos, "type mismatch in assignment: types must be equal".to_string(), vec![]));
            }
          }
        }

        computed_ty = lhs_ty;
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
            let mut k = v.kind;
            if k.module() == 0 && did.module() != 0 {
              k = crate::control::identy::AstId::new(k.kind(), did.module(), k.index());
            }
            computed_ty = k;
          } else if let crate::ast::DeclVari::Fun(f) = &decl.vari {
            let mut k = f.kind;
            if k.module() == 0 && did.module() != 0 {
              k = crate::control::identy::AstId::new(k.kind(), did.module(), k.index());
            }
            computed_ty = k;
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

      Extracted::Call(callee, args, generic_args) => {
        let callee_ty = self.check_expr(callee)?;
        for &a in &args {
          self.check_expr(a)?;
        }
        for &g in &generic_args {
          self.check_type(g)?;
        }

        let is_intrinsic_name = |name: &str| -> bool {
          matches!(
            name,
            "is_int" | "is_float" | "is_bool" | "is_char" | "is_ptr" | "is_pointer" |
            "is_reference" | "is_void" | "is_null" | "is_signed" | "is_unsigned" |
            "is_array" | "is_struct" | "is_enum" | "is_flags" | "is_function" |
            "is_trait" | "is_iface" | "is_extended"
          )
        };

        let (is_intrinsic_bool, intrinsic_name, pos_opt) = {
          let c_expr = self.mol.get_expr(callee);
          match &c_expr.vari {
            ExprVari::Nick(n) => {
              let name = &self.mol.nick_map[n.idx as usize];
              (is_intrinsic_name(name), Some(name.clone()), Some(n.pos))
            }
            ExprVari::Path(p) => {
              let path_strs: Vec<String> = p.iter().map(|n| self.mol.nick_map[n.idx as usize].clone()).collect();
              let name = if path_strs.len() == 2 && path_strs[0] == "sys" {
                Some(path_strs[1].clone())
              } else if path_strs.len() == 1 {
                Some(path_strs[0].clone())
              } else {
                None
              };
              let is_match = name.as_deref().map_or(false, is_intrinsic_name);
              (is_match, name, p.first().map(|n| n.pos))
            }
            _ => (false, None, None),
          }
        };

        if is_intrinsic_bool {
          let req_gen_count = if intrinsic_name.as_deref() == Some("is_extended") { 2 } else { 1 };
          if generic_args.len() != req_gen_count {
            if let Some(pos) = pos_opt {
              let err_msg = if req_gen_count == 2 {
                "intrinsic sys::is_extended requires 2 generic type arguments `<Base, Trait>`".to_string()
              } else {
                "intrinsic function requires exactly 1 generic type argument `<T>`".to_string()
              };
              return Err(Message::error(pos, err_msg, vec![]));
            }
          }
          if !args.is_empty() {
            if let Some(pos) = pos_opt {
              return Err(Message::error(pos, "intrinsic function does not accept arguments".to_string(), vec![]));
            }
          }
          computed_ty = self.get_ty_bool();
        } else {
          let callee_ty_val = self.mol.get_type(callee_ty);
          if let crate::ast::TypeVari::Function(f) = &callee_ty_val.vari {
            computed_ty = f.ret;
          } else {
            computed_ty = self.get_ty_void();
          }
        }
      }

      Extracted::Index(target, index) => {
        let target_ty_id = self.check_expr(target)?;
        let _index_ty_id = self.check_expr(index)?;
        let target_ty = self.mol.get_type(target_ty_id);
        if let crate::ast::TypeVari::ArrayOf { sub, ext } = &target_ty.vari {
          let sub_ty = *sub;
          if !ext.is_empty() {
            let total_capacity: u64 = ext.iter().fold(1u64, |acc, &l| acc.saturating_mul(l as u64));
            let index_expr = self.mol.get_expr(index);
            if let ExprVari::Number(num) = &index_expr.vari {
              if let Ok(idx_val) = num.pos.str().parse::<u64>() {
                if idx_val >= total_capacity {
                  return Err(Message::error(num.pos, format!("array index out of bounds: the length is {} but the index is {}", total_capacity, idx_val), vec![]));
                }
              }
            }
          }
          computed_ty = sub_ty;
        } else {
          let pos = match &self.mol.get_expr(target).vari {
            ExprVari::Nick(n) => n.pos,
            ExprVari::Number(n) => n.pos,
            ExprVari::Path(p) if !p.is_empty() => p[0].pos,
            _ => {
              let pos_opt = self.mol.list_type.iter().find_map(|t| {
                if let crate::ast::TypeVari::Nick(n) = &t.vari { Some(n.pos) } else { None }
              }).expect("Cannot find position word for error message");
              return Err(Message::error(pos_opt, "type is not indexable".to_string(), vec![]));
            }
          };
          return Err(Message::error(pos, "type is not indexable".to_string(), vec![]));
        }
      }
    
    }

    self.mol.get_mut_expr(id).ty = computed_ty;
    Ok(computed_ty)
  }

}
