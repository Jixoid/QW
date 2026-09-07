use std::fmt;

use owo_colors::OwoColorize;

use crate::{
  AnyId, Block, BlokId, FloatKind, Krate, SymbId, Symbol, SymbolKind, SymbolStat, Type, TypeId, Value, id::{MirId, NodeKind, SpecAny, ValuId},
};


pub trait DumpHandler {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result;
}

pub struct Dump<'a> {
  pub cre: &'a Krate,
}

impl<'a> fmt::Display for Dump<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "{}", "MIR Crate Dump".cyan().bold())?;

    if self.cre.symbols_len() == 0 && self.cre.types_len() == 0 {
      writeln!(f, "  {}", "<empty crate>".bright_black())?;
      return Ok(());
    }

    for symb in self.cre.symbols() {
      symb.dump(self.cre, f, 0)?;
      writeln!(f)?;
    }

    Ok(())
  }
}


impl DumpHandler for SymbId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let symb: &Symbol = cre.get(*self);
    symb.dump(cre, f, indent)
  }
}

impl DumpHandler for TypeId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let ty: &Type = cre.get(*self);
    ty.dump(cre, f, indent)
  }
}

impl DumpHandler for ValuId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let ty: &Value = cre.get(*self);
    ty.dump(cre, f, indent)
  }
}

impl DumpHandler for BlokId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let ty: &Block = cre.get(*self);
    ty.dump(cre, f, indent)
  }
}


impl DumpHandler for Symbol {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    self.stat.dump(cre, f, indent)?;

    match self.kind {
      SymbolKind::Function{ blok } => {
        write!(f, " {} {}{} ", "rx".blue().bold(), cre.sym_str(self.name).white().bold(), ":".bright_black())?;
        self.ety.dump(cre, f, indent)?;
        write!(f, " ")?;
        blok.dump(cre, f, indent)?;
      }
      
      SymbolKind::Variable{ism, value} => {
        write!(f, " {} {}{} ", if ism {"rw"} else {"ro"}.blue().bold(), cre.sym_str(self.name).white().bold(), ":".bright_black())?;
        self.ety.dump(cre, f, indent)?;
        write!(f, " {} ", "=".bright_black())?;
        value.dump(cre, f, indent)?;
      }
    }
    
    
    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self {
      Type::Unit => write!(f, "{}", "()".bright_black())?,
      Type::Bool => write!(f, "{}", "bool".yellow())?,

      Type::Ptr(elem) => {
        write!(f, "{}", "*".bright_black())?;
        elem.dump(cre, f, indent)?;
      }

      Type::Int(bits, signed) => {
        let prefix = if *signed { "i" } else { "u" };
        write!(f, "{}{}", prefix.yellow(), bits.to_string().yellow())?;
      }

      Type::Float(kind) => kind.dump(cre, f, indent)?,

      Type::Array(elem, count) => {
        write!(f, "{}", "[".bright_black())?;
        elem.dump(cre, f, indent)?;
        write!(
          f,
          "{}{}{}",
          "; ".bright_black(),
          count.to_string().bright_magenta(),
          "]".bright_black()
        )?;
      }

      Type::Fun { args, ret } => {
        write!(f, "{}", "fun".blue().bold())?;
        write!(f, "{}", "(".bright_black())?;
        let mut first = true;
        for (id, kind) in cre.extra_get(*args) {
          if !first {
            write!(f, "{}", ", ".bright_black())?;
          }
          first = false;
          if kind == NodeKind::Type {
            TypeId::new_from((id, kind)).dump(cre, f, indent)?;
          } else {
            write!(f, "<unknown>")?;
          }
        }
        write!(f, "{}{}", ")".bright_black(), " -> ".blue())?;
        ret.dump(cre, f, indent)?;
      }

      Type::Struct(rng) => {
        write!(f, "{} {{ ", "struct".blue().bold())?;
        let mut first = true;
        for (id, kind) in cre.extra_get(*rng) {
          if !first {
            write!(f, "{}", ", ".bright_black())?;
          }
          first = false;
          if kind == NodeKind::Type {
            TypeId::new_from((id, kind)).dump(cre, f, indent)?;
          } else {
            write!(f, "<unknown>")?;
          }
        }
        write!(f, " }}")?;
      }
    };

    Ok(())
  }
}

impl DumpHandler for Value {
  fn dump(&self, _: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(indent))?;

    match self {
      Value::Unit => write!(f, "unit()")?,

      Value::Bool(v) => write!(f, "bool({v})")?,

      Value::I32(v) => write!(f, "i32({v})")?,
      Value::I64(v) => write!(f, "i64({v})")?,
    }

    Ok(())
  }
}

impl DumpHandler for Block {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write!(f, "{{\n")?;
    
    write!(f, "{}{}{}", " ".repeat(indent+2), "stack".blue().bold(), ": [".bright_black())?;
    let mut first = true;
    for (id, kind) in cre.extra_get(self.stack) {
      if !first { write!(f, "{}", ", ".bright_black())?; }

      first = false;
      if kind == NodeKind::Type {
        TypeId::new_from((id, kind)).dump(cre, f, indent)?;
      } else {
        write!(f, "<unknown>")?;
      }
    }
    write!(f, "{}\n}}", "]".bright_black())?;

    Ok(())
  }
}


impl DumpHandler for SymbolStat {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match self {
      SymbolStat::Export => write!(f, "{}", "export".green().bold()),
      SymbolStat::Import => write!(f, "{}", "import".blue().bold()),
    }
  }
}

impl DumpHandler for FloatKind {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    let name = match self {
      FloatKind::BF16 => "bf16",
      FloatKind::F16 => "f16",
      FloatKind::F32 => "f32",
      FloatKind::F64 => "f64",
      FloatKind::F128 => "f128",
    };
    write!(f, "{}", name.yellow())
  }
}


impl DumpHandler for (MirId<SpecAny>, NodeKind) {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Type  => TypeId::new_from(*self).dump(cre, f, indent),
      NodeKind::Symb  => SymbId::new_from(*self).dump(cre, f, indent),
      NodeKind::Value => ValuId::new_from(*self).dump(cre, f, indent),
      NodeKind::Blok  => BlokId::new_from(*self).dump(cre, f, indent),
      NodeKind::Any => write!(f, "<any>"),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(cre, f, indent)
  }
}
