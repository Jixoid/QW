use crate::control::identy::IdentyId;
use crate::control::module::Module;
use super::Mangler;

pub struct QwMangler;

impl QwMangler {

  pub fn new() -> Self { Self {} }

  pub fn mangle_type(arg: IdentyId, ast_mol: &Module) -> String {
    let ty = ast_mol.get_type(arg);
    use crate::ast::types::TypeVari;
    match &ty.vari {
      TypeVari::Void => "v".to_string(),
      TypeVari::Bool => "b".to_string(),
      TypeVari::Char => "c".to_string(),
      TypeVari::Int{bit: 8, sig: true} => "St".to_string(),
      TypeVari::Int{bit: 8, sig: false} => "Ut".to_string(),
      TypeVari::Int{bit: 16, sig: true} => "Ss".to_string(),
      TypeVari::Int{bit: 16, sig: false} => "Us".to_string(),
      TypeVari::Int{bit: 32, sig: true} => "Si".to_string(),
      TypeVari::Int{bit: 32, sig: false} => "Ui".to_string(),
      TypeVari::Int{bit: 64, sig: true} => "Sl".to_string(),
      TypeVari::Int{bit: 64, sig: false} => "Ul".to_string(),
      TypeVari::Int{bit: 128, sig: true} => "Sy".to_string(),
      TypeVari::Int{bit: 128, sig: false} => "Uy".to_string(),
      TypeVari::Int{..} => "x".to_string(),
      TypeVari::Float{bit: 16} => "h".to_string(),
      TypeVari::Float{bit: 32} => "f".to_string(),
      TypeVari::Float{bit: 64} => "d".to_string(),
      TypeVari::Float{bit: 128} => "g".to_string(),
      TypeVari::Float{..} => "x".to_string(),
      TypeVari::ArchSize{sig: true} => "Sn".to_string(),
      TypeVari::ArchSize{sig: false} => "Un".to_string(),
      TypeVari::Ptr => "p".to_string(),
      TypeVari::Null => "l".to_string(),
      TypeVari::PointerOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::types::AccessKind::MUT {
          s.push('M');
        }
        s.push_str(&Self::mangle_type(*sub, ast_mol));
        s
      }
      TypeVari::ReferenceOf{sub, acc} => {
        let mut s = String::from("R");
        if *acc == crate::ast::types::AccessKind::MUT {
          s.push('M');
        }
        s.push_str(&Self::mangle_type(*sub, ast_mol));
        s
      }
      TypeVari::Struct(_) | TypeVari::Iface(_) | TypeVari::Enum(_) | TypeVari::Trait(_) | TypeVari::Flags(_) => {
        "x".to_string() // Temporary fallback for complex types
      }
      TypeVari::Path(p) => {
        if let Some(resolved_id) = p.last() {
          if resolved_id.kind() == crate::control::IdentyKind::Type {
            return Self::mangle_type(*resolved_id, ast_mol);
          } else {
            return "x".to_string(); // Temporary fallback for complex types
          }
        }
        "v".to_string()
      }
      _ => "v".to_string(),
    }
  }

}

impl Mangler for QwMangler {

  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<IdentyId>, ret_ty: Option<IdentyId>, arg_tys: &[IdentyId], ast_mol: &Module) -> String {
    let mut s = String::from("_qw_");
    for p in path {
      s.push_str(&format!("{}{}", p.len(), p));
    }
    s.push_str(&format!("{}{}", func_name.len(), func_name));
    
    // Self-type
    if let Some(st) = self_ty {
      s.push_str(&Self::mangle_type(st, ast_mol));
    } else {
      s.push('v');
    }
    
    // Return-type
    if let Some(r) = ret_ty {
      s.push_str(&Self::mangle_type(r, ast_mol));
    } else {
      s.push('v');
    }

    // Argument types
    for &arg in arg_tys {
      s.push_str(&Self::mangle_type(arg, ast_mol));
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

  #[test]
  fn test_qw_mangler_global() {
    let mangler = QwMangler::new();
    let path = vec!["std".to_string(), "io".to_string()];
    let sym = mangler.mangle_global(&path, "print");
    assert_eq!(sym, "_qw_3std2io5print");
  }
}

