use std::fmt;

use owo_colors::OwoColorize;
use qwc_dump::{kw, lit_bool, lit_num, name, op, punct, tmpval, ty, write_indent};

use crate::{
  AnyId, Block, BlokId, Const, Expr, FloatKind, Inst, Krate, Layout, LayoutKind, SymbId, Symbol, SymbolKind, SymbolStat, Terminator, Type, TypeId, TypeKind, Value, id::{InstId, MirId, NodeKind, SpecAny},
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
      writeln!(f, "  {}", punct("<empty crate>"))?;
      return Ok(());
    }

    for symb in self.cre.symbols() {
      symb.dump(self.cre, f, 0)?;
      writeln!(f)?;
    }

    Ok(())
  }
}



// Object
impl DumpHandler for Symbol {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write_indent(f, indent)?;
    self.stat.dump(cre, f, indent)?;

    match self.kind {
      SymbolKind::Function{ entry, blocks, stack } => {
        write!(f, "{} {}{} ", kw("fun"), name(cre.sym_str(self.name)), punct(":"))?;
        self.ety.dump(cre, f, indent)?;
        writeln!(f, " {{")?;

        let mut slot_idx = 0;
        for (id, kind) in cre.extra_get(stack) {
          if kind == NodeKind::Type {
            let ty_id = TypeId::new_from((id, kind));
            write_indent(f, indent + 1)?;
            write!(f, "{}{} ", tmpval(format!("${}", slot_idx)), punct(":"))?;
            ty_id.dump(cre, f, indent + 2)?;
            writeln!(f)?;
            slot_idx += 1;
          }
        }

        for (id, kind) in cre.extra_get(blocks) {
          if kind == NodeKind::Blok {
            let b_id = BlokId::new_from((id, kind));
            let is_entry = b_id == entry;
            write_indent(f, indent + 1)?;
            write!(f, "bb{}{}{} ", b_id.idx(), if is_entry { " (entry)" } else { "" }, punct(":"))?;
            b_id.dump(cre, f, indent + 1)?;
            writeln!(f)?;
          }
        }

        write_indent(f, indent)?;
        write!(f, "}}")?;
      }
      
      SymbolKind::Variable{ism} => {
        let kw_sym = if ism { kw("var") } else { kw("let") };
        write!(f, "{} {}{} ", kw_sym, name(cre.sym_str(self.name)), punct(":"))?;
        self.ety.dump(cre, f, indent)?;
        write!(f, " {} ", op("="))?;
      }
    }
    
    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    self.layout.dump(cre, f, indent)?;

    write!(f, " ")?;
    
    match self.kind {
      TypeKind::Unit => write!(f, "{}", punct("()"))?,

      TypeKind::Bool => write!(f, "{}", ty("bool"))?,
      TypeKind::Ptr => write!(f, "{}", ty("ptr"))?,

      TypeKind::Int(bits, signed) => {
        let prefix = if signed { "i" } else { "u" };
        write!(f, "{}{}", ty(prefix), ty(bits))?;
      }

      TypeKind::Float(kind) => kind.dump(cre, f, indent)?,

      TypeKind::Array(elem, count) => {
        write!(f, "{}", punct("["))?;
        elem.dump(cre, f, indent)?;
        write!(f, "{} {}{}", punct(";"), lit_num(count), punct("]"))?;
      }

      TypeKind::Slice(elem) => {
        write!(f, "{}", punct("["))?;
        elem.dump(cre, f, indent)?;
        write!(f, "{}", punct("]"))?;
      }

      TypeKind::Fun { args, ret } => {
        write!(f, "{}{}", ty("fun"), punct("("))?;
        let mut first = true;
        for (id, kind) in cre.extra_get(args) {
          if !first {
            write!(f, "{} ", punct(","))?;
          }
          first = false;
          if kind == NodeKind::Type {
            TypeId::new_from((id, kind)).dump(cre, f, indent)?;
          } else {
            write!(f, "<unknown>")?;
          }
        }
        write!(f, "{} {} ", punct(")"), op("->"))?;
        ret.dump(cre, f, indent)?;
      }

      TypeKind::Struct(rng) => {
        write!(f, "{} {{ ", kw("struct"))?;
        let mut first = true;
        for (id, kind) in cre.extra_get(rng) {
          if !first {
            write!(f, "{} ", punct(","))?;
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

    for (id, kind) in cre.extra_get(self.insts) {
      let it: &Inst = cre.get(InstId::new_from((id, kind)));
      
      write_indent(f, indent +1)?;
      it.dump(cre, f, indent +1)?;
      writeln!(f)?;
    }

    write_indent(f, indent +1)?;
    self.term.dump(cre, f, indent +1)?;
    writeln!(f)?;
    
    write_indent(f, indent)?;
    write!(f, "}}")?;

    Ok(())
  }
}

impl DumpHandler for Terminator {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self {
      Terminator::Jump(target) => {
        write!(f, "{} bb{}", kw("jump"), target.idx())
      }
      Terminator::Branch { cond, then_bb, else_bb } => {
        write!(f, "{} ", kw("branch"))?;
        cond.dump(cre, f, indent)?;
        write!(f, "{} bb{}, bb{}", punct(","), then_bb.idx(), else_bb.idx())
      }
      Terminator::Return(val) => {
        write!(f, "{}", kw("ret"))?;
        if let Some(val) = val {
          write!(f, " ")?;
          val.dump(cre, f, indent)?;
        }
        Ok(())
      }
      Terminator::Unreachable => {
        write!(f, "{}", "unreachable".red().bold())
      }
    }
  }
}

impl DumpHandler for Inst {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    if let Some(dest) = self.dest {
      write!(f, "{} {} ", dest, op("="))?;
    }
    self.kind.dump(cre, f, indent)
  }
}

impl DumpHandler for Expr {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      Expr::Store{target, kind, value} => {
        write!(f, "{} ", kw("store"))?;
        kind.dump(cre, f, indent)?;
        write!(f, "{} ", punct(","))?;
        value.dump(cre, f, indent)?;
        write!(f, " {} ", op("->"))?;
        target.dump(cre, f, indent)?;
      }

      Expr::Load{target, kind} => {
        write!(f, "{} ", kw("load"))?;
        kind.dump(cre, f, indent)?;
        write!(f, " {} ", op("<-"))?;
        target.dump(cre, f, indent)?;
      }

      Expr::Binary(lhs, rhs) => {
        write!(f, "{} ", kw("binary"))?;
        lhs.dump(cre, f, indent)?;
        write!(f, "{} ", punct(","))?;
        rhs.dump(cre, f, indent)?;
      }
    }
    
    Ok(())
  }
}

impl DumpHandler for Const {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match *self {
      Const::Unit => write!(f, "{}", punct("()"))?,
      Const::Bool(b) => write!(f, "{}", lit_bool(b))?,
      Const::Int(i) => write!(f, "{}", lit_num(i))?,
    }

    Ok(())
  }
}

impl DumpHandler for Value {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      Value::SSA(it) => write!(f, "{}", it)?,
      
      Value::Const(it) => it.dump(cre, f, indent)?,

      Value::GlobalRef(it) => {
        let sym: &Symbol = cre.get(it);
        write!(f, "{}", tmpval(format!("@{}", cre.sym_str(sym.name))))?;
      }

      Value::StackRef(idx) => write!(f, "{}", tmpval(format!("${}", idx)))?,
    }

    Ok(())
  }
}



// Others
impl DumpHandler for SymbolStat {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match self {
      SymbolStat::Private  => write!(f, "{} ", kw("private"))?,
      SymbolStat::Internal => write!(f, "{} ", kw("internal"))?,
      SymbolStat::Export  => write!(f, "{} ", kw("export"))?,
      SymbolStat::Import  => write!(f, "{} ", kw("import"))?,
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
    write!(f, "{}", ty(name))
  }
}

impl DumpHandler for Layout {
  fn dump(&self, _cre: &Krate, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    write!(f, "{}", punct("![layout("))?;
    
    match self.kind() {
      LayoutKind::SST {align, size} => write!(f, "{}{}{}{}{}{}", punct("sst"), punct("("), punct(size), punct(":"), punct(align), punct(")"))?,
      LayoutKind::ZST  => write!(f, "{}", punct("zst"))?,
      LayoutKind::DST {align} => write!(f, "{}{}{}{}", punct("dst"), punct("("), align, punct(")"))?,
      LayoutKind::DSAT => write!(f, "{}", punct("dsat"))?,
    }

    write!(f, "{}", punct(")]"))?;

    Ok(())
  }
}



// IDs
macro_rules! impl_dump_id {
  ($id_ty:ident, $node_ty:ident) => {
    impl DumpHandler for $id_ty {
      fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
        let node: &$node_ty = cre.get(*self);
        node.dump(cre, f, indent)
      }
    }
  };
}

impl_dump_id!(SymbId, Symbol);
impl_dump_id!(TypeId, Type);
impl_dump_id!(BlokId, Block);
impl_dump_id!(InstId, Inst);

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
