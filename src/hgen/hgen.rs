use crate::{control::{Module, identy::{AstId, IdentyKind}}, hir::types::{HirFloatSize, HirTypeVari}};
use crate::hir::module::HirModule;
use crate::ast::decls::{Decl, DeclVari};
use crate::hir::global::HirGlobalVar;
use std::collections::HashMap;
use crate::hgen::mangle::{Mangler, ManglerKind};

pub struct HGen<'a, 'd: 'a> {
  pub ast_mol: &'a Module<'a, 'd>,
  pub target: &'a crate::layout::Target,
  pub hir_mol: HirModule,

  pub mangler: ManglerKind,
  pub is_debug: bool,

  pub map_decl: HashMap<AstId, crate::hir::identy::HirId>,
  pub map_type: HashMap<AstId, crate::hir::identy::HirId>,
  pub local_scope: HashMap<String, crate::hir::identy::HirId>,
  pub parent_path: HashMap<AstId, Vec<String>>,
  pub parent_decl: HashMap<AstId, AstId>,
}

impl<'a,'d> HGen<'a,'d> {

  pub fn get_attr_val(&self, id: AstId, name: &str) -> Option<String> {
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

  pub fn has_attr(&self, id: AstId, name: &str) -> bool {
    if let Some(attrs) = self.ast_mol.map_attr.get(&id) {
      for attr in attrs {
        if attr.key.str() == name {
          return true;
        }
      }
    }
    false
  }

  
  pub fn new(ast_mol: &'a Module<'a,'d>, is_debug: bool, target: &'a crate::layout::Target) -> Self {
    Self {
      ast_mol,
      target,
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
    let root_id = AstId::new(IdentyKind::Decl, 0, 0);
    self.build_path_map(root_id, vec![]);
    
    self.gen_module();
    self.hir_mol
  }

  
  fn build_path_map(&mut self, decl_id: AstId, mut current_path: Vec<String>) {
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
      DeclVari::Extend(ext) => {
        for fun_id in &ext.impls {
          self.parent_decl.insert(*fun_id, decl_id);
          self.build_path_map(*fun_id, current_path.clone());
        }
      }
      DeclVari::Generic(g) => {
        for child_id in &g.decls {
          self.parent_decl.insert(*child_id, decl_id);
          self.build_path_map(*child_id, current_path.clone());
        }
      }
      _ => {}
    }
  }

  fn gen_module(&mut self) {
    let mut skipped = std::collections::HashSet::new();
    for x in &self.ast_mol.list_decl {
      if let crate::ast::DeclVari::Generic(g) = &x.vari {
        for &id in &g.decls {
          skipped.insert(id);
        }
      }
    }
    
    let mut changed = true;
    while changed {
      changed = false;
      for (i, _) in self.ast_mol.list_decl.iter().enumerate() {
        let child = crate::control::identy::AstId::new(crate::control::identy::IdentyKind::Decl, 0, i as u32);
        if skipped.contains(&child) { continue; }
        if let Some(&parent) = self.parent_decl.get(&child) {
          if skipped.contains(&parent) {
            skipped.insert(child);
            changed = true;
          }
        }
      }
    }

    // Global Variables
    for (i,x) in self.ast_mol.list_decl.iter().enumerate() {
      let ast_id = crate::control::identy::AstId::new(crate::control::identy::IdentyKind::Decl, 0, i as u32);
      if skipped.contains(&ast_id) { continue; }
      if let crate::ast::DeclVari::Var(_) = &x.vari {
        self.gen_decl(ast_id, x);
      }
    }
    
    // Functions and others
    for (i,x) in self.ast_mol.list_decl.iter().enumerate() {
      let ast_id = crate::control::identy::AstId::new(crate::control::identy::IdentyKind::Decl, 0, i as u32);
      if skipped.contains(&ast_id) { continue; }
      if let crate::ast::DeclVari::Var(_) = &x.vari { continue; }
      self.gen_decl(ast_id, x);
    }
  }

  fn gen_decl(&mut self, ast_id: AstId, decl: &Decl) {
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

        let (ast_ret_ty, ast_self_ty, ast_arg_tys) = {
          let ast_ty = self.ast_mol.get_type(f.kind);
          if let crate::ast::types::TypeVari::Function(fun) = &ast_ty.vari {
            let mut args = fun.args.iter();
            let mut ast_self_ty = None;
            let mut ast_arg_tys = Vec::new();
            if is_instance_method {
              if let Some(first_arg) = args.next() {
                ast_self_ty = Some(first_arg.kind);
              }
            }
            ast_arg_tys.extend(args.map(|a| a.kind));
            (fun.ret, ast_self_ty, ast_arg_tys)
          } else {
            unreachable!("FunDecl kind must be Function type");
          }
        };

        let (ret_ty, _self_ty, arg_tys) = {
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
        
        let mut final_name = current_mangler.mangle_func(parent_path, &decl_name_str, ast_self_ty, Some(ast_ret_ty), &ast_arg_tys, &self.ast_mol);

        if let Some(parent_id) = self.parent_decl.get(&ast_id) {
          let parent = self.ast_mol.get_decl(*parent_id);
          if let crate::ast::DeclVari::Extend(ext) = &parent.vari {
             if let ManglerKind::Qw = current_mangler {
                 let t_tgt = crate::hgen::mangle::qw::QwMangler::mangle_type(ext.target, &self.ast_mol);
                 let t_tr = crate::hgen::mangle::qw::QwMangler::mangle_type(ext.tr, &self.ast_mol);
                 
                 let inner = current_mangler.mangle_func(&[], &decl_name_str, ast_self_ty, Some(ast_ret_ty), &ast_arg_tys, &self.ast_mol);
                 let inner = inner.trim_start_matches("_qw_");
                 
                 final_name = format!("qw_{}_ext_{}{}", t_tgt, t_tr, inner);
             }
          }
        }

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
      DeclVari::Module(_) | DeclVari::Import(_, _) | DeclVari::ImportWildcard(_, _) | DeclVari::Extend(_) | DeclVari::Generic(_) => {},
    }
  }

  fn gen_type(&mut self, ast_id: AstId) -> crate::hir::identy::HirId {
    if let Some(&hid) = self.map_type.get(&ast_id) {
      return hid;
    }

    let ty = self.ast_mol.get_type(ast_id);
    if let crate::ast::types::TypeVari::Path(path) = &ty.vari {
      if path.len() > 0 {
        let resolved_id = path.last().unwrap();
        // Handle if it's a Declaration
        if resolved_id.kind() == crate::control::identy::IdentyKind::Decl {
          let decl = self.ast_mol.get_decl(*resolved_id);
          if let crate::ast::DeclVari::Using(target_ty_id) = &decl.vari {
            let hid = self.gen_type(*target_ty_id);
            self.map_type.insert(ast_id, hid);
            return hid;
          }
        }
        
        if ast_id == *resolved_id {
          panic!("Infinite loop detected in gen_type: ast_id == resolved_id ({:?})", ast_id);
        }
        
        let hid = self.gen_type(*resolved_id);
        self.map_type.insert(ast_id, hid);
        return hid;
      }
    }

    let hid = self.hir_mol.new_type(crate::hir::types::HirType{ vari: crate::hir::types::HirTypeVari::Null });
    self.map_type.insert(ast_id, hid);
    
    let hir_ty_vari = match &ty.vari {
      crate::ast::TypeVari::ArchSize{sig} => HirTypeVari::Int{bit: (self.target.pointer_size*8) as u32, sig: *sig},
      crate::ast::TypeVari::Int{bit, sig} => HirTypeVari::Int{bit: *bit, sig: *sig},
      crate::ast::TypeVari::Float{bit} => {
        HirTypeVari::Float{bit: match *bit {
          16  => HirFloatSize::F16,
          32  => HirFloatSize::F32,
          64  => HirFloatSize::F64,
          128 => HirFloatSize::F128,
          _ => panic!("unknown float size"),
        }}
      }

      crate::ast::types::TypeVari::Bool => HirTypeVari::Bool,
      crate::ast::types::TypeVari::Char => HirTypeVari::Char,
      crate::ast::types::TypeVari::Ptr  => HirTypeVari::Ptr,
      crate::ast::types::TypeVari::Void => HirTypeVari::Void,
      crate::ast::types::TypeVari::Null => HirTypeVari::Null,

      crate::ast::types::TypeVari::ReferenceOf{..} => HirTypeVari::Ptr,
      crate::ast::types::TypeVari::PointerOf{..}   => HirTypeVari::Ptr,

      crate::ast::types::TypeVari::Enum(..)  => HirTypeVari::Int{bit: 32, sig: true},
      crate::ast::types::TypeVari::Flags(..) => HirTypeVari::Int{bit: 32, sig: true},

      
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
      
      crate::ast::types::TypeVari::Iface(..) | crate::ast::types::TypeVari::Trait(..) => {
        let ptr_ty = self.hir_mol.new_type(crate::hir::types::HirType { vari: crate::hir::types::HirTypeVari::Ptr });
        crate::hir::types::HirTypeVari::Struct(crate::hir::types::HirStructType {
          base: Vec::new(),
          vars: vec![
            crate::hir::types::HirFieldType {
              name: "data".to_string(),
              kind: ptr_ty,
            },
            crate::hir::types::HirFieldType {
              name: "vtable".to_string(),
              kind: ptr_ty,
            },
          ],
        })
      }
      
      crate::ast::types::TypeVari::ArrayOf{sub, ext} => {
        let sub_hid = self.gen_type(*sub);
        let len = if ext.is_empty() {
          None
        } else {
          Some(ext.iter().fold(1u64, |acc, &l| acc.saturating_mul(l as u64)))
        };
        crate::hir::types::HirTypeVari::ArrayOf(crate::hir::types::HirArrayType {
          sub: sub_hid,
          len,
        })
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

  fn unwrap_type_vari(&self, ty_id: AstId) -> &crate::ast::TypeVari<'a> {
    let mut current = ty_id;
    for _ in 0..10 {
      if current.kind() != crate::control::IdentyKind::Type {
        break;
      }
      let ty = self.ast_mol.get_type(current);
      match &ty.vari {
        crate::ast::TypeVari::Path(p) if !p.is_empty() => {
          if p[0].kind() == crate::control::IdentyKind::Type {
            current = p[0];
          } else {
            return &ty.vari;
          }
        }
        _ => return &ty.vari,
      }
    }
    &self.ast_mol.get_type(current).vari
  }

  fn check_is_int(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Int { .. } | crate::ast::TypeVari::ArchSize { .. })
  }

  fn check_is_float(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Float { .. })
  }

  fn check_is_bool(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Bool)
  }

  fn check_is_char(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Char)
  }

  fn check_is_ptr(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Ptr | crate::ast::TypeVari::PointerOf { .. })
  }

  fn check_is_reference(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::ReferenceOf { .. })
  }

  fn check_is_void(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Void)
  }

  fn check_is_null(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Null)
  }

  fn check_is_signed(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Int { sig: true, .. } | crate::ast::TypeVari::ArchSize { sig: true })
  }

  fn check_is_unsigned(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Int { sig: false, .. } | crate::ast::TypeVari::ArchSize { sig: false })
  }

  fn check_is_array(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::ArrayOf{..} )
  }

  fn check_is_struct(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Struct(..))
  }

  fn check_is_enum(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Enum(..))
  }

  fn check_is_flags(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Flags(..))
  }

  fn check_is_function(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Function(..))
  }

  fn check_is_trait(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Trait(..))
  }

  fn check_is_iface(&self, target_ty: AstId) -> bool {
    let vari = self.unwrap_type_vari(target_ty);
    matches!(vari, crate::ast::TypeVari::Iface(..))
  }

  fn check_is_type_equal(&self, a: AstId, b: AstId) -> bool {
    if a == b { return true; }
    let ta = self.ast_mol.get_type(a);
    let tb = self.ast_mol.get_type(b);
    match (&ta.vari, &tb.vari) {
      (crate::ast::TypeVari::Path(p1), crate::ast::TypeVari::Path(p2)) => p1 == p2,
      (crate::ast::TypeVari::Int{bit: b1, sig: s1}, crate::ast::TypeVari::Int{bit: b2, sig: s2}) => b1 == b2 && s1 == s2,
      (crate::ast::TypeVari::Float{bit: b1}, crate::ast::TypeVari::Float{bit: b2}) => b1 == b2,
      (crate::ast::TypeVari::ArchSize{sig: s1}, crate::ast::TypeVari::ArchSize{sig: s2}) => s1 == s2,
      (crate::ast::TypeVari::Bool, crate::ast::TypeVari::Bool) => true,
      (crate::ast::TypeVari::Char, crate::ast::TypeVari::Char) => true,
      (crate::ast::TypeVari::Ptr, crate::ast::TypeVari::Ptr) => true,
      (crate::ast::TypeVari::Void, crate::ast::TypeVari::Void) => true,
      (crate::ast::TypeVari::Null, crate::ast::TypeVari::Null) => true,
      _ => false,
    }
  }

  fn check_is_extended(&self, base_ty: AstId, trait_ty: AstId) -> bool {
    for decl in self.ast_mol.list_decl.iter() {
      if let crate::ast::DeclVari::Extend(ext) = &decl.vari {
        if self.check_is_type_equal(ext.target, base_ty) && self.check_is_type_equal(ext.tr, trait_ty) {
          return true;
        }
      }
    }
    false
  }

  fn eval_type_query(&self, name: &str, target_ty: AstId) -> bool {
    match name {
      "is_int" => self.check_is_int(target_ty),
      "is_float" => self.check_is_float(target_ty),
      "is_bool" => self.check_is_bool(target_ty),
      "is_char" => self.check_is_char(target_ty),
      "is_ptr" | "is_pointer" => self.check_is_ptr(target_ty),
      "is_reference" => self.check_is_reference(target_ty),
      "is_void" => self.check_is_void(target_ty),
      "is_null" => self.check_is_null(target_ty),
      "is_signed" => self.check_is_signed(target_ty),
      "is_unsigned" => self.check_is_unsigned(target_ty),
      "is_array" => self.check_is_array(target_ty),
      "is_struct" => self.check_is_struct(target_ty),
      "is_enum" => self.check_is_enum(target_ty),
      "is_flags" => self.check_is_flags(target_ty),
      "is_function" => self.check_is_function(target_ty),
      "is_trait" => self.check_is_trait(target_ty),
      "is_iface" => self.check_is_iface(target_ty),
      _ => false,
    }
  }

  fn gen_expr(&mut self, expr_id: AstId, func_hid: crate::hir::identy::HirId, current_block: &mut crate::hir::identy::HirId) -> crate::hir::value::HirValue {
    let expr = self.ast_mol.get_expr(expr_id);
    let ty_id = self.gen_type(expr.ty);

    match &expr.vari {
      crate::ast::ExprVari::Call(c) => {
        let callee_expr = self.ast_mol.get_expr(c.callee);
        let callee_name = match &callee_expr.vari {
          crate::ast::ExprVari::Nick(n) => Some(self.ast_mol.nick_map[n.idx as usize].clone()),
          crate::ast::ExprVari::Path(p) => {
            let path_strs: Vec<String> = p.iter().map(|n| self.ast_mol.nick_map[n.idx as usize].clone()).collect();
            if path_strs.len() == 2 && path_strs[0] == "sys" {
              Some(path_strs[1].clone())
            } else if path_strs.len() == 1 {
              Some(path_strs[0].clone())
            } else {
              None
            }
          }
          _ => None,
        };

        if let Some(ref name) = callee_name {
          if name == "is_extended" && c.generic_args.len() == 2 && c.args.is_empty() {
            return crate::hir::value::HirValue::ConstBool(self.check_is_extended(c.generic_args[0], c.generic_args[1]));
          }
          if c.generic_args.len() == 1 && c.args.is_empty() {
            if matches!(
              name.as_str(),
              "is_int" | "is_float" | "is_bool" | "is_char" | "is_ptr" | "is_pointer" |
              "is_reference" | "is_void" | "is_null" | "is_signed" | "is_unsigned" |
              "is_array" | "is_struct" | "is_enum" | "is_flags" | "is_function" |
              "is_trait" | "is_iface"
            ) {
              return crate::hir::value::HirValue::ConstBool(self.eval_type_query(name, c.generic_args[0]));
            }
          }
        }

        let func_val = self.gen_expr(c.callee, func_hid, current_block);
        let arg_vals = c.args.iter().map(|&a| self.gen_expr(a, func_hid, current_block)).collect();
        let call_instr = crate::hir::instr::HirInstr {
          vari: crate::hir::instr::HirInstrVari::Call(func_val, arg_vals),
          ty: ty_id,
        };
        let call_hid = self.hir_mol.new_instr(call_instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(call_hid);
        crate::hir::value::HirValue::Reg(call_hid)
      }

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
          let mod_name = &self.ast_mol.nick_map[p[0].idx as usize];
          let item_name = &self.ast_mol.nick_map[p[1].idx as usize];
          if mod_name == "sys" {
            if matches!(
              item_name.as_str(),
              "is_int" | "is_float" | "is_bool" | "is_char" | "is_ptr" | "is_pointer" |
              "is_reference" | "is_void" | "is_null" | "is_signed" | "is_unsigned" |
              "is_array" | "is_struct" | "is_enum" | "is_flags" | "is_function" |
              "is_trait" | "is_iface"
            ) {
              return crate::hir::value::HirValue::ConstBool(false);
            }
          }

          let variant_name = item_name.clone();
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
        if matches!(
          name.as_str(),
          "is_int" | "is_float" | "is_bool" | "is_char" | "is_ptr" | "is_pointer" |
          "is_reference" | "is_void" | "is_null" | "is_signed" | "is_unsigned" |
          "is_array" | "is_struct" | "is_enum" | "is_flags" | "is_function" |
          "is_trait" | "is_iface"
        ) {
          return crate::hir::value::HirValue::ConstBool(false);
        }
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
              global_ast_id = Some(crate::control::identy::AstId::new(crate::control::IdentyKind::Decl, 0, i as u32));
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
      
      crate::ast::ExprVari::Index(idx) => {
        let target_expr = self.ast_mol.get_expr(idx.target);
        let target_ptr = match &target_expr.vari {
          crate::ast::ExprVari::Nick(n) => {
            let name = self.ast_mol.nick_map[n.idx as usize].clone();
            if let Some(&local_hid) = self.local_scope.get(&name) {
              crate::hir::value::HirValue::Reg(local_hid)
            } else {
              panic!("Index target Nick {} not found", name);
            }
          }
          _ => self.gen_expr(idx.target, func_hid, current_block),
        };

        let index_val = self.gen_expr(idx.index, func_hid, current_block);
        
        let index_u32 = match index_val {
          crate::hir::value::HirValue::ConstInt(i) => i as u32,
          _ => 0,
        };

        let is_slice = match &self.unwrap_type_vari(target_expr.ty) {
          crate::ast::TypeVari::ArrayOf { ext, .. } => ext.is_empty(),
          _ => false,
        };

        let element_ptr = if is_slice {
          // Slice fat pointer { ptr, len }: GEP index 0 (ptr member), load buffer pointer, GEP by element index
          let ptr_ty = self.hir_mol.new_type(crate::hir::types::HirType { vari: crate::hir::types::HirTypeVari::Ptr });
          let gep_ptr_field = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::GetElementPtr(ptr_ty, target_ptr, vec![0, 0]),
            ty: ptr_ty,
          };
          let gep_ptr_hid = self.hir_mol.new_instr(gep_ptr_field);
          self.hir_mol.get_block_mut(*current_block).push_instr(gep_ptr_hid);

          let load_buf_ptr = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::Load(ptr_ty, crate::hir::value::HirValue::Reg(gep_ptr_hid)),
            ty: ptr_ty,
          };
          let load_buf_hid = self.hir_mol.new_instr(load_buf_ptr);
          self.hir_mol.get_block_mut(*current_block).push_instr(load_buf_hid);

          let gep_elem = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::GetElementPtr(ty_id, crate::hir::value::HirValue::Reg(load_buf_hid), vec![index_u32]),
            ty: ty_id,
          };
          let gep_elem_hid = self.hir_mol.new_instr(gep_elem);
          self.hir_mol.get_block_mut(*current_block).push_instr(gep_elem_hid);
          gep_elem_hid
        } else {
          // Fixed size array [T, N]: GEP index [0, index] directly
          let gep_instr = crate::hir::instr::HirInstr {
            vari: crate::hir::instr::HirInstrVari::GetElementPtr(ty_id, target_ptr, vec![0, index_u32]),
            ty: ty_id,
          };
          let gep_hid = self.hir_mol.new_instr(gep_instr);
          self.hir_mol.get_block_mut(*current_block).push_instr(gep_hid);
          gep_hid
        };

        let load_instr = crate::hir::instr::HirInstr {
          vari: crate::hir::instr::HirInstrVari::Load(ty_id, crate::hir::value::HirValue::Reg(element_ptr)),
          ty: ty_id,
        };
        let load_hid = self.hir_mol.new_instr(load_instr);
        self.hir_mol.get_block_mut(*current_block).push_instr(load_hid);
        crate::hir::value::HirValue::Reg(load_hid)
      }

      _ => todo!("hgen: unimplemented expr: {:?}", expr.vari),
    }
  }
  
}
