use std::fmt;

use owo_colors::OwoColorize;

use crate::{
  AnyId, Block, BlokId, Const, Expr, FloatKind, Inst, Krate, Layout, SymbId, Symbol, SymbolKind, SymbolStat, Type, TypeId, TypeKind, id::{InstId, MirId, NodeKind, SpecAny},
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

impl DumpHandler for BlokId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let ty: &Block = cre.get(*self);
    ty.dump(cre, f, indent)
  }
}

impl DumpHandler for InstId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let ty: &Inst = cre.get(*self);
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
      
      SymbolKind::Variable{ism} => {
        write!(f, " {} {}{} ", if ism {"rw"} else {"ro"}.blue().bold(), cre.sym_str(self.name).white().bold(), ":".bright_black())?;
        self.ety.dump(cre, f, indent)?;
        write!(f, " {} ", "=".bright_black())?;
        //value.dump(cre, f, indent)?;
      }
    }
    
    
    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      TypeKind::Unit => write!(f, "{}", "()".green())?,

      TypeKind::Bool => write!(f, "{}", "bool".green())?,
      TypeKind::Ptr => write!(f, "{}", "ptr".green())?,

      TypeKind::Int(bits, signed) => {
        let prefix = if signed { "i" } else { "u" };
        write!(f, "{}{}", prefix.yellow(), bits.to_string().yellow())?;
      }

      TypeKind::Float(kind) => kind.dump(cre, f, indent)?,

      TypeKind::Array(elem, count) => {
        write!(f, "{}", "[".bright_black())?;
        elem.dump(cre, f, indent)?;
        write!(f, "{}{}{}",
          "; ".bright_black(),
          count.to_string().bright_magenta(),
          "]".bright_black()
        )?;
      }

      TypeKind::Fun { args, ret } => {
        write!(f, "{}", "fun".blue().bold())?;
        write!(f, "{}", "(".bright_black())?;
        let mut first = true;
        for (id, kind) in cre.extra_get(args) {
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

      TypeKind::Struct(rng) => {
        write!(f, "{} {{ ", "struct".blue().bold())?;
        let mut first = true;
        for (id, kind) in cre.extra_get(rng) {
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

impl DumpHandler for Block {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    writeln!(f, "{{")?;

    for id in cre.extra_get(self.insts) {
      let it: &Inst = cre.get(InstId::new_from(id));

      write!(f, "{}", " ".repeat(indent+2))?;
      it.dump(cre, f, indent)?;
      writeln!(f)?;
    }

    if let Some(ret) = self.ret {
      writeln!(f, "{}{} {}{}", " ".repeat(indent+2), "ret".blue().bold(), ret, ";".bright_black())?;
    }
    
    write!(f, "}}")?;

    Ok(())
  }
}

impl DumpHandler for Inst {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    if let Some(dest) = self.dest {
      write!(f, "{} {} ", dest, "=".bright_black())?;
    }
    self.kind.dump(cre, f, indent)
  }
}

impl DumpHandler for Expr {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      Expr::Const(cons) => cons.dump(cre, f, indent)?,

      Expr::GlobalRef(symb) => write!(f, "{}{}", "@".bright_yellow(), cre.sym_str((cre.get(symb) as &Symbol).name).bright_yellow())?,

      Expr::Store{target, kind, value} => {
        write!(f, "{} ", "store".blue().bold())?;
        kind.dump(cre, f, indent)?;
        write!(f, " {}", value)?; 
        
        write!(f, ", {} {}", "ptr", target)?;
      }

      _ => todo!("{self:#?}")
    }
    
    Ok(())
  }
}

impl DumpHandler for Const {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match *self {
      Const::Unit => write!(f, "{}", "()".yellow())?,

      _ => todo!("{self:#?}")
    }

    Ok(())
  }
}


impl DumpHandler for SymbolStat {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match self {
      SymbolStat::Normal  => {},
      SymbolStat::Private => write!(f, "{}", "priv".green().bold())?,
      SymbolStat::Export  => write!(f, "{}", "export".green().bold())?,
      SymbolStat::Import  => write!(f, "{}", "import".blue().bold())?,
    }
    Ok(())
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

impl DumpHandler for Layout {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    write!(f, "({}:{})", self.size(), self.size())
  }
}


impl DumpHandler for (MirId<SpecAny>, NodeKind) {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Type  => TypeId::new_from(*self).dump(cre, f, indent),
      NodeKind::Symb  => SymbId::new_from(*self).dump(cre, f, indent),
      NodeKind::Blok  => BlokId::new_from(*self).dump(cre, f, indent),
      NodeKind::Inst  => InstId::new_from(*self).dump(cre, f, indent),
      NodeKind::Any => write!(f, "<any>"),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(cre, f, indent)
  }
}
