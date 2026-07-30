use crate::{cgen::ICGen, layout::Target};

pub struct BasicCGen<'a> {
	pub target: &'a Target,
	pub instructions: Vec<String>,
	pub module_type_counter: u32,
}

impl<'a> BasicCGen<'a> {
	
	pub fn new(target: &'a Target) -> Self {
		Self{
			target,
			instructions: Vec::new(),
			module_type_counter: 0,
		}
	}
	
  fn type_to_llvm_str(&self, hir_mol: &crate::hir::module::HirModule, ty_id: crate::hir::identy::HirId) -> String {
    let ty = hir_mol.get_type(ty_id);
    match &ty.vari {
      crate::hir::types::HirTypeVari::ISize => "i64".to_string(),
      crate::hir::types::HirTypeVari::USize => "i64".to_string(),
      crate::hir::types::HirTypeVari::I8 | crate::hir::types::HirTypeVari::U8 => "i8".to_string(),
      crate::hir::types::HirTypeVari::I16 | crate::hir::types::HirTypeVari::U16 => "i16".to_string(),
      crate::hir::types::HirTypeVari::I32 | crate::hir::types::HirTypeVari::U32 => "i32".to_string(),
      crate::hir::types::HirTypeVari::I64 | crate::hir::types::HirTypeVari::U64 => "i64".to_string(),
      crate::hir::types::HirTypeVari::I128 | crate::hir::types::HirTypeVari::U128 => "i128".to_string(),
      crate::hir::types::HirTypeVari::F16 => "half".to_string(),
      crate::hir::types::HirTypeVari::F32 => "float".to_string(),
      crate::hir::types::HirTypeVari::F64 => "double".to_string(),
      crate::hir::types::HirTypeVari::F128 => "fp128".to_string(),
      crate::hir::types::HirTypeVari::Bool => "i1".to_string(),
      crate::hir::types::HirTypeVari::Char => "i32".to_string(),
      crate::hir::types::HirTypeVari::Ptr => "ptr".to_string(),
      crate::hir::types::HirTypeVari::Void => "void".to_string(),
      crate::hir::types::HirTypeVari::Null => "ptr".to_string(),
      crate::hir::types::HirTypeVari::PointerOf { .. } => "ptr".to_string(),
      crate::hir::types::HirTypeVari::ReferenceOf { .. } => "ptr".to_string(),
      crate::hir::types::HirTypeVari::Function(_) => "ptr".to_string(),
      crate::hir::types::HirTypeVari::Struct(_) => format!("%type.{}", ty_id.index()),
      crate::hir::types::HirTypeVari::Iface(_) => format!("%type.{}", ty_id.index()),
      _ => panic!("unimplemented type in cgen: {:?}", ty.vari),
    }
  }

  fn value_to_llvm_str(&self, hir_mol: &crate::hir::module::HirModule, val: &crate::hir::value::HirValue) -> String {
    match val {
      crate::hir::value::HirValue::ConstInt(i) => format!("{}", i),
      crate::hir::value::HirValue::ConstFloat(f) => format!("{:?}", f),
      crate::hir::value::HirValue::ConstBool(b) => if *b { "true".to_string() } else { "false".to_string() },
      crate::hir::value::HirValue::Reg(hid) => format!("%{}", hid.index()),
      crate::hir::value::HirValue::Global(hid) => {
        let global = hir_mol.get_global(*hid);
        format!("@{}", global.name)
      },
      crate::hir::value::HirValue::Null => "null".to_string(),
    }
  }

}

use std::fmt::Write;


impl<'a> ICGen for BasicCGen<'a> {

	fn generate(&mut self, hir_mol: &crate::hir::module::HirModule) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "; ModuleID = '{}'", hir_mol.name);
    let _ = writeln!(out, "source_filename = \"{}\"", hir_mol.name);
    let _ = writeln!(out, "");

    for (i, t) in hir_mol.list_type.iter().enumerate() {
      if let crate::hir::types::HirTypeVari::Struct(s) = &t.vari {
        let mut fields_str = String::new();
        for (idx, f) in s.vars.iter().enumerate() {
          fields_str.push_str(&self.type_to_llvm_str(hir_mol, f.kind));
          if idx + 1 < s.vars.len() {
            fields_str.push_str(", ");
          }
        }
        let _ = writeln!(out, "%type.{} = type {{ {} }}", i, fields_str);
      } else if let crate::hir::types::HirTypeVari::Iface(_) = &t.vari {
        let _ = writeln!(out, "%type.{} = type {{}}", i);
      }
    }
    let _ = writeln!(out, "");

    for g in &hir_mol.list_global {
      let mut modifiers = String::new();
      
      if g.is_weak {
        modifiers.push_str("weak ");
      }

      if g.is_const {
        modifiers.push_str("constant ");
      } else {
        modifiers.push_str("global ");
      }

      let ty_str = self.type_to_llvm_str(hir_mol, g.ty);
      let init_str = if let Some(init_val) = &g.init {
        self.value_to_llvm_str(hir_mol, init_val)
      } else {
        "0".to_string()
      };

      let _ = writeln!(out, "@{} = {}{} {}", g.name, modifiers, ty_str, init_str);
    }
    let _ = writeln!(out, "");

    for f in &hir_mol.list_func {
      let ret_ty_str = self.type_to_llvm_str(hir_mol, f.ret_ty);
      
      let mut args_str = String::new();
      for (i, arg_ty) in f.arg_tys.iter().enumerate() {
        args_str.push_str(&self.type_to_llvm_str(hir_mol, *arg_ty));
        args_str.push_str(&format!(" %{}", i));
        if i + 1 < f.arg_tys.len() {
          args_str.push_str(", ");
        }
      }

      let weak_str = if f.is_weak { "weak " } else { "" };
      
      let _ = writeln!(out, "define {}{} @{}({}) {{", weak_str, ret_ty_str, f.name, args_str);
      if f.blocks.is_empty() {
        let _ = writeln!(out, "entry:");

        if ret_ty_str == "void" {
          let _ = writeln!(out, "  ret void");
        } else if ret_ty_str == "i1" {
          let _ = writeln!(out, "  ret i1 false");
        } else if ret_ty_str == "ptr" {
          let _ = writeln!(out, "  ret ptr null");
        } else if ret_ty_str == "float" || ret_ty_str == "double" {
          let _ = writeln!(out, "  ret {} 0.0", ret_ty_str);
        } else {
          let _ = writeln!(out, "  ret {} 0", ret_ty_str);
        }
      } else {
        for block_id in &f.blocks {
          let block = hir_mol.get_block(*block_id);
          let _ = writeln!(out, "{}:", block.name);
          
          for instr_id in &block.instrs {
            let instr = hir_mol.get_instr(*instr_id);
            match &instr.vari {
              crate::hir::instr::HirInstrVari::Alloca(ty_id) => {
                let ty_str = self.type_to_llvm_str(hir_mol, *ty_id);
                let _ = writeln!(out, "  %{} = alloca {}", instr_id.index(), ty_str);
              }
              crate::hir::instr::HirInstrVari::Store(val, ptr) => {
                let val_str = self.value_to_llvm_str(hir_mol, val);
                let ptr_str = self.value_to_llvm_str(hir_mol, ptr);
                

                let val_ty_str = if let crate::hir::value::HirValue::Reg(ptr_hid) = ptr {
                  let ptr_instr = hir_mol.get_instr(*ptr_hid);
                  self.type_to_llvm_str(hir_mol, ptr_instr.ty)
                } else {
                  "i32".to_string()
                };

                let _ = writeln!(out, "  store {} {}, ptr {}", val_ty_str, val_str, ptr_str);
              }
              crate::hir::instr::HirInstrVari::Load(ty_id, ptr) => {
                let ty_str = self.type_to_llvm_str(hir_mol, *ty_id);
                let ptr_str = self.value_to_llvm_str(hir_mol, ptr);
                let _ = writeln!(out, "  %{} = load {}, ptr {}", instr_id.index(), ty_str, ptr_str);
              }
              crate::hir::instr::HirInstrVari::Ret(opt_val) => {
                if let Some(val) = opt_val {

                  let val_str = self.value_to_llvm_str(hir_mol, val);
                  let _ = writeln!(out, "  ret {} {}", ret_ty_str, val_str);
                } else {
                  let _ = writeln!(out, "  ret void");
                }
              }
              _ => {
                todo!("cgen: unimplemented instr {:?}", instr.vari);
              }
            }
          }
        }
      }
      
      let _ = writeln!(out, "}}\n");
    }

    out
	}

}
