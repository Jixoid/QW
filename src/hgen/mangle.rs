use crate::hir::identy::HirId;
use crate::hir::module::HirModule;

pub trait Mangler {
  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<HirId>, ret_ty: Option<HirId>, arg_tys: &[HirId], hir_mol: &HirModule) -> String;
  fn mangle_global(&self, path: &[String], var_name: &str) -> String;
}

pub struct ItaniumMangler;

impl ItaniumMangler {
  
  pub fn new() -> Self { Self {} }

  pub fn mangle_type(arg: HirId, hir_mol: &HirModule) -> String {
    let ty = hir_mol.get_type(arg);
    use crate::hir::types::HirTypeVari;
    match &ty.vari {
      HirTypeVari::Void => "v".to_string(),
      HirTypeVari::Bool => "b".to_string(),
      HirTypeVari::Char => "c".to_string(),
      HirTypeVari::I8 => "a".to_string(),
      HirTypeVari::U8 => "h".to_string(),
      HirTypeVari::I16 => "s".to_string(),
      HirTypeVari::U16 => "t".to_string(),
      HirTypeVari::I32 => "i".to_string(),
      HirTypeVari::U32 => "j".to_string(),
      HirTypeVari::I64 => "l".to_string(),
      HirTypeVari::U64 => "m".to_string(),
      HirTypeVari::I128 => "n".to_string(),
      HirTypeVari::U128 => "o".to_string(),
      HirTypeVari::F16 => "Dh".to_string(),
      HirTypeVari::F32 => "f".to_string(),
      HirTypeVari::F64 => "d".to_string(),
      HirTypeVari::F128 => "g".to_string(),
      HirTypeVari::ISize => "l".to_string(),
      HirTypeVari::USize => "m".to_string(),
      HirTypeVari::Ptr => "Pv".to_string(),
      HirTypeVari::Null => "Dn".to_string(),
      HirTypeVari::PointerOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::AccessKind::IMM {
          s.push('K');
        }
        s.push_str(&Self::mangle_type(*sub, hir_mol));
        s
      }
      HirTypeVari::ReferenceOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::AccessKind::IMM {
          s.push('K');
        }
        s.push_str(&Self::mangle_type(*sub, hir_mol));
        s
      }
      HirTypeVari::Struct(_) | HirTypeVari::Iface(_) | HirTypeVari::Enum(_) => {
        "S_".to_string()
      }
      _ => "v".to_string(),
    }
  }

}

impl Mangler for ItaniumMangler {

  fn mangle_func(&self, path: &[String], func_name: &str, _self_ty: Option<HirId>, _ret_ty: Option<HirId>, arg_tys: &[HirId], hir_mol: &HirModule) -> String {
    let mut s = String::from("_ZN");
    for p in path {
      s.push_str(&format!("{}{}", p.len(), p));
    }
    s.push_str(&format!("{}{}", func_name.len(), func_name));
    s.push('E');
    
    if let Some(st) = _self_ty {
      s.push_str(&Self::mangle_type(st, hir_mol));
    }
    
    if arg_tys.is_empty() && _self_ty.is_none() {
      s.push('v');
    } else {
      for &arg in arg_tys {
        s.push_str(&Self::mangle_type(arg, hir_mol));
      }
    }
    s
  }

  fn mangle_global(&self, path: &[String], var_name: &str) -> String {
    let mut s = String::from("_ZN");
    for p in path {
      s.push_str(&format!("{}{}", p.len(), p));
    }
    s.push_str(&format!("{}{}", var_name.len(), var_name));
    s.push('E');
    s
  }
  
}



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
