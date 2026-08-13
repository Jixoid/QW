use crate::control::identy::AstId;
use crate::control::module::Module;
use super::Mangler;


pub struct QwMangler;

impl QwMangler {

  pub fn new() -> Self { Self {} }

  pub fn mangle_type(arg: AstId, ast_mol: &Module) -> String {
    let ty = ast_mol.get_type(arg);
    use crate::ast::types::TypeVari;
    match &ty.vari {
      TypeVari::Void => "v".to_string(),
      TypeVari::Bool => "b".to_string(),
      TypeVari::Char => "c".to_string(),
      TypeVari::ArchSize{sig: true} => "Sn".to_string(),
      TypeVari::ArchSize{sig: false} => "Un".to_string(),
      TypeVari::Ptr => "p".to_string(),
      TypeVari::Null => "l".to_string(),
      TypeVari::SelfType => "x".to_string(),
      
      TypeVari::Int {bit, sig} => {
        let s = match *sig { true => 'S', false => 'U' };

        let b = match *bit { 8 => 't', 16 => 's', 32 => 'i', 64 => 'l', 128 => 'y', _ => todo!() };

        format!("{}{}", s,b)
      }
      
      TypeVari::Float{bit} => {
        match *bit {
          16  => 'h',
          32  => 'f',
          64  => 'd',
          128 => 'g',
          _ => todo!(),
        }.to_string()
      }
          
      TypeVari::PointerOf{sub, acc} => {
        let mut s = String::from("P");
        if *acc == crate::ast::types::AccessKind::MUT { s.push('M'); }
        s.push_str(&Self::mangle_type(*sub, ast_mol));
        s
      }
      
      TypeVari::ReferenceOf{sub, acc} => {
        let mut s = String::from("R");
        if *acc == crate::ast::types::AccessKind::MUT { s.push('M'); }
        s.push_str(&Self::mangle_type(*sub, ast_mol));
        s
      }
      
      TypeVari::Struct(_) | TypeVari::Iface(_) | TypeVari::Enum(_) | TypeVari::Trait(_) | TypeVari::Flags(_) => {
        for decl in &ast_mol.list_decl {
          if let crate::ast::DeclVari::Using(uid) = &decl.vari {
            if *uid == arg {
              let name = decl.name.to_string();
              if let Some(g_idx) = name.find("G") {
                if name[g_idx..].chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
                  let base_name = &name[..g_idx];
                  let generic_part = &name[g_idx..];
                  return format!("{}{}{}", base_name.len(), base_name, generic_part);
                }
              }
              return format!("{}{}", name.len(), name);
            }
          }
          if let crate::ast::DeclVari::Generic(g) = &decl.vari {
            for inner_decl_id in &g.decls {
              let inner_decl = ast_mol.get_decl(*inner_decl_id);
              if let crate::ast::DeclVari::Using(uid) = &inner_decl.vari {
                if *uid == arg {
                  let name = inner_decl.name.to_string();
                  if let Some(g_idx) = name.find("G") {
                    if name[g_idx..].chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
                      let base_name = &name[..g_idx];
                      let generic_part = &name[g_idx..];
                      return format!("{}{}{}", base_name.len(), base_name, generic_part);
                    }
                  }
                  return format!("{}{}", name.len(), name);
                }
              }
            }
          }
        }

        todo!()
      }
      
      TypeVari::Path(p) => {
        if let Some(resolved_id) = p.last() {
          if resolved_id.kind() == crate::control::IdentyKind::Type {
            let target_mol = if resolved_id.module() == 0 {
              ast_mol
            } else {
              ast_mol.get_dep(resolved_id.module())
            };
            
            // If underlying type is a primitive, mangle it directly
            let inner_ty = ast_mol.get_type(*resolved_id);
            match &inner_ty.vari {
              crate::ast::types::TypeVari::Void |
              crate::ast::types::TypeVari::Bool |
              crate::ast::types::TypeVari::Char |
              crate::ast::types::TypeVari::Int{..} |
              crate::ast::types::TypeVari::Float{..} |
              crate::ast::types::TypeVari::ArchSize{..} |
              crate::ast::types::TypeVari::Ptr |
              crate::ast::types::TypeVari::Null |
              crate::ast::types::TypeVari::SelfType => {
                return Self::mangle_type(*resolved_id, ast_mol);
              },
              _ => {}
            }

            for d in target_mol.list_decl.iter() {
              if let crate::ast::DeclVari::Using(uid) = &d.vari {
                if *uid == arg {
                  let name = d.name.to_string();
                  if let Some(g_idx) = name.find("G") {
                    if name[g_idx..].chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
                      let base_name = &name[..g_idx];
                      let generic_part = &name[g_idx..];
                      return format!("{}{}{}", base_name.len(), base_name, generic_part);
                    }
                  }
                  return format!("{}{}", name.len(), name);
                }
              }
            }
            return Self::mangle_type(*resolved_id, ast_mol);
          } else if resolved_id.kind() == crate::control::IdentyKind::Decl {
            let decl = ast_mol.get_decl(*resolved_id);
            let name = decl.name.to_string();
            return format!("{}{}", name.len(), name);
          } else {
            todo!() // Temporary fallback for complex types
          }
        }
        "v".to_string()
      }
      
      _ => todo!(),
    }
  }

}

impl Mangler for QwMangler {

  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<AstId>, ret_ty: Option<AstId>, arg_tys: &[AstId], ast_mol: &Module) -> String {
    let mut s = String::from("_qw_");
    let mut last_path = String::new();
    for p in path {
      if let Some(g_idx) = p.find("G") {
        if p[g_idx..].chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
          let base_name = &p[..g_idx];
          let generic_part = &p[g_idx..];
          last_path = format!("{}{}{}", base_name.len(), base_name, generic_part);
          s.push_str(&last_path);
          continue;
        }
      }
      last_path = format!("{}{}", p.len(), p);
      s.push_str(&last_path);
    }
    s.push_str(&format!("{}{}", func_name.len(), func_name));
    
    s.push('v');
    
    // Return-type
    if let Some(r) = ret_ty {
      let rt = Self::mangle_type(r, ast_mol);
      if !last_path.is_empty() {
        s.push_str(&rt.replace(&last_path, "x"));
      } else {
        s.push_str(&rt);
      }
    } else {
      s.push('v');
    }

    // Argument types
    if let Some(st) = self_ty {
      let st_mangled = Self::mangle_type(st, ast_mol);
      if !last_path.is_empty() {
        s.push_str(&st_mangled.replace(&last_path, "x"));
      } else {
        s.push_str(&st_mangled);
      }
    }
    for &arg in arg_tys {
      let arg_mangled = Self::mangle_type(arg, ast_mol);
      if !last_path.is_empty() {
        s.push_str(&arg_mangled.replace(&last_path, "x"));
      } else {
        s.push_str(&arg_mangled);
      }
    }
    
    s
  }

  fn mangle_global(&self, path: &[String], var_name: &str) -> String {
    let mut s = String::from("_qw_");
    for p in path {
      if let Some(g_idx) = p.find("G") {
        if p[g_idx..].chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
          let base_name = &p[..g_idx];
          let generic_part = &p[g_idx..];
          s.push_str(&format!("{}{}{}", base_name.len(), base_name, generic_part));
          continue;
        }
      }
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

