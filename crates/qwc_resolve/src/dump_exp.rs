use std::fmt;

use owo_colors::OwoColorize;
use qwc_dump::{kw, name, punct, write_indent};
use qwc_hir::{AnyId, DumpHandler, Krate};
use qwc_string_interner::StrInterner;

use crate::{Export, ExportKind, ExportMap};


pub struct Dump<'a> {
  pub exp: &'a ExportMap,
  pub cre: &'a Krate,
  pub sin: &'a StrInterner,
  pub root: AnyId,
}

impl<'a> Dump<'a> {
  pub fn new(exp: &'a ExportMap, cre: &'a Krate, sin: &'a StrInterner, root: AnyId) -> Self {
    Self { exp, cre, sin, root }
  }
}

impl<'a> fmt::Display for Dump<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "{}", "Export Dump".cyan().bold())?;

    if let Some(root_export) = self.exp.get(&self.root) {
      if root_export.map.is_empty() {
        writeln!(f, "  {}", "<empty export>".bright_black())?;
      } else {
        dump(self, root_export, 0, f)?;
      }
    } else {
      writeln!(f, "  {}", "<empty export>".bright_black())?;
    }

    Ok(())
  }
}


fn dump(dmp: &Dump, exp: &Export, indent: usize, f: &mut fmt::Formatter) -> fmt::Result {
  let mut entries: Vec<_> = exp.map.iter().collect();
  entries.sort_by_key(|(name_sid, _)| dmp.sin.str(**name_sid));

  for (&name_sid, kind) in entries {
    let name_str = dmp.sin.str(name_sid);

    match kind {
      ExportKind::NameSpace(id) => {
        write_indent(f, indent)?;
        writeln!(f, "{} {} {{", kw("namespace"), name(name_str))?;

        if let Some(sub_export) = dmp.exp.get(&id.to_any()) {
          dump(dmp, sub_export, indent + 1, f)?;
        }

        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ExportKind::Type(..) => {
        write_indent(f, indent)?;
        writeln!(f, "{} {}{}", kw("type"), name(name_str), punct(";"))?;
      }

      ExportKind::Expr(_id, ty_id) => {
        write_indent(f, indent)?;
        write!(f, "{} {}{} ", kw("expr"), name(name_str), punct(":"))?;
        ty_id.dump(dmp.cre, dmp.sin, f, indent)?;
        writeln!(f, "{}", punct(";"))?;
      }
    }
  }

  Ok(())
}
