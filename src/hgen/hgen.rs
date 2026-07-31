use crate::control::{Module, identy::{IdentyId, IdentyKind}};
use crate::hir::module::HirModule;
use crate::ast::decls::{Decl, DeclVari};
use crate::hir::global::HirGlobalVar;
use std::collections::HashMap;
use crate::hgen::mangle::{Mangler, ManglerKind};

pub struct HGen<'a,'d> {
  pub ast_mol: &'a Module<'a,'d>,
  pub hir_mol: HirModule,

  pub mangler: ManglerKind,
  pub is_debug: bool,

  pub map_decl: HashMap<IdentyId, crate::hir::identy::HirId>,
  pub map_type: HashMap<IdentyId, crate::hir::identy::HirId>,
  pub local_scope: HashMap<String, crate::hir::identy::HirId>,
  pub parent_path: HashMap<IdentyId, Vec<String>>,
  pub parent_decl: HashMap<IdentyId, IdentyId>,
}

impl<'a,'d> HGen<'a,'d> {

  pub fn get_attr_val(&self, id: IdentyId, name: &str) -> Option<String> {
    if let Some(attrs) = self.ast_mol.map_attr.get(&id) {
      for attr in attrs {
        if attr.key.str() == name {
          if let Some(val) = &attr.val {
            return Some(val.str().to_string());
          }
        }
      }
    }
    None
  }

  pub fn has_attr(&self, id: IdentyId, name: &str) -> bool {
    if let Some(attrs) = self.ast_mol.map_attr.get(&id) {
      for attr in attrs {
        if attr.key.str() == name {
          return true;
        }
      }
    }
    false
  }

  
  pub fn new(ast_mol: &'a Module<'a,'d>, is_debug: bool) -> Self {
    Self {
      ast_mol,
      hir_mol: HirModule::new(ast_mol.name.clone()),
      mangler: ManglerKind::Qw,
      is_debug,
      map_decl: HashMap::new(),
      map_type: HashMap::new(),
      local_scope: HashMap::new(),
      parent_path: HashMap::new(),
      parent_decl: HashMap::new(),
    }
  }

  pub fn generate(mut self) -> HirModule {
    let root_id = IdentyId::new(IdentyKind::Decl, 0, 0);
    self.build_path_map(root_id, vec![]);
    
    self.gen_module();
    self.hir_mol
  }

  
  fn build_path_map(&mut self, decl_id: IdentyId, mut current_path: Vec<String>) {
    let decl = self.ast_mol.get_decl(decl_id);
    let name_str = decl.name.to_string();
    
    current_path.push(name_str.clone());
    self.parent_path.insert(decl_id, current_path.clone());

    match &decl.vari {
      DeclVari::Module(m) => {
        for child_id in &m.decls {
          self.parent_decl.insert(*child_id, decl_id);
          self.build_path_map(*child_id, current_path.clone());
        }
      }
      DeclVari::Using(ty_id) => {
        let ty = self.ast_mol.get_type(*ty_id);
        if let crate::ast::TypeVari::Struct(s) = &ty.vari {
          for fun_id in &s.funs {
            self.parent_decl.insert(*fun_id, decl_id);
            self.build_path_map(*fun_id, current_path.clone());
          }
        }
      }
      _ => {}
    }
  }

  fn gen_module(&mut self) {
    // Global Variables
    for (i,x) in self.ast_mol.list_decl.iter().enumerate() {
      if let crate::ast::DeclVari::Var(_) = &x.vari {
        let ast_id = IdentyId::new(IdentyKind::Decl, 0, i as u32);
        self.gen_decl(ast_id, x);
      }
    }
    
    // Functions and others
    for (i,x) in self.ast_mol.list_decl.iter().enumerate() {
      if let crate::ast::DeclVari::Var(_) = &x.vari { continue; }
      let ast_id = IdentyId::new(IdentyKind::Decl, 0, i as u32);
      self.gen_decl(ast_id, x);
    }
  }

  fn gen_decl(&mut self, ast_id: IdentyId, decl: &Decl) {
    if self.map_decl.contains_key(&ast_id) { return; }

    match &decl.vari {
      DeclVari::Var(v) => {
        let ty_id = self.gen_type(v.kind);
        
        let init_val = None; // TODO: v.init için gen_expr eklenecek
        
        let is_const = match v.acck {
          crate::ast::types::AccessKind::IMM => true,
          crate::ast::types::AccessKind::MUT => false,
        };
        
        let decl_name_str = decl.name.to_string();
        
        let path = self.parent_path.get(&ast_id).cloned().unwrap_or_else(|| vec![self.ast_mol.name.clone()]);
        let parent_path = if path.len() > 1 { &path[0..path.len()-1] } else { &path[..] };
        
        let mut current_mangler = self.mangler;

        if let Some(parent_id) = self.parent_decl.get(&ast_id) {
          if let Some(val) = self.get_attr_val(*parent_id, "mangle") {
            match val.as_str() {
              "bare" => current_mangler = ManglerKind::Bare,
              "itanium" => current_mangler = ManglerKind::Itanium,
              "qw" => current_mangler = ManglerKind::Qw,
              _ => {}
            }
          }
        }

        let is_weak = self.has_attr(ast_id, "weak");

        if let Some(val) = self.get_attr_val(ast_id, "mangle") {
          match val.as_str() {
            "bare" => current_mangler = ManglerKind::Bare,
            "itanium" => current_mangler = ManglerKind::Itanium,
            "qw" => current_mangler = ManglerKind::Qw,
            _ => {}
          }
        }
        
        let final_name = current_mangler.mangle_global(parent_path, &decl_name_str);

        let g = HirGlobalVar::new(final_name, ty_id, init_val, is_const, is_weak);
        let hid = self.hir_mol.new_global(g);
        self.map_decl.insert(ast_id, hid);
      }
      DeclVari::Using(u) => {
        self.gen_type(*u);
      }
      DeclVari::Fun(f) => {
        let ty_id = self.gen_type(f.kind);
        
        let decl_name_str = decl.name.to_string();
        
        let path = self.parent_path.get(&ast_id).cloned().unwrap_or_else(|| vec![self.ast_mol.name.clone()]);
        let parent_path = if path.len() > 1 { &path[0..path.len()-1] } else { &path[..] };
        
        let is_instance_method = {
          let ast_ty = self.ast_mol.get_type(f.kind);
          if let crate::ast::types::TypeVari::Function(ast_fun) = &ast_ty.vari {
            if !ast_fun.is_static {
               if let Some(parent_id) = self.parent_decl.get(&ast_id) {
                 let parent_decl = self.ast_mol.get_decl(*parent_id);
                 matches!(parent_decl.vari, crate::ast::decls::DeclVari::Using(_))
               } else { false }
            } else { false }
          } else { false }
        };

        let (ret_ty, self_ty, arg_tys) = {
          let hir_ty = self.hir_mol.get_type(ty_id);
          if let crate::hir::types::HirTypeVari::Function(fun) = &hir_ty.vari {
            let mut args = fun.args.iter();
            let mut self_ty = None;
            let mut arg_tys = Vec::new();
            if is_instance_method {
              if let Some(first_arg) = args.next() {
                self_ty = Some(first_arg.kind);
              }
            }
            arg_tys.extend(args.map(|a| a.kind));
            (fun.ret, self_ty, arg_tys)
          } else {
            unreachable!("FunDecl kind must be Function type");
          }
        };

        let mut current_mangler = self.mangler;

        if let Some(parent_id) = self.parent_decl.get(&ast_id) {
          if let Some(val) = self.get_attr_val(*parent_id, "mangle") {
            match val.as_str() {
              "bare" => current_mangler = ManglerKind::Bare,
              "itanium" => current_mangler = ManglerKind::Itanium,
              "qw" => current_mangler = ManglerKind::Qw,
              _ => {}
            }
          }
        }

        let is_weak = self.has_attr(ast_id, "weak");

        if let Some(val) = self.get_attr_val(ast_id, "mangle") {
          match val.as_str() {
            "bare" => current_mangler = ManglerKind::Bare,
            "itanium" => current_mangler = ManglerKind::Itanium,
            "qw" => current_mangler = ManglerKind::Qw,
            _ => {}
          }
        }
        
        let final_name = current_mangler.mangle_func(parent_path, &decl_name_str, self_ty, Some(ret_ty), &arg_tys, &self.hir_mol);

        let h_func = crate::hir::func::HirFunc::new(final_name, ret_ty, arg_tys, is_weak);
        let func_hid = self.hir_mol.new_func(h_func.clone());
        self.map_decl.insert(ast_id, func_hid);


        self.local_scope.clear();


        let entry_block = crate::hir::block::HirBlock::new("entry".to_string());
        let mut block_hid = self.hir_mol.new_block(entry_block);
        
        let h_func_mut = self.hir_mol.get_func_mut(func_hid);
        h_func_mut.push_block(block_hid);

        let block_expr = self.ast_mol.get_expr(f.blok);
        if let crate::ast::ExprVari::Block(b) = &block_expr.vari {
          for stmt_id in &b.ctn {
            let stmt = self.ast_mol.get_stmt(*stmt_id);
            self.gen_stmt(stmt, func_hid, &mut block_hid);
          }
        }
      }
      DeclVari::Module(_) | DeclVari::Import(_, _) | DeclVari::ImportWildcard(_, _) => {},
    }
  }

  fn gen_type(&mut self, ast_id: IdentyId) -> crate::hir::identy::HirId {
    if let Some(&hid) = self.map_type.get(&ast_id) {
      return hid;
    }

    let ty = self.ast_mol.get_type(ast_id);
    if let crate::ast::types::TypeVari::Path(path) = &ty.vari {
      if path.len() > 0 {
        let resolved_id = path.last().unwrap();
        let hid = self.gen_type(*resolved_id);
        self.map_type.insert(ast_id, hid);
        return hid;
      }
    }

    let hid = self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Null });
    self.map_type.insert(ast_id, hid);
    
    let hir_ty_vari = match &ty.vari {
      crate::ast::types::TypeVari::ISize => crate::hir::types::HirTypeVari::ISize,
      crate::ast::types::TypeVari::USize => crate::hir::types::HirTypeVari::USize,
      crate::ast::types::TypeVari::I8 => crate::hir::types::HirTypeVari::I8,
      crate::ast::types::TypeVari::I16 => crate::hir::types::HirTypeVari::I16,
      crate::ast::types::TypeVari::I32 => crate::hir::types::HirTypeVari::I32,
      crate::ast::types::TypeVari::I64 => crate::hir::types::HirTypeVari::I64,
      crate::ast::types::TypeVari::I128 => crate::hir::types::HirTypeVari::I128,
      crate::ast::types::TypeVari::U8 => crate::hir::types::HirTypeVari::U8,
      crate::ast::types::TypeVari::U16 => crate::hir::types::HirTypeVari::U16,
      crate::ast::types::TypeVari::U32 => crate::hir::types::HirTypeVari::U32,
      crate::ast::types::TypeVari::U64 => crate::hir::types::HirTypeVari::U64,
      crate::ast::types::TypeVari::U128 => crate::hir::types::HirTypeVari::U128,
      crate::ast::types::TypeVari::F16 => crate::hir::types::HirTypeVari::F16,
      crate::ast::types::TypeVari::F32 => crate::hir::types::HirTypeVari::F32,
      crate::ast::types::TypeVari::F64 => crate::hir::types::HirTypeVari::F64,
      crate::ast::types::TypeVari::F128 => crate::hir::types::HirTypeVari::F128,
      crate::ast::types::TypeVari::Bool => crate::hir::types::HirTypeVari::Bool,
      crate::ast::types::TypeVari::Char => crate::hir::types::HirTypeVari::Char,
      crate::ast::types::TypeVari::Ptr => crate::hir::types::HirTypeVari::Ptr,
      crate::ast::types::TypeVari::Void => crate::hir::types::HirTypeVari::Void,
      crate::ast::types::TypeVari::Null => crate::hir::types::HirTypeVari::Null,
      
      crate::ast::types::TypeVari::Function(fun) => {
        let mut args = Vec::new();
        for arg in &fun.args {
          args.push(crate::hir::types::HirFieldType {
            name: arg.name.string(),
            kind: self.gen_type(arg.kind),
          });
        }
        crate::hir::types::HirTypeVari::Function(crate::hir::types::HirFunType {
          args,
          ret: self.gen_type(fun.ret),
        })
      }
      
      crate::ast::types::TypeVari::Struct(s) => {
        let mut vars = Vec::new();
        for f in &s.vars {
          let kind = self.gen_type(f.kind);
          vars.push(crate::hir::types::HirFieldType {
            name: f.name.string(),
            kind,
          });
        }
        crate::hir::types::HirTypeVari::Struct(crate::hir::types::HirStructType {
          base: Vec::new(),
          vars,
        })
      }
      
      crate::ast::types::TypeVari::Iface(i) => {
        let mut funs = Vec::new();
        for f in &i.funs {
          let kind = self.gen_type(f.kind);
          funs.push(crate::hir::types::HirFieldType {
            name: f.name.string(),
            kind,
          });
        }
        crate::hir::types::HirTypeVari::Iface(crate::hir::types::HirIfaceType {
          funs,
        })
      }
      
      crate::ast::types::TypeVari::Enum(e) => {
        let mut vals = Vec::new();
        for v in &e.vals {
          vals.push(crate::hir::types::HirFieldCons {
            val: match v.val {
              crate::ast::types::IntegerValue::SIG(i) => i as i128,
              crate::ast::types::IntegerValue::USG(u) => u as i128,
            },
            name: v.name.string(),
          });
        }
        crate::hir::types::HirTypeVari::Enum(crate::hir::types::HirEnumType {
          vals,
        })
      }
      
      crate::ast::types::TypeVari::Flags(e) => {
        let mut vals = Vec::new();
        for v in &e.vals {
          vals.push(crate::hir::types::HirFieldCons {
            val: match v.val {
              crate::ast::types::IntegerValue::SIG(i) => i as i128,
              crate::ast::types::IntegerValue::USG(u) => u as i128,
            },
            name: v.name.string(),
          });
        }
        crate::hir::types::HirTypeVari::Enum(crate::hir::types::HirEnumType {
          vals,
        })
      }
      
      crate::ast::types::TypeVari::ReferenceOf { sub, acc } => {
        let hir_sub = self.gen_type(*sub);
        crate::hir::types::HirTypeVari::ReferenceOf {
          sub: hir_sub,
          acc: *acc,
        }
      }
      
      crate::ast::types::TypeVari::PointerOf { sub, acc } => {
        let hir_sub = self.gen_type(*sub);
        crate::hir::types::HirTypeVari::PointerOf {
          sub: hir_sub,
          acc: *acc,
        }
      }
      
      _ => panic!("unimplemented type in hgen: {:?}", ty.vari),
    };

    self.hir_mol.list_type[hid.index() as usize].vari = hir_ty_vari;
    hid
  }

  fn gen_stmt(&mut self, stmt: &crate::ast::stmts::Stmt, func_hid: crate::hir::identy::HirId, current_block: &mut crate::hir::identy::HirId) {
    match &stmt.vari {
      crate::ast::stmts::StmtVari::Let(l) => {
        let ty_id = self.gen_type(l.kind);
        

        let alloca_instr = crate::hir::instr::HirInstr {
          vari: crate::hir::instr::HirInstrVari::Alloca(ty_id),
          ty: ty_id,
        };
        let alloca_hid = self.hir_mol.new_instr(alloca_instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(alloca_hid);

        self.local_scope.insert(l.name.string(), alloca_hid);

        if let Some(init_id) = l.init {
          let init_val = self.gen_expr(init_id, func_hid, current_block);
          let store_instr = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::Store(init_val, crate::hir::value::HirValue::Reg(alloca_hid)),
            ty: self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Void }),
          };
          let store_hid = self.hir_mol.new_instr(store_instr);
          self.hir_mol.get_block_mut(*current_block).push_instr(store_hid);
        }
      }
      crate::ast::stmts::StmtVari::Ret(r) => {
        let ret_val = self.gen_expr(r.val, func_hid, current_block);
        let ret_instr = crate::hir::instr::HirInstr {
          vari: crate::hir::instr::HirInstrVari::Ret(Some(ret_val)),
          ty: self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Void }),
        };
        let ret_hid = self.hir_mol.new_instr(ret_instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(ret_hid);
      }
      crate::ast::stmts::StmtVari::Expr(e) => {
        self.gen_expr(e.expr, func_hid, current_block);
      }
    }
  }

  fn gen_expr(&mut self, expr_id: IdentyId, func_hid: crate::hir::identy::HirId, current_block: &mut crate::hir::identy::HirId) -> crate::hir::value::HirValue {
    let expr = self.ast_mol.get_expr(expr_id);
    let ty_id = self.gen_type(expr.ty);

    match &expr.vari {
      crate::ast::ExprVari::Number(n) => {
        let num_str = n.pos.str();
        if let Ok(i) = num_str.parse::<i64>() {
          crate::hir::value::HirValue::ConstInt(i)
        } else if let Ok(f) = num_str.parse::<f64>() {
          crate::hir::value::HirValue::ConstFloat(f)
        } else {
          crate::hir::value::HirValue::ConstInt(0)
        }
      }

      crate::ast::ExprVari::Path(p) => {
        if p.len() == 2 {
          let variant_name = self.ast_mol.nick_map[p[1].idx as usize].clone();
          let ty = self.ast_mol.get_type(expr.ty);

          if let crate::ast::types::TypeVari::Enum(e) = &ty.vari {
            if let Some(val) = e.vals.iter().find(|v| v.name.str() == variant_name) {
              let val_int = match val.val {
                crate::ast::types::IntegerValue::SIG(i) => i,
                crate::ast::types::IntegerValue::USG(u) => u as i64,
              };
              return crate::hir::value::HirValue::ConstInt(val_int);
            }
          }
        }
        crate::hir::value::HirValue::ConstInt(0)
      }

      crate::ast::ExprVari::Nick(n) => {
        let name = self.ast_mol.nick_map[n.idx as usize].clone();

        if name == "true" { return crate::hir::value::HirValue::ConstBool(true); }
        if name == "false" { return crate::hir::value::HirValue::ConstBool(false); }
        if name == "is_debug" { return crate::hir::value::HirValue::ConstBool(self.is_debug); }
        if name == "null" { return crate::hir::value::HirValue::Null; }

        if let Some(&local_hid) = self.local_scope.get(&name) {

          let load_instr = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::Load(ty_id, crate::hir::value::HirValue::Reg(local_hid)),
            ty: ty_id,
          };
          let load_hid = self.hir_mol.new_instr(load_instr);
          self.hir_mol.get_block_mut(*current_block).push_instr(load_hid);
          crate::hir::value::HirValue::Reg(load_hid)
        } else {

          let mut global_ast_id = None;
          for (i, decl) in self.ast_mol.list_decl.iter().enumerate() {
            if decl.name.to_string() == name {
              global_ast_id = Some(crate::control::identy::IdentyId::new(crate::control::IdentyKind::Decl, 0, i as u32));
              break;
            }
          }
          if let Some(ast_id) = global_ast_id {
            if let Some(hid) = self.map_decl.get(&ast_id) {
              let decl = &self.ast_mol.list_decl[ast_id.index() as usize];
              if let crate::ast::DeclVari::Var(_) = &decl.vari {
                let global_var = self.hir_mol.get_global(*hid);
                let ty_id = global_var.ty;
                let load_instr = crate::hir::instr::HirInstr {
                  vari: crate::hir::instr::HirInstrVari::Load(ty_id, crate::hir::value::HirValue::Global(*hid)),
                  ty: ty_id,
                };
                let load_hid = self.hir_mol.new_instr(load_instr);
                self.hir_mol.get_block_mut(*current_block).push_instr(load_hid);
                return crate::hir::value::HirValue::Reg(load_hid);
              } else {
                return crate::hir::value::HirValue::Global(*hid);
              }
            }
          }
          
          panic!("Nick {} not found in local_scope or global decls. HIR requires Nicks to be fully resolved.", name);
        }
      }
      
      crate::ast::ExprVari::Block(b) => {
        let mut last_val = crate::hir::value::HirValue::Null;
        for (i, stmt_id) in b.ctn.iter().enumerate() {
          let stmt = self.ast_mol.get_stmt(*stmt_id);
          if i == b.ctn.len() - 1 {
            if let crate::ast::stmts::StmtVari::Expr(e) = &stmt.vari {
              last_val = self.gen_expr(e.expr, func_hid, current_block);
              continue;
            }
          }
          self.gen_stmt(stmt, func_hid, current_block);
        }
        last_val
      }
      
      crate::ast::ExprVari::If(i) => {
        let cond_val = self.gen_expr(i.cond, func_hid, current_block);

        let then_block = crate::hir::block::HirBlock::new("then".to_string());
        let then_hid = self.hir_mol.new_block(then_block);
        self.hir_mol.get_func_mut(func_hid).push_block(then_hid);

        let else_block = crate::hir::block::HirBlock::new("else".to_string());
        let else_hid = self.hir_mol.new_block(else_block);
        self.hir_mol.get_func_mut(func_hid).push_block(else_hid);

        let merge_block = crate::hir::block::HirBlock::new("merge".to_string());
        let merge_hid = self.hir_mol.new_block(merge_block);
        self.hir_mol.get_func_mut(func_hid).push_block(merge_hid);

        let cond_br_instr = crate::hir::instr::HirInstr {
          vari: crate::hir::instr::HirInstrVari::CondBr(cond_val, then_hid, else_hid),
          ty: self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Void }),
        };
        let cond_br_id = self.hir_mol.new_instr(cond_br_instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(cond_br_id);

        let is_void = matches!(self.hir_mol.list_type[ty_id.index() as usize].vari, crate::hir::types::HirTypeVari::Void);

        let alloca_id = if !is_void {
          let alloca_instr = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::Alloca(ty_id),
            ty: ty_id,
          };
          let aid = self.hir_mol.new_instr(alloca_instr);
          self.hir_mol.get_block_mut(*current_block).push_instr(aid);
          Some(aid)
        } else {
          None
        };

        *current_block = then_hid;
        let then_val = self.gen_expr(i.then_block, func_hid, current_block);
        if let Some(aid) = alloca_id {
          let store_then_instr = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::Store(then_val, crate::hir::value::HirValue::Reg(aid)),
            ty: self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Void }),
          };
          let store_then_id = self.hir_mol.new_instr(store_then_instr);
          self.hir_mol.get_block_mut(*current_block).push_instr(store_then_id);
        }
        
        let br_then_instr = crate::hir::instr::HirInstr {
          vari: crate::hir::instr::HirInstrVari::Br(merge_hid),
          ty: self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Void }),
        };
        let br_then_id = self.hir_mol.new_instr(br_then_instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(br_then_id);

        *current_block = else_hid;
        if let Some(eb) = i.else_block {
          let else_val = self.gen_expr(eb, func_hid, current_block);
          if let Some(aid) = alloca_id {
            let store_else_instr = crate::hir::instr::HirInstr {
              vari: crate::hir::instr::HirInstrVari::Store(else_val, crate::hir::value::HirValue::Reg(aid)),
              ty: self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Void }),
            };
            let store_else_id = self.hir_mol.new_instr(store_else_instr);
            self.hir_mol.get_block_mut(*current_block).push_instr(store_else_id);
          }
        }

        let br_else_instr = crate::hir::instr::HirInstr {
          vari: crate::hir::instr::HirInstrVari::Br(merge_hid),
          ty: self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Void }),
        };
        let br_else_id = self.hir_mol.new_instr(br_else_instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(br_else_id);

        *current_block = merge_hid;
        if let Some(aid) = alloca_id {
          let load_instr = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::Load(ty_id, crate::hir::value::HirValue::Reg(aid)),
            ty: ty_id,
          };
          let load_id = self.hir_mol.new_instr(load_instr);
          self.hir_mol.get_block_mut(*current_block).push_instr(load_id);
          crate::hir::value::HirValue::Reg(load_id)
        } else {
          crate::hir::value::HirValue::Null
        }
      }
      
      crate::ast::ExprVari::Binary(b) => {
        let lhs_val = self.gen_expr(b.lhs, func_hid, current_block);
        let rhs_val = self.gen_expr(b.rhs, func_hid, current_block);
        let vari = match b.op {
          crate::lexer::WordKind::Add => crate::hir::instr::HirInstrVari::Add(lhs_val, rhs_val),
          crate::lexer::WordKind::Sub => crate::hir::instr::HirInstrVari::Sub(lhs_val, rhs_val),
          crate::lexer::WordKind::Mul => crate::hir::instr::HirInstrVari::Mul(lhs_val, rhs_val),
          crate::lexer::WordKind::Div => crate::hir::instr::HirInstrVari::Div(lhs_val, rhs_val),
          crate::lexer::WordKind::Equal => crate::hir::instr::HirInstrVari::ICmp("eq".to_string(), lhs_val, rhs_val),
          crate::lexer::WordKind::NotEqual => crate::hir::instr::HirInstrVari::ICmp("ne".to_string(), lhs_val, rhs_val),
          crate::lexer::WordKind::AngleBeg => crate::hir::instr::HirInstrVari::ICmp("slt".to_string(), lhs_val, rhs_val),
          crate::lexer::WordKind::AngleEnd => crate::hir::instr::HirInstrVari::ICmp("sgt".to_string(), lhs_val, rhs_val),
          crate::lexer::WordKind::SmallerEqual => crate::hir::instr::HirInstrVari::ICmp("sle".to_string(), lhs_val, rhs_val),
          crate::lexer::WordKind::BiggerEqual => crate::hir::instr::HirInstrVari::ICmp("sge".to_string(), lhs_val, rhs_val),
          _ => todo!("hgen: unimplemented binary op: {:?}", b.op),
        };
        let instr = crate::hir::instr::HirInstr {
          vari,
          ty: ty_id,
        };
        let instr_hid = self.hir_mol.new_instr(instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(instr_hid);
        crate::hir::value::HirValue::Reg(instr_hid)
      }
      
      _ => todo!("hgen: unimplemented expr: {:?}", expr.vari),
    }
  }
  
}
