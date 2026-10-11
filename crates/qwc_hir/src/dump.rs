/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fmt;

use owo_colors::OwoColorize;
use qwc_dump::{attr, kw, lit_bool, lit_num, lit_str, name, op, punct, tmpval, ty, write_indent};
use qwc_string_interner::StrInterner;

use crate::{
  AnyId, CID, Const, DefPath, DefPathId, Deps, Expr, ExprId, ExprKind, Item, ItemAttr, ItemId, ItemKind, ItemVis, Krate, Layout, LayoutBy, LayoutKind, SymVis, Thing, ThingId, Type, TypeId, TypeKind, id::{HirId, HirKind, NodeKind, Rng, SpecAny},
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

  pub fn get_dpath(&self, id: DefPathId) -> Option<&'a DefPath> {
    self.get_krate(id.cid()).map(|k| k.get(id))
  }

  pub fn extra_get<T: HirKind>(&self, rng: Rng<T>) -> Vec<HirId<T>> {
    if let Some(krate) = self.get_krate(rng.cid()) {
      krate.extra_get(rng).collect()
    } else {
      vec![]
    }
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
    writeln!(f)?;

    Ok(())
  }
}



// Object
impl DumpHandler for Item {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write_indent(f, indent)?;
    
    // Attributes
    if self.attr.contains(ItemAttr::Entry) {
      write!(f, "{} ", attr("![entry]"))?;
    }
    if self.attr.contains(ItemAttr::C) {
      write!(f, "{} ", attr("![c]"))?;
    }

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
        let items = ctx.extra_get(rng);
        if items.is_empty() {
          write!(f, "{} {{}}", kw("root"))?;
        } else {
          writeln!(f, "{} {{", kw("root"))?;
          for id in items {
            id.dump(ctx, f, indent + 1)?;
            writeln!(f)?;
          }
          write_indent(f, indent)?;
          write!(f, "}}")?;
        }
      }

      ItemKind::NameSpace {rng} => {
        write!(f, "{} ", kw("mod"))?;
        if let Some(name_sid) = self.name {
          write!(f, "{} ", name(ctx.sin.str(name_sid)))?;
        }
        let items = ctx.extra_get(rng);
        if items.is_empty() {
          write!(f, "{{}}")?;
        } else {
          writeln!(f, "{{")?;
          for id in items {
            id.dump(ctx, f, indent + 1)?;
            writeln!(f)?;
          }
          write_indent(f, indent)?;
          write!(f, "}}")?;
        }
      }

      ItemKind::GenericNS { rng, args } => {
        write!(f, "{} ", kw("generic"))?;
        if let Some(name_sid) = self.name {
          write!(f, "{} ", name(ctx.sin.str(name_sid)))?;
        }
        write!(f, "{}", punct("<"))?;
        let mut first = true;
        for id in ctx.extra_get(args) {
          if !first { write!(f, "{}", punct(", "))?; }
          first = false;
          let thing = ctx.get_thing(id);
          match thing {
            Some(Thing::NamedType(name_sid, kind)) => {
              write!(f, "{}", name(ctx.sin.str(*name_sid)))?;
              // Only print bound if it's not the default primitive `type`
              let is_default_type = ctx.get_type(*kind).map(|t| match t.kind {
                TypeKind::GenericSelfType => {
                  let path_name = ctx.get_krate(t.path.cid()).and_then(|k| match k.get(t.path) {
                    DefPath::Path { name, .. } => Some(ctx.sin.str(*name)),
                    _ => None,
                  });
                  path_name == Some("type")
                }
                _ => false,
              }).unwrap_or(false);

              if !is_default_type {
                write!(f, "{} ", punct(":"))?;
                kind.dump(ctx, f, indent)?;
              }
            }
            _ => {
              id.dump(ctx, f, indent)?;
            }
          }
        }
        write!(f, "{}", punct(">"))?;
        let items = ctx.extra_get(rng);
        if items.is_empty() {
          write!(f, " {{}}")?;
        } else {
          writeln!(f, " {{")?;
          for id in items {
            id.dump(ctx, f, indent + 1)?;
            writeln!(f)?;
          }
          write_indent(f, indent)?;
          write!(f, "}}")?;
        }
      }

      ItemKind::Using { kind } => {
        write!(f, "{} ", kw("using"))?;
        if let Some(name_sid) = self.name {
          write!(f, "{} ", name(ctx.sin.str(name_sid)))?;
        }
        write!(f, "{} ", op("="))?;
        kind.dump(ctx, f, indent)?;
        write!(f, "{}", punct(";"))?;
      }

      ItemKind::Variable { kind, expr, ism } => {
        let kw_label = if ism { kw("var") } else { kw("let") };
        write!(f, "{} ", kw_label)?;
        if let Some(name_sid) = self.name {
          write!(f, "{}{}", name(ctx.sin.str(name_sid)), punct(": "))?;
        }
        kind.dump(ctx, f, indent)?;
        write!(f, " {} ", op("="))?;
        expr.dump(ctx, f, indent)?;
        write!(f, "{}", punct(";"))?;
      }

      ItemKind::Function { kind, expr } => {
        write!(f, "{} ", kw("fun"))?;
        if let Some(name_sid) = self.name {
          write!(f, "{}{}", name(ctx.sin.str(name_sid)), punct(": "))?;
        }
        kind.dump(ctx, f, indent)?;
        write!(f, " ")?;
        expr.dump(ctx, f, indent)?;
      }

      ItemKind::Task { kind, expr } => {
        write!(f, "{} ", kw("task"))?;
        if let Some(name_sid) = self.name {
          write!(f, "{}{}", name(ctx.sin.str(name_sid)), punct(": "))?;
        }
        kind.dump(ctx, f, indent)?;
        write!(f, " ")?;
        expr.dump(ctx, f, indent)?;
      }

      ItemKind::Impl { type_ty, trait_ty, methods } => {
        write!(f, "{} ", kw("impl"))?;
        type_ty.dump(ctx, f, indent)?;
        
        if let Some(iface) = trait_ty {
          write!(f, " {} ", punct(":"))?;
          iface.dump(ctx, f, indent)?;
        }
        
        let methods_list = ctx.extra_get(methods);
        if methods_list.is_empty() {
          write!(f, " {{}}")?;
        } else {
          writeln!(f, " {{")?;
          for id in methods_list {
            id.dump(ctx, f, indent + 1)?;
            writeln!(f)?;
          }
          write_indent(f, indent)?;
          write!(f, "}}")?;
        }
      }
    }

    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    if self.attr.contains(crate::TypeAttr::C) {
      write!(f, "{} ", attr("![c]"))?;
    }

    match self.kind {
      // Generic
      TypeKind::GenericType{idx} => write!(f, "{}", ty(format!("<{}>", idx)))?,

      TypeKind::GenericRaw{generic, kind} => {
        write!(f, "{}<", punct("raw_generic"))?;
        let it = ctx.get_item(generic);
        if let Some(item) = it && let Some(name_sid) = item.name {
          write!(f, "{}, ", name(ctx.sin.str(name_sid)))?;
        }
        kind.dump(ctx, f, indent)?;
        write!(f, ">")?;
      }

      TypeKind::GenericSelfType => {
        let is_type = ctx.get_krate(self.path.cid()).map(|k| match k.get(self.path) {
          DefPath::Path { name, .. } => ctx.sin.str(*name) == "type",
          _ => false,
        }).unwrap_or(false);
        if is_type {
          write!(f, "{}", ty("type"))?;
        } else {
          write!(f, "{}", ty("Self"))?;
        }
      }
      
      // Basic
      TypeKind::Error => write!(f, "{}", "{error}".red().bold())?,
      TypeKind::Unit => write!(f, "{}", ty("()"))?,
      TypeKind::Never => write!(f, "{}", ty("!"))?,


      // Primitive
      TypeKind::Bit(bits) => write!(f, "{}", ty(format!("b{}", bits)))?,
      
      TypeKind::Int(bits, signed) => {
        let prefix = if signed { "i" } else { "u" };
        write!(f, "{}", ty(format!("{}{}", prefix, bits)))?;
      }

      TypeKind::Float(bits) => write!(f, "{}", ty(format!("f{}", bits)))?,

      TypeKind::ArchInt(signed) => {
        let s = if signed { "isize" } else { "usize" };
        write!(f, "{}", ty(s))?;
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

        if ism { write!(f, "{} ", kw("mut"))?; }
        
        sub.dump(ctx, f, indent)?;
      }
      
      TypeKind::Ptr(sub, ism) => {
        write!(f, "{}", op("^"))?;

        if ism { write!(f, "{} ", kw("mut"))?; }
        
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
        write!(f, "{}", kw("struct"))?;
        let items = ctx.extra_get(rng);
        if items.is_empty() {
          write!(f, " {{}}")?;
        } else {
          write!(f, " {{ ")?;
          let mut first = true;
          for id in items {
            if !first { write!(f, "{}", punct(", "))?; }
            first = false;
            id.dump(ctx, f, indent)?;
          }
          write!(f, " }}")?;
        }
      }

      TypeKind::Tuple(rng) => {
        write!(f, "{}", punct("("))?;
        let items = ctx.extra_get(rng);
        let mut first = true;
        for id in items {
          if !first { write!(f, "{}", punct(", "))?; }
          first = false;
          id.dump(ctx, f, indent)?;
        }
        write!(f, "{}", punct(")"))?;
      }


      // Trait
      TypeKind::Trait(methods, types) => {
        write!(f, "{}", kw("trait"))?;
        let t_items = ctx.extra_get(types);
        let m_items = ctx.extra_get(methods);
        if t_items.is_empty() && m_items.is_empty() {
          write!(f, " {{}}")?;
        } else {
          write!(f, " {{ ")?;
          let mut first = true;
          for id in t_items {
            if !first { write!(f, "{}", punct(", "))?; }
            first = false;
            id.dump(ctx, f, indent)?;
          }
          for id in m_items {
            if !first { write!(f, "{}", punct(", "))?; }
            first = false;
            id.dump(ctx, f, indent)?;
          }
          write!(f, " }}")?;
        }
      }

      TypeKind::Iface(rng) => {
        write!(f, "{}", kw("iface"))?;
        let items = ctx.extra_get(rng);
        if items.is_empty() {
          write!(f, " {{}}")?;
        } else {
          write!(f, " {{ ")?;
          let mut first = true;
          for id in items {
            if !first { write!(f, "{}", punct(", "))?; }
            first = false;
            id.dump(ctx, f, indent)?;
          }
          write!(f, " }}")?;
        }
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
        write!(f, "{}", kw("enum"))?;
        let items = ctx.extra_get(rng);
        if items.is_empty() {
          write!(f, " {{}}")?;
        } else {
          write!(f, " {{ ")?;
          let mut first = true;
          for id in items {
            if !first { write!(f, "{}", punct(", "))?; }
            first = false;
            id.dump(ctx, f, indent)?;
          }
          write!(f, " }}")?;
        }
      }


      // Function
      TypeKind::Fun { self_kind, args, ret } => {
        write!(f, "{}{}", ty("fun"), punct("("))?;

        let mut first = true;
        if let Some(self_kind) = self_kind {
          first = false;
          let self_kind_val = ctx.get_type(self_kind);
          match self_kind_val.map(|t| &t.kind) {
            Some(TypeKind::Ref(.., false)) => write!(f, "{}", tmpval("&self"))?,
            Some(TypeKind::Ref(.., true)) => write!(f, "{}", tmpval("&mut self"))?,
            _ => write!(f, "{}", tmpval("self"))?,
          }
        }

        for id in ctx.extra_get(args) {
          if !first { write!(f, "{}", punct(", "))?; }
          first = false;
          id.dump(ctx, f, indent)?;
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
      ExprKind::GenericExpr => write!(f, "{}", ty("<generic expr>"))?,

      ExprKind::GenericRaw{generic, kind} => {
        write!(f, "{}<", punct("raw_generic"))?;
        let it = ctx.get_item(generic);
        if let Some(item) = it && let Some(name_sid) = item.name {
          write!(f, "{}, ", name(ctx.sin.str(name_sid)))?;
        }
        kind.dump(ctx, f, indent)?;
        write!(f, ">")?;
      }

      ExprKind::Error => write!(f, "{}", "{error}".red().bold())?,

      ExprKind::Const(c) => c.dump(ctx, f, indent)?,

      ExprKind::Ref(e) => {
        write!(f, "{}", op("&"))?;
        e.dump(ctx, f, indent)?;
      }

      ExprKind::Deref(e) => {
        write!(f, "{}", op("^"))?;
        e.dump(ctx, f, indent)?;
      }

      ExprKind::GlobalRef(item_id) => {
        let it = ctx.get_item(item_id);
        write!(f, "{}", punct("@"))?;
        if let Some(item) = it {
          if let Some(name_sid) = item.name {
            write!(f, "{}", name(ctx.sin.str(name_sid)))?;
          } else {
            write!(f, "item_{}", item_id.idx())?;
          }
        } else {
          write!(f, "ext_item_{}_{}", item_id.cid().0.get(), item_id.idx())?;
        }
      }

      ExprKind::LocalRef(local) => {
        write!(f, "{}", tmpval(format!("_{}", local)))?;
      }

      ExprKind::Let { local, init } => {
        write!(f, "{} {}{} ", kw("let"), tmpval(format!("_{}", local)), punct(":"))?;
        if let Some(init_expr) = ctx.get_expr(init) {
          init_expr.ety.dump(ctx, f, indent)?;
        }
        write!(f, " {} ", op("="))?;
        init.dump(ctx, f, indent)?;
      }

      ExprKind::Block { stmt, expr } => {
        let stmts = ctx.extra_get(stmt);
        if stmts.is_empty() && expr.is_none() {
          write!(f, "{{}}")?;
        } else {
          writeln!(f, "{{")?;
          for id in stmts {
            write_indent(f, indent + 1)?;
            id.dump(ctx, f, indent + 1)?;
            let ex = ctx.get_expr(id);
            if let Some(ex) = ex && matches!(ex.kind, ExprKind::Block { .. } | ExprKind::Loop { .. } | ExprKind::If { .. }) {
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
        write!(f, "{} ", kw("if"))?;
        cond.dump(ctx, f, indent)?;
        write!(f, " ")?;
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
        let mut first = true;
        for id in ctx.extra_get(args) {
          if !first { write!(f, "{}", punct(", "))?; }
          first = false;
          id.dump(ctx, f, indent)?;
        }
        write!(f, "{}", punct(")"))?;
      }

      ExprKind::BoundSelfMethod { callee, this } => {
        callee.dump(ctx, f, indent)?;
        write!(f, "{}{}{} ", punct("("), tmpval("@self"), punct(":"))?;
        this.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }


      // Field
      ExprKind::Field { target, idx } => {
        target.dump(ctx, f, indent)?;
        write!(f, "{}{}", punct("."), idx)?;
      }

      ExprKind::CombinatedInit { kind, fields } => {
        kind.dump(ctx, f, indent)?;
        let items = ctx.extra_get(fields);
        if items.is_empty() {
          write!(f, " {{}}")?;
        } else {
          write!(f, " {{ ")?;
          let mut first = true;
          for id in items {
            if !first { write!(f, "{}", punct(", "))?; }
            first = false;
            id.dump(ctx, f, indent)?;
          }
          write!(f, " }}")?;
        }
      }


      // Integer
      ExprKind::IntArithmetic { op: aop, flg, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntArithmeticOp::*;
        use crate::IntArithmeticFlg::*;
        
        let op_ch = match aop { Add => '+', Sub => '-', Mul => '*', Div => '/', Rem => '%' };
        let flg_str = match flg { Overflow => "", Checked => "?", Saturating => "|" };
        
        write!(f, " {}{} ", op(op_ch), punct(flg_str))?;
        
        rhs.dump(ctx, f, indent)?;
      }

      ExprKind::AssignIntArithmetic { op: aop, flg, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntArithmeticOp::*;
        use crate::IntArithmeticFlg::*;
        
        let op_str = match aop { Add => "+=", Sub => "-=", Mul => "*=", Div => "/=", Rem => "%=" };
        let flg_str = match flg { Overflow => "", Checked => "?", Saturating => "|" };
        
        write!(f, " {}{} ", op(op_str), punct(flg_str))?;
        
        rhs.dump(ctx, f, indent)?;
      }
      
      ExprKind::IntCondition { op: cop, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntConditionOp::*;
        
        let op_str = match cop { Gt => ">", Lt => "<", GtEq => ">=", LtEq => "<=", Eq => "==", Ne => "!=" };
        
        write!(f, " {} ", op(op_str))?;
        
        rhs.dump(ctx, f, indent)?;
      }


      // Floating
      ExprKind::FloatArithmetic { op: fop, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::FloatArithmeticOp::*;
        
        let op_str = match fop { Add => "+", Sub => "-", Mul => "*", Div => "/", Rem => "%" };
        
        write!(f, " {} ", op(op_str))?;
        
        rhs.dump(ctx, f, indent)?;
      }


      // Logic
      ExprKind::BoolLogic { op: bop, lhs, rhs } => {
        lhs.dump(ctx, f, indent)?;
        
        use crate::BoolLogicOp::*;
        
        let op_str = match bop { And => "&&", Or => "||", Xor => "^^" };
        
        write!(f, " {} ", op(op_str))?;
        
        rhs.dump(ctx, f, indent)?;
      }

      ExprKind::BoolNot (val) => {
        write!(f, "{}{}", op("!"), punct("("))?;
        val.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }


      // Cast
      ExprKind::CastToIfaceRef { ref_of_expr, ref_of_type: _, target_iface } => {
        ref_of_expr.dump(ctx, f, indent)?;
        write!(f, " {} {}", kw("as"), op("&"))?;
        target_iface.dump(ctx, f, indent)?;
      }

      ExprKind::CastToTrait { expr, target_trait } => {
        expr.dump(ctx, f, indent)?;
        write!(f, " {} ", kw("as"))?;
        target_trait.dump(ctx, f, indent)?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Thing {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      Thing::Name(name_sid) => {
        write!(f, "{}", name(ctx.sin.str(name_sid)))?;
      }
      
      Thing::NamedType(name_sid, kind) => {
        write!(f, "{}{}", name(ctx.sin.str(name_sid)), punct(": "))?;
        kind.dump(ctx, f, indent)?;
      }
      
      Thing::NamedExpr(name_sid, expr) => {
        write!(f, "{}{}", name(ctx.sin.str(name_sid)), op(" = "))?;
        expr.dump(ctx, f, indent)?;
      }

      Thing::NamedConst(name_sid, cons) => {
        write!(f, "{}{}", name(ctx.sin.str(name_sid)), op(" = "))?;
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
      Const::Str(s) => write!(f, "{}", lit_str(format!("\"{}\"", ctx.sin.str(s))))?,
    }

    Ok(())
  }
}

impl DumpHandler for DefPath {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      DefPath::Root(name_sid) => write!(f, "{}", name(ctx.sin.str(name_sid)))?,
      
      DefPath::Path{base, name: name_sid} => {
        base.dump(ctx, f, indent)?;
        write!(f, "{}", punct("::"))?;
        write!(f, "{}", name(ctx.sin.str(name_sid)))?;
      }

      DefPath::Impl{base, spec} => {
        write!(f, "{}", punct("<"))?;
        base.dump(ctx, f, indent)?;
        write!(f, " {} ", kw("as"))?;
        spec.dump(ctx, f, indent)?;
        write!(f, "{}", punct(">"))?;
      }

      DefPath::Spec{base, spec} => {
        base.dump(ctx, f, indent)?;
        write!(f, "{}", punct("::<"))?;
        spec.dump(ctx, f, indent)?;
        write!(f, "{}", punct(">"))?;
      }

      DefPath::AnonymousTuple(rng) => {
        write!(f, "{}", punct("("))?;
        let mut first = true;
        for id in ctx.extra_get(rng) {
          if !first {
            write!(f, "{}", punct(", "))?;
          }
          first = false;
          id.dump(ctx, f, indent)?;
        }
        write!(f, "{}", punct(")"))?;
      }
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
impl_dump_id!(DefPathId, DefPath);

impl DumpHandler for (HirId<SpecAny>, NodeKind) {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Item  => ItemId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Type  => TypeId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Expr  => ExprId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Thing => ThingId::new_from(*self).dump(ctx, f, indent),
      NodeKind::DefPath => DefPathId::new_from(*self).dump(ctx, f, indent),
      NodeKind::Any => write!(f, "<any>"),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(ctx, f, indent)
  }
}
