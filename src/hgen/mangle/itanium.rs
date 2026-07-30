use crate::hir::identy::HirId;
use crate::hir::module::HirModule;
use super::Mangler;


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
        if *acc == crate::ast::types::AccessKind::IMM {
          s.push('K');
        }
        s.push_str(&Self::mangle_type(*sub, hir_mol));
        s
      }
      HirTypeVari::ReferenceOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::types::AccessKind::IMM {
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


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_itanium_mangler_global() {
    let mangler = ItaniumMangler::new();
    let path = vec!["std".to_string(), "io".to_string()];
    let sym = mangler.mangle_global(&path, "print");
    assert_eq!(sym, "_ZN3std2io5printE");
  }
}

