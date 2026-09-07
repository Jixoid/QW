use std::fmt;

use owo_colors::OwoColorize;
use qwc_ast::AnyId;
use qwc_string_interner::StrInterner;

use crate::{Scope, ScopeKind, ScopeMap};


pub struct Dump<'a> {
  pub scp: &'a ScopeMap,
  pub sin: &'a StrInterner,
  pub root: AnyId,
}

impl<'a> fmt::Display for Dump<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    writeln!(f, "{}", "Scope Dump".cyan().bold())?;

    dump(self, self.scp.get(&self.root).unwrap(), 0, f)?;

    Ok(())
  }
}


fn dump(dmp: &Dump, lscp: &Scope, padd: usize, f: &mut std::fmt::Formatter) -> std::fmt::Result {
  for (&name, (kind, ..)) in &lscp.map {
    match kind {
      ScopeKind::Module(id) => {
        writeln!(f, "{}{} {} {{", " ".repeat(padd), "module".blue().bold(), dmp.sin.str(name).green().bold())?;
        
        dmp.scp.get(&id.to_any()).map(|lscp| dump(dmp, lscp, padd+1, f));
        
        writeln!(f, "{}}}", " ".repeat(padd))?;
      }

      ScopeKind::Type(..) => writeln!(f, "{}{} {}{}", " ".repeat(padd), "type".blue().bold(), dmp.sin.str(name).green().bold(), ";".bright_black())?,
      ScopeKind::Expr(..) => writeln!(f, "{}{} {}{}", " ".repeat(padd), "expr".blue().bold(), dmp.sin.str(name).green().bold(), ";".bright_black())?,

      _ => todo!("{kind:#?}")
    }
  }

  Ok(())
}
