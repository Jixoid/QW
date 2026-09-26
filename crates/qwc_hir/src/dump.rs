use std::fmt;

use owo_colors::OwoColorize;
use qwc_dump::{attr, kw, lit_bool, lit_num, name, op, punct, tmpval, ty, write_indent};
use qwc_string_interner::StrInterner;

use crate::{AnyId, Const, Expr, ExprId, ExprKind, Item, ItemId, ItemKind, ItemVis, Krate, Layout, LayoutBy, LayoutKind, SymVis, Type, TypeId, TypeKind, id::{HirId, NodeKind, SpecAny}};


pub trait DumpHandler {
  fn dump(&self, cre: &Krate, sin: &StrInterner, f: &mut fmt::Formatter, indent: usize) -> fmt::Result;
}

pub struct Dump<'a> {
  pub cre: &'a Krate,
  pub sin: &'a StrInterner,
}

impl<'a> fmt::Display for Dump<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let root = match self.cre.root() {
      None => {
        writeln!(f, "{}", "crate does not have a root node!".red().bold())?;
        return Ok(());
      }
      Some(v) => v,
    };

    writeln!(f, "{}", "HIR Crate Dump".cyan().bold())?;
    root.dump(self.cre, self.sin, f, 0)?;

    Ok(())
  }
}



// Object
impl DumpHandler for Item {
  fn dump(&self, cre: &Krate, sin: &StrInterner, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write_indent(f, indent)?;
    
    if let Some(svis) = self.svis {
      match svis {
        SymVis::Export => write!(f, "{} ", attr("![export]"))?,
        SymVis::Import => write!(f, "{} ", attr("![import]"))?,
      }
    }

    match self.vis {
      ItemVis::Public => write!(f, "{} ", kw("pub"))?,
      ItemVis::Private => {},
    }

    match self.kind {
      ItemKind::RootNS {rng} => {
        writeln!(f, "{} {{", kw("root"))?;
        for (id, kind) in cre.extra_get(rng) {
          let id = ItemId::new_from((id, kind));
          
          id.dump(cre, sin, f, indent +1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::NameSpace {rng, name: ns_name} => {
        let name_str = sin.str(ns_name);

        writeln!(f, "{} {} {{", kw("namespace"), name(name_str))?;
        for (id, kind) in cre.extra_get(rng) {
          let id = ItemId::new_from((id, kind));

          id.dump(cre, sin, f, indent +1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::GenericNS { rng } => {
        writeln!(f, "{} {{", kw("generic"))?;
        for (cid, kind) in cre.extra_get(rng) {
          if kind == NodeKind::Item {
            let item_id = ItemId::new_from((cid, kind));
            item_id.dump(cre, sin, f, indent + 1)?;
          }
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }


      ItemKind::Using { kind, name: use_name } => {
        let name_str = sin.str(use_name);
        
        write!(f, "{} {} {} ", kw("using"), name(name_str), punct("="))?;
        kind.dump(cre, sin, f, indent)?;
        writeln!(f, "{}", punct(";"))?;
      }


      ItemKind::Variable { kind, expr, name: var_name, ism } => {
        let kw_label = if ism { kw("var") } else { kw("let") };
        let name_str = sin.str(var_name);
        
        write!(f, "{} {}{} ", kw_label, name(name_str), punct(":"))?;
        kind.dump(cre, sin, f, indent)?;
        write!(f, " {} ", op("="))?;
        expr.dump(cre, sin, f, indent)?;
        writeln!(f, "{}", punct(";"))?;
      }

      ItemKind::Function { kind, expr, name: fn_name } => {
        let name_str = sin.str(fn_name);

        write!(f, "{} {}{} ", kw("fun"), name(name_str), punct(":"))?;
        
        kind.dump(cre, sin, f, indent)?;
        write!(f, " ")?;
        expr.dump(cre, sin, f, indent)?;
        writeln!(f)?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, cre: &Krate, sin: &StrInterner, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    self.layout.dump(cre, sin, f, indent)?;

    write!(f, " ")?;

    match self.kind {
      TypeKind::GenericType => write!(f, "{}", punct("<generic>"))?,
      TypeKind::Unit => write!(f, "{}", punct("()"))?,
      TypeKind::Never => write!(f, "{}", op("!"))?,

      TypeKind::Int(bits, signed) => write!(f, "{}{}", if signed { "i" } else { "u" }, bits)?,

      TypeKind::Float(bits) => write!(f, "f{}", bits)?,

      TypeKind::ArchInt(signed) => {
        let s = if signed { "isize" } else { "usize" };
        write!(f, "{}", s)?;
      }


      TypeKind::Bool => write!(f, "{}", ty("bool"))?,
      

      TypeKind::Struct(rng) => {
        write!(f, "{} {{ ", kw("struct"))?;
        let mut first = true;
        for (id, kind) in cre.extra_get(rng) {
          if !first {
            write!(f, "{} ", punct(","))?;
          }
          first = false;
          if kind == NodeKind::Type {
            TypeId::new_from((id, kind)).dump(cre, sin, f, indent)?;
          } else {
            write!(f, "<unknown>")?;
          }
        }
        write!(f, " }}")?;
      }

      TypeKind::Array(elem, len) => {
        write!(f, "{}", punct("["))?;
        elem.dump(cre, sin, f, indent)?;
        write!(f, "{} ", punct(";"))?;
        len.dump(cre, sin, f, indent)?;
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
            TypeId::new_from((id, kind)).dump(cre, sin, f, indent)?;
          } else {
            write!(f, "<unknown>")?;
          }
        }
        write!(f, "{} {} ", punct(")"), op("->"))?;
        ret.dump(cre, sin, f, indent)?;
      }

      TypeKind::Option(sub) => {
        write!(f, "{}", op("?"))?;
        sub.dump(cre, sin, f, indent)?;
      }

      TypeKind::Ref(sub, ism) => {
        write!(f, "{}", op("&"))?;

        if ism { write!(f, "{}", kw("mut"))? }
        
        sub.dump(cre, sin, f, indent)?;
      }
      
      TypeKind::Ptr(sub, ism) => {
        write!(f, "{}", op("^"))?;

        if ism { write!(f, "{}", kw("mut"))? }
        
        sub.dump(cre, sin, f, indent)?;
      }

      TypeKind::Slice(sub) => {
        write!(f, "{}", punct("["))?;
        sub.dump(cre, sin, f, indent)?;
        write!(f, "{}", punct("]"))?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Expr {
  fn dump(&self, cre: &Krate, sin: &StrInterner, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      ExprKind::GenericExpr => write!(f, "{}", punct("<generic expr>"))?,

      ExprKind::Const(c) => match c {
        Const::Unit => write!(f, "{}", punct("()"))?,
        Const::Bool(b) => write!(f, "{}", lit_bool(b))?,
        Const::Int(i) => write!(f, "{}", lit_num(i))?,
      }

      ExprKind::Deref(e) => {
        write!(f, "{}", op("*"))?;
        e.dump(cre, sin, f, indent)?;
      }

      ExprKind::GlobalRef(item_id) => {
        let it: &Item = cre.get(item_id);
        match it.kind {
          ItemKind::Function { name, .. } | ItemKind::Variable { name, .. } => {
            let n = sin.str(name);
            write!(f, "{}{}", tmpval("@"), tmpval(n))?;
          }
          _ => write!(f, "@item_{}", item_id.idx())?,
        }
      }

      ExprKind::LocalRef(local) => {
        write!(f, "_{}", local)?;
      }

      ExprKind::Let { local, init } => {
        write!(f, "{} _{}{} ", kw("let"), local, punct(":"))?;
        let init_expr: &Expr = cre.get(init);
        init_expr.ety.dump(cre, sin, f, indent)?;
        write!(f, " {} ", op("="))?;
        init.dump(cre, sin, f, indent)?;
      }

      ExprKind::Block { stmt, expr } => {
        writeln!(f, "{{")?;
        for (sid, kind) in cre.extra_get(stmt) {
          if kind == NodeKind::Expr {
            let expr_id = ExprId::new_from((sid, kind));
            write_indent(f, indent + 1)?;
            expr_id.dump(cre, sin, f, indent + 1)?;
            let ex: &Expr = cre.get(expr_id);
            if matches!(ex.kind, ExprKind::Block { .. } | ExprKind::Loop { .. }) {
              writeln!(f)?;
            } else {
              writeln!(f, "{}", punct(";"))?;
            }
          }
        }
        if let Some(last) = expr {
          write_indent(f, indent + 1)?;
          last.dump(cre, sin, f, indent + 1)?;
          writeln!(f)?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")?;
      }

      ExprKind::Assign { lhs, rhs } => {
        lhs.dump(cre, sin, f, indent)?;
        write!(f, " {} ", op("="))?;
        rhs.dump(cre, sin, f, indent)?;
      }

      ExprKind::Loop { blok, elsb } => {
        write!(f, "{} ", kw("loop"))?;
        blok.dump(cre, sin, f, indent)?;
        if let Some(el) = elsb {
          write!(f, " {} ", kw("else"))?;
          el.dump(cre, sin, f, indent)?;
        }
      }

      ExprKind::Return(val) => {
        write!(f, "{}", kw("ret"))?;
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(cre, sin, f, indent)?;
        }
      }

      ExprKind::Break(val) => {
        write!(f, "{}", kw("break"))?;
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(cre, sin, f, indent)?;
        }
      }

      ExprKind::Continue => {
        write!(f, "{}", kw("continue"))?;
      }

      ExprKind::If { cond, then, elsb } => {
        write!(f, "{} ", kw("if"))?;
        cond.dump(cre, sin, f, indent)?;
        write!(f, " ")?;
        then.dump(cre, sin, f, indent)?;
        
        if let Some(elsb) = elsb {
          write!(f, " {} ", kw("else"))?;
          elsb.dump(cre, sin, f, indent)?;
        }
      }
    }

    Ok(())
  }
}


// Others
impl DumpHandler for Layout {
  fn dump(&self, _: &Krate, _: &StrInterner, f: &mut fmt::Formatter, _: usize) -> fmt::Result {
    write!(f, "{}", punct("![layout("))?;

    match self.kind() {
      LayoutKind::Static => write!(f, "{}", punct("static, "))?,
      LayoutKind::DST    => write!(f, "{}", punct("dst, "))?,
      LayoutKind::DSAT   => write!(f, "{}", punct("dsat, "))?,
    }

    match self.by() {
      LayoutBy::SYS => write!(f, "{}", punct("sys"))?,
      LayoutBy::QW  => write!(f, "{}", punct("qw"))?,
      LayoutBy::C   => write!(f, "{}", punct("c"))?,
    }

    if self.is_inhabited() {
      write!(f, "{}", punct(",inhabited"))?;
    }

    write!(f, "{}", punct(")]"))
  }
}


// IDs
macro_rules! impl_dump_id {
  ($id_ty:ident, $node_ty:ident) => {
    impl DumpHandler for $id_ty {
      fn dump(&self, cre: &Krate, sin: &StrInterner, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
        let node: &$node_ty = cre.get(*self);
        node.dump(cre, sin, f, indent)
      }
    }
  };
}

impl_dump_id!(ItemId, Item);
impl_dump_id!(TypeId, Type);
impl_dump_id!(ExprId, Expr);

impl DumpHandler for (HirId<SpecAny>, NodeKind) {
  fn dump(&self, cre: &Krate, sin: &StrInterner, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Item => ItemId::new_from(*self).dump(cre, sin, f, indent),
      NodeKind::Type => TypeId::new_from(*self).dump(cre, sin, f, indent),
      NodeKind::Expr => ExprId::new_from(*self).dump(cre, sin, f, indent),
      NodeKind::Any => write!(f, "<any>"),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(cre, sin, f, indent)
  }
}
