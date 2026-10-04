use std::fmt;

use owo_colors::OwoColorize;
use qwc_ast::AnyId;
use qwc_dump::{kw, name, punct, write_indent};
use qwc_string_interner::StrInterner;

use crate::{Scope, ScopeKind, ScopeKindAst, ScopeKindHir, ScopeMap};


pub struct Dump<'a> {
  pub scp: &'a ScopeMap,
  pub sin: &'a StrInterner,
  pub root: AnyId,
}

impl<'a> fmt::Display for Dump<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "{}", "Scope Dump".cyan().bold())?;

    if let Some(root_scope) = self.scp.get(&self.root) {
      dump(self, root_scope, 0, f)?;
    } else {
      writeln!(f, "  {}", "<empty scope>".bright_black())?;
    }

    Ok(())
  }
}


fn dump(dmp: &Dump, lscp: &Scope, indent: usize, f: &mut fmt::Formatter) -> fmt::Result {
  let mut entries: Vec<_> = lscp.map.iter().collect();
  entries.sort_by_key(|(name_sid, _)| dmp.sin.str(**name_sid));

  for (&name_sid, (item, ..)) in entries {
    let name_str = dmp.sin.str(name_sid);

    match item {
      ScopeKind::Ast(kind) => match kind {
        ScopeKindAst::Module(id) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {} {{", kw("mod"), name(name_str))?;

          if let Some(sub_scope) = dmp.scp.get(&id.to_any()) {
            dump(dmp, sub_scope, indent + 1, f)?;
          }

          write_indent(f, indent)?;
          writeln!(f, "}}")?;
        }

        ScopeKindAst::Type(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {}{}", kw("type"), name(name_str), punct(";"))?;
        }

        ScopeKindAst::TypeParam(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {}{}", kw("type_param"), name(name_str), punct(";"))?;
        }

        ScopeKindAst::Expr(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {}{}", kw("expr"), name(name_str), punct(";"))?;
        }

        ScopeKindAst::ExprParam(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {}{}", kw("param"), name(name_str), punct(";"))?;
        }

        ScopeKindAst::Local(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {}{}", kw("local"), name(name_str), punct(";"))?;
        }
      },

      ScopeKind::Hir(kind) => match kind {
        ScopeKindHir::Module(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {}{} {{", kw("imod"), kw("mod"), name(name_str))?;
          write_indent(f, indent)?;
          writeln!(f, "}}")?;
        }

        ScopeKindHir::Type(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {} {}{}", kw("imod"), kw("type"), name(name_str), punct(";"))?;
        }

        ScopeKindHir::Expr(..) => {
          write_indent(f, indent)?;
          writeln!(f, "{} {} {}{}", kw("imod"), kw("expr"), name(name_str), punct(";"))?;
        }
      }
    }
  }

  Ok(())
}
