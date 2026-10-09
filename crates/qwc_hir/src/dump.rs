/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fmt;

use owo_colors::OwoColorize;
use qwc_dump::{attr, kw, lit_bool, lit_num, name, op, punct, tmpval, ty, write_indent};
use qwc_string_interner::StrInterner;

use crate::{
  id::{HirId, NodeKind, SpecAny},
  AnyId, Const, Deps, Expr, ExprId, ExprKind, Item, ItemId, ItemKind, ItemVis, Krate, Layout,
  LayoutBy, LayoutKind, SymVis, Thing, ThingId, Type, TypeId, TypeKind, CID,
};


#[derive(Clone, Copy)]
pub struct DumpCtx<'a> {
  pub cre: &'a Krate,
  pub deps: Option<&'a Deps>,
  pub sin: &'a StrInterner,
}

impl<'a> DumpCtx<'a> {
  pub fn new(cre: &'a Krate, deps: Option<&'a Deps>, sin: &'a StrInterner) -> Self {
    Self { cre, deps, sin }
  }

  pub fn get_krate(&self, cid: CID) -> Option<&'a Krate> {
    if cid == self.cre.cid() {
      Some(self.cre)
    } else if let Some(deps) = self.deps {
      Some(deps.get(cid))
    } else {
      None
    }
  }

  pub fn with_krate(&self, cre: &'a Krate) -> Self {
    Self {
      cre,
      deps: self.deps,
      sin: self.sin,
    }
  }

  pub fn get_type(&self, id: TypeId) -> Option<&'a Type> {
    self.get_krate(id.cid()).map(|k| k.get(id))
  }

  pub fn get_item(&self, id: ItemId) -> Option<&'a Item> {
    self.get_krate(id.cid()).map(|k| k.get(id))
  }

  pub fn get_expr(&self, id: ExprId) -> Option<&'a Expr> {
    self.get_krate(id.cid()).map(|k| k.get(id))
  }

  pub fn get_thing(&self, id: ThingId) -> Option<&'a Thing> {
    self.get_krate(id.cid()).map(|k| k.get(id))
  }
}


pub trait DumpHandler {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result;
}

pub struct Dump<'a> {
  pub cre: &'a Krate,
  pub deps: Option<&'a Deps>,
  pub sin: &'a StrInterner,
}

impl<'a> Dump<'a> {
  pub fn new(cre: &'a Krate, sin: &'a StrInterner) -> Self {
    Self { cre, deps: None, sin }
  }

  pub fn with_deps(cre: &'a Krate, deps: &'a Deps, sin: &'a StrInterner) -> Self {
    Self { cre, deps: Some(deps), sin }
  }
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

    let ctx = DumpCtx {
      cre: self.cre,
      deps: self.deps,
      sin: self.sin,
    };

    writeln!(f, "{}", "HIR Crate Dump".cyan().bold())?;
    root.dump(&ctx, f, 0)?;

    Ok(())
  }
}



// Object
impl DumpHandler for Item {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write_indent(f, indent)?;
    
    match self.vis {
      ItemVis::Public(svis) => {
        match svis {
          SymVis::Export => write!(f, "{} ", attr("![export]"))?,
          SymVis::Import => write!(f, "{} ", attr("![import]"))?,
          SymVis::Internal => {},
        }
        
        write!(f, "{} ", kw("pub"))?;
      }
      ItemVis::Private => {},
    }

    match self.kind {
      ItemKind::RootNS {rng} => {
        writeln!(f, "{} {{", kw("root"))?;
        for id in ctx.cre.extra_get(rng) {
          id.dump(ctx, f, indent +1)?;
          writeln!(f)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::NameSpace {rng, name: ns_name} => {
        let name_str = ctx.sin.str(ns_name);

        writeln!(f, "{} {} {{", kw("namespace"), name(name_str))?;
        for id in ctx.cre.extra_get(rng) {
          id.dump(ctx, f, indent +1)?;
          writeln!(f)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::GenericNS { rng } => {
        writeln!(f, "{} {{", kw("generic"))?;
        for id in ctx.cre.extra_get(rng) {
          id.dump(ctx, f, indent +1)?;
          writeln!(f)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }


      ItemKind::Using { kind, name: use_name } => {
        let name_str = ctx.sin.str(use_name);
        
        write!(f, "{} {} {} ", kw("using"), name(name_str), punct("="))?;
        kind.dump(ctx, f, indent)?;
        writeln!(f, "{}", punct(";"))?;
      }


      ItemKind::Variable { kind, expr, name: var_name, ism } => {
        let kw_label = if ism { kw("var") } else { kw("let") };
        let name_str = ctx.sin.str(var_name);
        
        write!(f, "{} {}{} ", kw_label, name(name_str), punct(":"))?;
        kind.dump(ctx, f, indent)?;
        write!(f, " {} ", op("="))?;
        expr.dump(ctx, f, indent)?;
        writeln!(f, "{}", punct(";"))?;
      }

      ItemKind::Function { kind, expr, name: fn_name } => {
        let name_str = ctx.sin.str(fn_name);

        write!(f, "{} {}{} ", kw("fun"), name(name_str), punct(":"))?;
        
        kind.dump(ctx, f, indent)?;
        write!(f, " ")?;
        expr.dump(ctx, f, indent)?;
        writeln!(f)?;
      }

      ItemKind::Task { kind, expr, name: fn_name } => {
        let name_str = ctx.sin.str(fn_name);

        write!(f, "{} {}{} ", kw("task"), name(name_str), punct(":"))?;
        
        kind.dump(ctx, f, indent)?;
        write!(f, " ")?;
        expr.dump(ctx, f, indent)?;
        writeln!(f)?;
      }

      
      ItemKind::Impl { type_ty, trait_ty, methods } => {
        write!(f, "{} ", kw("impl"))?;
        type_ty.dump(ctx, f, indent)?;
        
        if let Some(iface) = trait_ty {
          write!(f, " {} ", punct(":"))?;
          iface.dump(ctx, f, indent)?;
        }
        
        writeln!(f, " {{")?;
        for id in ctx.cre.extra_get(methods) {
          id.dump(ctx, f, indent +1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      // Generic
      TypeKind::GenericType => write!(f, "{}", punct("<generic>"))?,
      TypeKind::GenericSelfType => write!(f, "{}", punct("Self"))?,
      
      // Basic
      TypeKind::Error => write!(f, "{}", punct("{error}"))?,
      TypeKind::Unit => write!(f, "{}", punct("()"))?,
      TypeKind::Never => write!(f, "{}", op("!"))?,


      // Primitive
      TypeKind::Bit(bits) => write!(f, "b{}", bits)?,
      
      TypeKind::Int(bits, signed) => write!(f, "{}{}", if signed { "i" } else { "u" }, bits)?,

      TypeKind::Float(bits) => write!(f, "f{}", bits)?,

      TypeKind::ArchInt(signed) => {
        let s = if signed { "isize" } else { "usize" };
        write!(f, "{}", s)?;
      }

      TypeKind::Bool => write!(f, "{}", ty("bool"))?,
      
      TypeKind::Str => write!(f, "{}", ty("str"))?,


      // Meta
      TypeKind::Meta(kind) => {
        write!(f, "{}", "#".bright_black())?;
        kind.dump(ctx, f, indent)?;
      }


      // Reference
      TypeKind::Ref(sub, ism) => {
        write!(f, "{}", op("&"))?;

        if ism { write!(f, "{}", kw("mut"))? }
        
        sub.dump(ctx, f, indent)?;
      }
      
      TypeKind::Ptr(sub, ism) => {
        write!(f, "{}", op("^"))?;

        if ism { write!(f, "{}", kw("mut"))? }
        
        sub.dump(ctx, f, indent)?;
      }


      // Vector
      TypeKind::Vector(elem, len) => {
        write!(f, "{}", punct("["))?;
        elem.dump(ctx, f, indent)?;
        write!(f, " {} ", punct("*"))?;
        len.dump(ctx, f, indent)?;
        write!(f, "{}", punct("]"))?;
      }

      TypeKind::VScale(sub) => {
        write!(f, "{}", punct("["))?;
        sub.dump(ctx, f, indent)?;
        write!(f, " {}", punct("*]"))?;
      }


      // Sequentiel
      TypeKind::Array(elem, len) => {
        write!(f, "{}", punct("["))?;
        elem.dump(ctx, f, indent)?;
        write!(f, "{} ", punct(";"))?;
        len.dump(ctx, f, indent)?;
        write!(f, "{}", punct("]"))?;
      }

      TypeKind::Slice(sub) => {
        write!(f, "{}", punct("["))?;
        sub.dump(ctx, f, indent)?;
        write!(f, "{}", punct("]"))?;
      }


      // Combinated
      TypeKind::Struct(rng) => {
        write!(f, "{} {{ ", kw("struct"))?;
        let mut first = true;
        for id in ctx.cre.extra_get(rng) {
          if !first { write!(f, "{} ", punct(","))? }
          first = false;

          id.dump(ctx, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Tuple(rng) => {
        write!(f, "{} {{ ", kw("struct"))?;
        let mut first = true;
        for id in ctx.cre.extra_get(rng) {
          if !first { write!(f, "{} ", punct(","))? }
          first = false;

          id.dump(ctx, f, indent)?;
        }
        write!(f, " }}")?;
      }


      // Trait
      TypeKind::Trait(rng) => {
        write!(f, "{} {{ ", kw("trait"))?;
        let mut first = true;
        for id in ctx.cre.extra_get(rng) {
          if !first { write!(f, "{} ", punct(","))? }
          first = false;

          id.dump(ctx, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Iface(rng) => {
        write!(f, "{} {{ ", kw("iface"))?;
        let mut first = true;
        for id in ctx.cre.extra_get(rng) {
          if !first { write!(f, "{} ", punct(","))? }
          first = false;

          id.dump(ctx, f, indent)?;
        }
        write!(f, " }}")?;
      }


      TypeKind::TraitFrom {trait_ty, hidden} => {
        write!(f, "{}{}", kw("trait"), punct("("))?;
        trait_ty.dump(ctx, f, indent)?;
        write!(f, "{} ", punct(", hidden:"))?;
        hidden.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }


      // Enum
      TypeKind::Enum(rng) => {
        write!(f, "{} {{ ", kw("enum"))?;
        let mut first = true;
        for id in ctx.cre.extra_get(rng) {
          if !first { write!(f, "{} ", punct(","))? }
          first = false;

          id.dump(ctx, f, indent)?;
        }
        write!(f, " }}")?;
      }


      // Function
      TypeKind::Fun { self_kind, args, ret } => {
        write!(f, "{}{}", ty("fun"), punct("("))?;

        if let Some(self_kind) = self_kind {
          let self_kind_val = ctx.get_type(self_kind);
          match self_kind_val.map(|t| &t.kind) {
            Some(TypeKind::Ref(.., false)) => write!(f, "{}", "&self".magenta())?,
            Some(TypeKind::Ref(.., true)) => write!(f, "{}", "&mut self".magenta())?,
            _ => write!(f, "{}", "self".magenta())?,
          }
        }

        for id in ctx.cre.extra_get(args) {
          id.dump(ctx, f, indent)?;
          write!(f, "{} ", punct(","))?;
        }
        
        write!(f, "{} {} ", punct(")"), op("->"))?;
        ret.dump(ctx, f, indent)?;
      }


      // Variant
      TypeKind::Option(sub) => {
        write!(f, "{}", op("?"))?;
        sub.dump(ctx, f, indent)?;
      }

    }

    Ok(())
  }
}

impl DumpHandler for Expr {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      ExprKind::GenericExpr => write!(f, "{}", punct("<generic expr>"))?,
      ExprKind::Error => write!(f, "{}", punct("{error}"))?,

      ExprKind::Const(c) => c.dump(ctx, f, indent)?,

      ExprKind::Ref(e) => {
        e.dump(ctx, f, indent)?;
        write!(f, "{}", op("&"))?;
      }

      ExprKind::Deref(e) => {
        e.dump(ctx, f, indent)?;
        write!(f, "{}", op("^"))?;
      }

      ExprKind::GlobalRef(item_id) => {
        let it = ctx.get_item(item_id);
        match it.map(|i| &i.kind) {
          Some(ItemKind::Function { name, .. } | ItemKind::Variable { name, .. }) => {
            let n = ctx.sin.str(*name);
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
        if let Some(init_expr) = ctx.get_expr(init) {
          init_expr.ety.dump(ctx, f, indent)?;
        }
        write!(f, " {} ", op("="))?;
        init.dump(ctx, f, indent)?;
      }

      ExprKind::Block { stmt, expr } => {
        writeln!(f, "{{")?;
        for id in ctx.cre.extra_get(stmt) {
          write_indent(f, indent + 1)?;
          id.dump(ctx, f, indent + 1)?;
          let ex = ctx.get_expr(id);
          if let Some(ex) = ex && matches!(ex.kind, ExprKind::Block { .. } | ExprKind::Loop { .. }) {
            writeln!(f)?;
          } else {
            writeln!(f, "{}", punct(";"))?;
          }
        }
        if let Some(last) = expr {
          write_indent(f, indent + 1)?;
          last.dump(ctx, f, indent + 1)?;
          writeln!(f)?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")?;
      }

      ExprKind::Assign { lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        write!(f, " {} ", op("="))?;
        rhs.dump(ctx, f, indent)?;
      }

      ExprKind::Loop { blok, elsb } => {
        write!(f, "{} ", kw("loop"))?;
        blok.dump(ctx, f, indent)?;
        if let Some(el) = elsb {
          write!(f, " {} ", kw("else"))?;
          el.dump(ctx, f, indent)?;
        }
      }


      // Route
      ExprKind::Return (val) => {
        write!(f, "{} ", kw("ret"))?;
        val.dump(ctx, f, indent)?;
      }

      ExprKind::Break (val) => {
        write!(f, "{}", kw("break"))?;
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(ctx, f, indent)?;
        }
      }

      ExprKind::Continue => {
        write!(f, "{}", kw("continue"))?;
      }


      // Condition
      ExprKind::If { cond, then, elsb } => {
        write!(f, "{} {}", kw("if"), punct("("))?;
        cond.dump(ctx, f, indent)?;
        write!(f, "{} ", punct(")"))?;
        then.dump(ctx, f, indent)?;
        
        if let Some(elsb) = elsb {
          write!(f, " {} ", kw("else"))?;
          elsb.dump(ctx, f, indent)?;
        }
      }
    

      // TypeOf
      ExprKind::TypeOf { kind } => {
        kind.dump(ctx, f, indent)?;
      }


      // Call
      ExprKind::Call { callee, args } => {
        callee.dump(ctx, f, indent)?;
        write!(f, "{}", punct("("))?;
        for id in ctx.cre.extra_get(args) {
          id.dump(ctx, f, indent)?;
          write!(f, "{}", punct(","))?;
        }
        write!(f, "{}", punct(")"))?;
      }

      ExprKind::BoundSelfMethod { callee, this } => {
        callee.dump(ctx, f, indent)?;
        write!(f, "{}{}{} ", punct("("), "@self".yellow(), punct(":"))?;
        this.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }


      // Field
      ExprKind::Field { target, idx } => {
        target.dump(ctx, f, indent)?;
        write!(f, ".{}", idx)?;
      }

      ExprKind::CombinatedInit { kind, fields } => {
        kind.dump(ctx, f, indent)?;
        write!(f, "{}", "{".bright_black())?;
        for id in ctx.cre.extra_get(fields) {
          id.dump(ctx, f, indent)?;
          write!(f, "{} ", ",".bright_black())?;
        }
        write!(f, "{}", "}".bright_black())?;
      }


      // Integer
      ExprKind::IntArithmetic { op, flg, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntArithmeticOp::*;
        use crate::IntArithmeticFlg::*;
        
        let op = match op { Add => '+', Sub => '-', Mul => '*', Div => '/', Rem => '%' };
        let flg = match flg { Overflow => "", Checked => "?", Saturating => "|" };
        
        write!(f, " {}{} ", punct(op), punct(flg))?;
        
        rhs.dump(ctx, f, indent)?;
      }

      ExprKind::AssignIntArithmetic { op, flg, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntArithmeticOp::*;
        use crate::IntArithmeticFlg::*;
        
        let op = match op { Add => '+', Sub => '-', Mul => '*', Div => '/', Rem => '%' };
        let flg = match flg { Overflow => "", Checked => "?", Saturating => "|" };
        
        write!(f, " {}{}{} ", punct(op), punct(flg), punct("="))?;
        
        rhs.dump(ctx, f, indent)?;
      }
      
      ExprKind::IntCondition { op, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntConditionOp::*;
        
        let op = match op { Gt => ">", Lt => "<", GtEq => ">=", LtEq => "<=", Eq => "==", Ne => "!=" };
        
        write!(f, " {} ", punct(op))?;
        
        rhs.dump(ctx, f, indent)?;
      }


      // Floating
      ExprKind::FloatArithmetic { op, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::FloatArithmeticOp::*;
        
        let op = match op { Add => '+', Sub => '-', Mul => '*', Div => '/', Rem => '%' };
        
        write!(f, " {} ", punct(op))?;
        
        rhs.dump(ctx, f, indent)?;
      }


      // Logic
      ExprKind::BoolLogic { op, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::BoolLogicOp::*;
        
        let op = match op { And => "&&", Or => "||", Xor => "^^" };
        
        write!(f, " {} ", punct(op))?;
        
        rhs.dump(ctx, f, indent)?;
      }

      ExprKind::BoolNot (val) => {
        write!(f, "{}", "!(".bright_black())?;
        val.dump(ctx, f, indent)?;
        write!(f, "{}", ")".bright_black())?;
      }


      // Cast
      ExprKind::CastToIfaceRef { ref_of_expr, ref_of_type: _, target_iface } => {
        ref_of_expr.dump(ctx, f, indent)?;
        write!(f, " {} {} {}", punct("/* to iface ref */"), punct("as"), punct("&"))?;
        target_iface.dump(ctx, f, indent)?;
      }

      ExprKind::CastToTrait { expr, target_trait } => {
        expr.dump(ctx, f, indent)?;
        write!(f, " {} {} ", punct("/* to trait */"), punct("as"))?;
        target_trait.dump(ctx, f, indent)?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Thing {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      Thing::NamedType(name, expr) => {
        write!(f, "{}", ctx.sin.str(name).white())?;
        write!(f, " = ")?;
        expr.dump(ctx, f, indent)?;
      }
      
      Thing::NamedExpr(name, expr) => {
        write!(f, "{}", ctx.sin.str(name).white())?;
        write!(f, " = ")?;
        expr.dump(ctx, f, indent)?;
      }

      Thing::NamedConst(name, cons) => {
        write!(f, "{}", ctx.sin.str(name).white())?;
        write!(f, " = ")?;
        cons.dump(ctx, f, indent)?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Const {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match *self {
      Const::Unit => write!(f, "{}", punct("()"))?,
      Const::Bool(b) => write!(f, "{}", lit_bool(b))?,
      Const::Int(i) => write!(f, "{}", lit_num(i))?,
      Const::Str(s) => write!(f, "{}", format!("\"{}\"", ctx.sin.str(s)).yellow())?,
    }

    Ok(())
  }
}



// Others
impl DumpHandler for Layout {
  fn dump(&self, _: &DumpCtx, f: &mut fmt::Formatter, _: usize) -> fmt::Result {
    write!(f, "{}", punct("![layout("))?;

    match self.kind() {
      LayoutKind::Static => write!(f, "{}", punct("static, "))?,
      LayoutKind::Meta   => write!(f, "{}", punct("meta, "))?,
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
      fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
        if let Some(krate) = ctx.get_krate(self.cid()) {
          let node: &$node_ty = krate.get(*self);
          let sub_ctx = ctx.with_krate(krate);
          node.dump(&sub_ctx, f, indent)
        } else {
          write!(f, "<external {} {:?}>", stringify!($node_ty), self.cid())
        }
      }
    }
  };
}

impl_dump_id!(ItemId, Item);
impl_dump_id!(TypeId, Type);
impl_dump_id!(ExprId, Expr);
impl_dump_id!(ThingId, Thing);

impl DumpHandler for (HirId<SpecAny>, NodeKind) {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Item  => ItemId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Type  => TypeId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Expr  => ExprId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Thing => ThingId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Any => write!(f, "<any>"),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(ctx, f, indent)
  }
}
