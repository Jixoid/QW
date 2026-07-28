use crate::control::{Module, identy::{IdentyId, IdentyKind}};
use crate::hir::module::HirModule;
use crate::ast::decls::{Decl, DeclVari};
use crate::hir::global::HirGlobalVar;
use std::collections::HashMap;
use crate::hgen::mangle::{Mangler, ItaniumMangler};

pub struct HGen<'a,'d> {
  pub ast_mol: &'a Module<'a,'d>,
  pub hir_mol: HirModule,

  pub mangler: Box<dyn Mangler>,

  pub map_decl: HashMap<IdentyId, crate::hir::identy::HirId>,
  pub map_type: HashMap<IdentyId, crate::hir::identy::HirId>,
}

impl<'a,'d> HGen<'a,'d> {

  pub fn new(ast_mol: &'a Module<'a,'d>) -> Self {
    Self {
      ast_mol,
      hir_mol: HirModule::new(ast_mol.name.clone()),
      mangler: Box::new(ItaniumMangler::new()),
      map_decl: HashMap::new(),
      map_type: HashMap::new(),
    }
  }

  pub fn generate(mut self) -> HirModule {
    self.gen_module();
    self.hir_mol
  }

  fn gen_module(&mut self) {
    for (i,x) in self.ast_mol.list_decl.iter().enumerate() {
      let ast_id = IdentyId::new(IdentyKind::Decl, 0, i as u32);
      let decl = x;
      self.gen_decl(ast_id, decl);
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
        let mut final_name = self.mangler.mangle_global(&self.ast_mol.name, &decl_name_str);
        let mut is_weak = false;

        if let Some(attrs) = self.ast_mol.map_attr.get(&ast_id) {
          for attr in attrs {
            let key = attr.key.str();
            if key == "mangle" {
              if let Some(val) = &attr.val {
                if val.str() == "bare" {
                  final_name = decl_name_str.clone();
                }
              }
            } else if key == "weak" {
              is_weak = true;
            }
          }
        }

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
        let mut final_name = self.mangler.mangle_func(&self.ast_mol.name, &decl_name_str);
        let mut is_weak = false;

        if let Some(attrs) = self.ast_mol.map_attr.get(&ast_id) {
          for attr in attrs {
            let key = attr.key.str();
            if key == "mangle" {
              if let Some(val) = &attr.val {
                if val.str() == "bare" {
                  final_name = decl_name_str.clone();
                }
              }
            } else if key == "weak" {
              is_weak = true;
            }
          }
        }

        let (ret_ty, arg_tys) = {
          let hir_ty = self.hir_mol.get_type(ty_id);
          if let crate::hir::types::HirTypeVari::Function(fun) = &hir_ty.vari {
            let arg_tys: Vec<_> = fun.args.iter().map(|a| a.kind).collect();
            (fun.ret, arg_tys)
          } else {
            unreachable!("FunDecl kind must be Function type");
          }
        };

        let h_func = crate::hir::func::HirFunc::new(final_name, ret_ty, arg_tys, is_weak);
        let hid = self.hir_mol.new_func(h_func);
        self.map_decl.insert(ast_id, hid);
      }
      _ => {}
    }
  }

  fn gen_type(&mut self, ast_id: IdentyId) -> crate::hir::identy::HirId {
    if let Some(&hid) = self.map_type.get(&ast_id) {
      return hid;
    }

    let ty = self.ast_mol.get_type(ast_id);
    if let crate::ast::types::TypeVari::Path{path} = &ty.vari {
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
      _ => panic!("unimplemented type in hgen: {:?}", ty.vari),
    };

    self.hir_mol.list_type[hid.index() as usize].vari = hir_ty_vari;
    hid
  }
  
}
