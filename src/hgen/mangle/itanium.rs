use crate::control::identy::IdentyId;
use crate::control::module::Module;
use super::Mangler;

pub struct ItaniumMangler;

impl ItaniumMangler {
  
  pub fn new() -> Self { Self {} }

  pub fn mangle_type(arg: IdentyId, ast_mol: &Module) -> String {
    let ty = ast_mol.get_type(arg);
    use crate::ast::types::TypeVari;
    match &ty.vari {
      TypeVari::Void => "v".to_string(),
      TypeVari::Bool => "b".to_string(),
      TypeVari::Char => "c".to_string(),
      TypeVari::Int{bit: 8, sig: true} => "a".to_string(),
      TypeVari::Int{bit: 8, sig: false} => "h".to_string(),
      TypeVari::Int{bit: 16, sig: true} => "s".to_string(),
      TypeVari::Int{bit: 16, sig: false} => "t".to_string(),
      TypeVari::Int{bit: 32, sig: true} => "i".to_string(),
      TypeVari::Int{bit: 32, sig: false} => "j".to_string(),
      TypeVari::Int{bit: 64, sig: true} => "l".to_string(),
      TypeVari::Int{bit: 64, sig: false} => "m".to_string(),
      TypeVari::Int{bit: 128, sig: true} => "n".to_string(),
      TypeVari::Int{bit: 128, sig: false} => "o".to_string(),
      TypeVari::Int{..} => "v".to_string(),
      TypeVari::Float{bit: 16} => "Dh".to_string(),
      TypeVari::Float{bit: 32} => "f".to_string(),
      TypeVari::Float{bit: 64} => "d".to_string(),
      TypeVari::Float{bit: 128} => "g".to_string(),
      TypeVari::Float{..} => "v".to_string(),
      TypeVari::ArchSize{sig: true} => "l".to_string(),
      TypeVari::ArchSize{sig: false} => "m".to_string(),
      TypeVari::Ptr => "Pv".to_string(),
      TypeVari::Null => "Dn".to_string(),
      TypeVari::PointerOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::types::AccessKind::IMM {
          s.push('K');
        }
        s.push_str(&Self::mangle_type(*sub, ast_mol));
        s
      }
      TypeVari::ReferenceOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::types::AccessKind::IMM {
          s.push('K');
        }
        s.push_str(&Self::mangle_type(*sub, ast_mol));
        s
      }
      TypeVari::Struct(_) | TypeVari::Iface(_) | TypeVari::Enum(_) | TypeVari::Trait(_) | TypeVari::Flags(_) => {
        "S_".to_string()
      }
      TypeVari::Path(p) => {
        if let Some(resolved_id) = p.last() {
          if resolved_id.kind() == crate::control::IdentyKind::Type {
            return Self::mangle_type(*resolved_id, ast_mol);
          } else {
            return "S_".to_string();
          }
        }
        "v".to_string()
      }
      _ => "v".to_string(),
    }
  }

}

impl Mangler for ItaniumMangler {

  fn mangle_func(&self, path: &[String], func_name: &str, _self_ty: Option<IdentyId>, _ret_ty: Option<IdentyId>, arg_tys: &[IdentyId], ast_mol: &Module) -> String {
    let mut s = String::from("_ZN");
    for p in path {
      s.push_str(&format!("{}{}", p.len(), p));
    }
    s.push_str(&format!("{}{}", func_name.len(), func_name));
    s.push('E');
    
    if let Some(st) = _self_ty {
      s.push_str(&Self::mangle_type(st, ast_mol));
    }
    
    if arg_tys.is_empty() && _self_ty.is_none() {
      s.push('v');
    } else {
      for &arg in arg_tys {
        s.push_str(&Self::mangle_type(arg, ast_mol));
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

