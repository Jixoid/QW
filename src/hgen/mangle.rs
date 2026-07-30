use crate::hir::identy::HirId;
use crate::hir::module::HirModule;

pub trait Mangler {
  fn mangle_func(&self, path: &[String], func_name: &str, arg_tys: &[HirId], hir_mol: &HirModule) -> String;
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

  fn mangle_func(&self, path: &[String], func_name: &str, arg_tys: &[HirId], hir_mol: &HirModule) -> String {
    let mut s = String::from("_ZN");
    for p in path {
      s.push_str(&format!("{}{}", p.len(), p));
    }
    s.push_str(&format!("{}{}", func_name.len(), func_name));
    s.push('E');
    
    if arg_tys.is_empty() {
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
