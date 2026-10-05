use std::fmt;

use owo_colors::OwoColorize;
use qwc_dump::{kw, lit_bool, lit_num, name, op, punct, tmpval, ty, write_indent};

use crate::{AnyId, Block, BlokId, Const, Expr, FloatKind, Inst, Krate, Layout, LayoutKind, SymbId, Symbol, SymbolKind, SymbolStat, Terminator, Type, TypeId, TypeKind, Value, ValueId, id::{InstId, MirId, NodeKind, SpecAny}};


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
      writeln!(f, "\n")?;
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
      SymbolKind::Function{ entry, blocks } => {
        write!(f, "{} {}{} ", kw("fun"), name(cre.sym_str(self.name)), punct(":"))?;
        self.ety.dump(cre, f, indent)?;
        writeln!(f, " {{")?;

        for id in cre.extra_get(blocks) {
          let is_entry = id == entry;
          write_indent(f, indent + 1)?;
          write!(f, "{}{}{} ", format!("#{}", id.idx()).purple().bold(), punct(if is_entry { " (entry)" } else { "" }), punct(":"))?;
          id.dump(cre, f, indent + 1)?;
          writeln!(f)?;
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

      SymbolKind::Vmt { size, align, table } => {
        write!(f, "{} {}{} ", kw("vmt"), name(cre.sym_str(self.name)), punct(":"))?;
        self.ety.dump(cre, f, indent)?;
        write!(f, " {} {}", op("="), punct("{"))?;
        write!(
          f,
          " {}: {}, {}: {}, {}: {}",
          name("size"),
          lit_num(size),
          name("align"),
          lit_num(align),
          name("fini"),
          kw("null")
        )?;
        for id in cre.extra_get(table) {
          let it = cre.get(id);
          write!(f, "{}{}", punct(", "), name(format!("@{}", cre.sym_str(it.name))))?;
        }
        write!(f, " {}", punct("}"))?;
      }
    }
    
    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
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
        for id in cre.extra_get(args) {
          if !first { write!(f, "{} ", punct(","))? }
          first = false;

          id.dump(cre, f, indent)?;
        }
        write!(f, "{} {} ", punct(")"), op("->"))?;
        ret.dump(cre, f, indent)?;
      }

      TypeKind::Struct(rng) => {
        write!(f, "{} {{ ", kw("struct"))?;
        let mut first = true;
        for id in cre.extra_get(rng) {
          if !first { write!(f, "{} ", punct(","))? }
          first = false;

          id.dump(cre, f, indent)?;
        }
        write!(f, " }}")?;
      }
    };

    Ok(())
  }
}

impl DumpHandler for Block {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    writeln!(f, "{}", punct("{"))?;

    for id in cre.extra_get(self.insts) {
      let it: &Inst = cre.get(id);
      
      write_indent(f, indent +1)?;
      it.dump(cre, f, indent +1)?;
      writeln!(f)?;
    }

    write_indent(f, indent +1)?;
    self.term.dump(cre, f, indent +1)?;
    writeln!(f)?;
    
    write_indent(f, indent)?;
    write!(f, "{}", punct("}"))?;

    Ok(())
  }
}

impl DumpHandler for Terminator {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self {
      Terminator::Jump(target) => {
        write!(f, "{} {}", kw("jump"), format!("#{}", target.idx()).purple().bold())
      }
      Terminator::Branch { cond, then_bb, else_bb } => {
        write!(f, "{} ", kw("branch"))?;
        cond.dump(cre, f, indent)?;
        write!(f, "{} {}, {}", punct(","), format!("#{}", then_bb.idx()).purple().bold(), format!("#{}", else_bb.idx()).purple().bold())
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
      write!(f, "{} {} ", tmpval(format!("%{}", dest.0)), op("="))?;
    }
    self.kind.dump(cre, f, indent)
  }
}

impl DumpHandler for Expr {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      Expr::Alloca { kind } => {
        write!(f, "{} ", kw("alloca"))?;
        kind.dump(cre, f, indent)?;
      }


      Expr::Store{target, kind, value} => {
        write!(f, "{} ", kw("store"))?;
        kind.dump(cre, f, indent)?;
        
        write!(f, " ")?;
        value.dump(cre, f, indent)?;

        write!(f, "{} ", op(","))?;
        target.dump(cre, f, indent)?;
      }

      Expr::Load{target, kind} => {
        write!(f, "{} ", kw("load"))?;
        kind.dump(cre, f, indent)?;
        write!(f, " ")?;

        target.dump(cre, f, indent)?;
      }


      Expr::Call{callee, args} => {
        write!(f, "{} ", kw("call"))?;
        callee.dump(cre, f, indent)?;
        
        write!(f, " {}", punct("("))?;
        for id in cre.extra_get(args) {
          id.dump(cre, f, indent)?;

          write!(f, "{}", punct(","))?;
        }
        write!(f, "{}", punct(")"))?;
      }


      Expr::Gep { target, kind, idx } => {
        write!(f, "{} ", kw("gep"))?;
        kind.dump(cre, f, indent)?;
        write!(f, " ")?;
        target.dump(cre, f, indent)?;
        write!(f, ", {}", idx)?;
      }

      
      Expr::IntArithmetic{op, flg, flg2, kind, lhs, rhs} => {
        use crate::IntArithmeticOp::*;
        use crate::IntArithmeticFlg::*;
        use crate::IntArithmeticFlg2::*;

        let op = match op { Add => "add", Sub => "sub", Mul => "mul", Div => "div", Rem => "rem" };
        let flg = match flg { Overflow => "", Checked => ".checked", Saturating => ".saturating" };
        let flg2 = match flg2 { Signed => ".i", Unsigned => ".u" };
        write!(f, "{}{}{} ", kw(op), kw(flg2), kw(flg))?;
        kind.dump(cre, f, indent)?;
        write!(f, " ")?;

        lhs.dump(cre, f, indent)?;
        write!(f, "{} ", punct(","))?;
        rhs.dump(cre, f, indent)?;
      }
    
      Expr::IntCondition{op, flg2, kind, lhs, rhs} => {
        use crate::IntConditionOp::*;
        use crate::IntConditionFlg2::*;

        let op = match op { GtEq => ".ge", LtEq => ".le", Gt => ".gt", Lt => ".ls", Eq => ".eq", Ne => ".ne" };
        let flg2 = match flg2 { Signed => ".i", Unsigned => ".u" };
        write!(f, "{}{}{} ", kw("cmp"), kw(flg2), kw(op))?;
        kind.dump(cre, f, indent)?;
        write!(f, " ")?;

        lhs.dump(cre, f, indent)?;
        write!(f, "{} ", punct(","))?;
        rhs.dump(cre, f, indent)?;
      }
    
      Expr::IntLogic{op, kind, lhs, rhs} => {
        use crate::IntLogicOp::*;

        let op = match op { And => "and", Or => "or", Xor => "xor" };
        write!(f, "{} ", kw(op))?;
        kind.dump(cre, f, indent)?;
        write!(f, " ")?;

        lhs.dump(cre, f, indent)?;
        write!(f, "{} ", punct(","))?;
        rhs.dump(cre, f, indent)?;
      }

      Expr::IntUnary{op, kind, val} => {
        use crate::IntUnaryOp::*;

        let op = match op { Not => "not" };
        write!(f, "{} ", kw(op))?;
        kind.dump(cre, f, indent)?;
        write!(f, " ")?;

        val.dump(cre, f, indent)?;
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
      Value::SSA(idx) => write!(f, "{}", tmpval(format!("%{}", idx.0)))?,
      
      Value::Const(it) => it.dump(cre, f, indent)?,

      Value::GlobalRef(it) => {
        let sym: &Symbol = cre.get(it);
        write!(f, "{}", tmpval(format!("@{}", cre.sym_str(sym.name))))?;
      }

      Value::Param(idx) => write!(f, "{}", tmpval(format!("${}", idx)))?,
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

impl_dump_id!(SymbId,  Symbol);
impl_dump_id!(TypeId,  Type);
impl_dump_id!(BlokId,  Block);
impl_dump_id!(InstId,  Inst);
impl_dump_id!(ValueId, Value);

impl DumpHandler for (MirId<SpecAny>, NodeKind) {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Type  => TypeId::new_from(*self).dump(cre, f, indent),
      NodeKind::Symb  => SymbId::new_from(*self).dump(cre, f, indent),
      NodeKind::Blok  => BlokId::new_from(*self).dump(cre, f, indent),
      NodeKind::Inst  => InstId::new_from(*self).dump(cre, f, indent),
      NodeKind::Value => ValueId::new_from(*self).dump(cre, f, indent),
      NodeKind::Any => write!(f, "<any>"),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, cre: &Krate, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(cre, f, indent)
  }
}
