use crate::hir::identy::HirId;
use crate::hir::module::HirModule;
use super::Mangler;


pub struct QwMangler;

impl QwMangler {

  pub fn new() -> Self { Self {} }

  pub fn mangle_type(arg: HirId, hir_mol: &HirModule) -> String {
    let ty = hir_mol.get_type(arg);
    use crate::hir::types::HirTypeVari;
    match &ty.vari {
      HirTypeVari::Void => "v".to_string(),
      HirTypeVari::Bool => "b".to_string(),
      HirTypeVari::Char => "c".to_string(),
      HirTypeVari::I8 => "St".to_string(),
      HirTypeVari::U8 => "Ut".to_string(),
      HirTypeVari::I16 => "Ss".to_string(),
      HirTypeVari::U16 => "Us".to_string(),
      HirTypeVari::I32 => "Si".to_string(),
      HirTypeVari::U32 => "Ui".to_string(),
      HirTypeVari::I64 => "Sl".to_string(),
      HirTypeVari::U64 => "Ul".to_string(),
      HirTypeVari::I128 => "Sy".to_string(),
      HirTypeVari::U128 => "Uy".to_string(),
      HirTypeVari::F16 => "h".to_string(),
      HirTypeVari::F32 => "f".to_string(),
      HirTypeVari::F64 => "d".to_string(),
      HirTypeVari::F128 => "g".to_string(),
      HirTypeVari::ISize => "Sn".to_string(),
      HirTypeVari::USize => "Un".to_string(),
      HirTypeVari::Ptr => "p".to_string(),
      HirTypeVari::Null => "l".to_string(),
      HirTypeVari::PointerOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::types::AccessKind::MUT {
          s.push('M');
        }
        s.push_str(&Self::mangle_type(*sub, hir_mol));
        s
      }
      HirTypeVari::ReferenceOf{sub, acc} => {
        let mut s = String::from("R");
        if *acc == crate::ast::types::AccessKind::MUT {
          s.push('M');
        }
        s.push_str(&Self::mangle_type(*sub, hir_mol));
        s
      }
      HirTypeVari::Struct(_) | HirTypeVari::Iface(_) | HirTypeVari::Enum(_) => {
        "x".to_string() // Temporary fallback for complex types
      }
      _ => "v".to_string(),
    }
  }

}

impl Mangler for QwMangler {

  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<HirId>, ret_ty: Option<HirId>, arg_tys: &[HirId], hir_mol: &HirModule) -> String {
    let mut s = String::from("_qw_");
    for p in path {
      s.push_str(&format!("{}{}", p.len(), p));
    }
    s.push('F');
    s.push_str(&format!("{}{}", func_name.len(), func_name));
    
    // Self-type
    if let Some(st) = self_ty {
      s.push_str(&Self::mangle_type(st, hir_mol));
    } else {
      s.push('v');
    }
    
    // Return-type
    if let Some(r) = ret_ty {
      s.push_str(&Self::mangle_type(r, hir_mol));
    } else {
      s.push('v');
    }

    // Argument types
    for &arg in arg_tys {
      s.push_str(&Self::mangle_type(arg, hir_mol));
    }
    
    s
  }

  fn mangle_global(&self, path: &[String], var_name: &str) -> String {
    let mut s = String::from("_qw_");
    for p in path {
      s.push_str(&format!("{}{}", p.len(), p));
    }
    s.push_str(&format!("{}{}", var_name.len(), var_name));
    s
  }
  
}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::hir::types::{HirType, HirTypeVari};
  use crate::ast::types::AccessKind as AstAccessKind;

  #[test]
  fn test_qw_mangler_global() {
    let mangler = QwMangler::new();
    let path = vec!["std".to_string(), "io".to_string()];
    let sym = mangler.mangle_global(&path, "print");
    assert_eq!(sym, "_qw_3std2io5print");
  }

  #[test]
  fn test_qw_mangler_func() {
    let mangler = QwMangler::new();
    let mut mol = HirModule::new("test".to_string());
    let path = vec!["test".to_string(), "vec".to_string(), "Vec".to_string()];
    
    let ret_ty = mol.new_type(HirType { vari: HirTypeVari::I32 });
    let struct_ty = mol.new_type(HirType { vari: HirTypeVari::Struct(crate::hir::types::HirStructType { base: vec![], vars: vec![] }) });
    let self_ty = mol.new_type(HirType { vari: HirTypeVari::ReferenceOf { sub: struct_ty, acc: AstAccessKind::MUT } });
    let arg1 = mol.new_type(HirType { vari: HirTypeVari::Ptr });

    let arg_tys = vec![arg1];
    let sym = mangler.mangle_func(&path, "draw", Some(self_ty), Some(ret_ty), &arg_tys, &mol);
    
    assert_eq!(sym, "_qw_4test3vec3VecF4drawRMxSip");
  }
}

