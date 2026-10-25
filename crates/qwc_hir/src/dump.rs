/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fmt;

use owo_colors::OwoColorize;
use qwc_dump::{attr, ident, kw, kw2, lit_bool, lit_num, lit_str, name, op, punct, tmpval, ty, write_indent};
use qwc_string_interner::StrInterner;

use crate::{
  AnyId, CID, Const, DefPath, DefPathId, Deps, Expr, ExprId, ExprKind, GetApi, Item, ItemAttr, ItemId, ItemKind, ItemVis, Krate, Layout, LayoutKind, SymVis, Thing, ThingId, Type, TypeId, TypeKind, id::{HirId, HirKind, NodeKind, Rng, SpecAny},
};


#[derive(Clone, Copy)]
pub struct DumpCtx<'a> {
  pub cre: &'a Krate,
  pub deps: &'a Deps,
  pub sin: &'a StrInterner,
}

impl<'a> DumpCtx<'a> {
  pub fn new(cre: &'a Krate, deps: &'a Deps, sin: &'a StrInterner) -> Self {
    Self { cre, deps, sin }
  }

  pub fn get_krate(&self, cid: CID) -> &'a Krate {
    if cid == self.cre.cid() {
      self.cre
    } else {
      self.deps.get(cid)
    }
  }

  pub fn with_krate(&self, cre: &'a Krate) -> Self {
    Self {
      cre,
      deps: self.deps,
      sin: self.sin,
    }
  }

  pub fn get<T: HirKind + GetApi<T>>(&self, id: HirId<T>) -> &'a T {
    self.get_krate(id.cid()).get(id)
  }

  pub fn extra_get<T: HirKind>(&self, rng: Rng<T>) -> Vec<HirId<T>> {
    self.get_krate(rng.cid()).extra_get(rng).collect()
  }
}


pub trait DumpHandler {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result;
}

pub struct Dump<'a> {
  pub cre: &'a Krate,
  pub deps: &'a Deps,
  pub sin: &'a StrInterner,
}

impl<'a> Dump<'a> {
  pub fn new(cre: &'a Krate, deps: &'a Deps, sin: &'a StrInterner) -> Self {
    Self { cre, deps, sin }
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
            writeln!(f, "\n")?;
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
        write!(f, "{}", kw("generic"))?;
        if let Some(name_sid) = self.name {
          write!(f, "{} ", name(ctx.sin.str(name_sid)))?;
        }
        write!(f, "{}", punct("<"))?;
        let mut first = true;
        for id in ctx.extra_get(args) {
          if !first { write!(f, "{}", punct(", "))?; }
          first = false;
          let thing = ctx.get(id);
          match *thing {
            Thing::NamedType(name_sid, kind) => {
              write!(f, "{}", name(ctx.sin.str(name_sid)))?;
              write!(f, "{} ", punct(":"))?;
              kind.dump(ctx, f, indent)?;
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
        ctx.get(type_ty).dump(ctx, f, indent)?;
        
        if let Some(iface) = trait_ty {
          write!(f, " {} ", punct(":"))?;
          ctx.get(iface).dump(ctx, f, indent)?;
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

    use TypeKind::*;
    match self.kind {
      // Generic
      GenericType{generic, idx} => {
        let it = ctx.get(generic);
        if let ItemKind::GenericNS { args, .. } = it.kind {
          let thing = ctx.extra_get(args)[idx as usize];
          if let Thing::Name(name) | Thing::NamedType(name, _) = *ctx.get(thing) {
            write!(f, "{}", ty(ctx.sin.str(name)))?;
          } else {
            write!(f, "{}", ty(format!("<{}>", idx)))?;
          }
        } else {
          write!(f, "{}", ty(format!("<{}>", idx)))?;
        }
      }

      GenericRaw{generic, kind} => {
        write!(f, "{}<", punct("raw_generic"))?;
        let it = ctx.get(generic);
        if let Some(name_sid) = it.name {
          write!(f, "{}, ", name(ctx.sin.str(name_sid)))?;
        }
        kind.dump(ctx, f, indent)?;
        write!(f, ">")?;
      }

      GenericSelfType => write!(f, "{}", ty("Self"))?,

      AssociatedType{trait_ty, idx} => {
        let trait_ty = ctx.get(trait_ty);
        trait_ty.dump(ctx, f, indent)?;

        let sub = match trait_ty.kind {
          TypeKind::ItemTrait{types, ..} => {
            let thing = ctx.extra_get(types)[idx as usize];

            if let Thing::Name(name) | Thing::NamedType(name, _) = *ctx.get(thing) {
               ctx.sin.str(name)
            } else { unreachable!() }
          }

          _ => panic!()
        };

        write!(f, "{}{}", punct("::"), ty(sub))?;
      }
      
      // Basic
      Error => write!(f, "{}", "{error}".red().bold())?,
      Never => write!(f, "{}", punct("!"))?,
      Unit => write!(f, "{}", punct("()"))?,
      
      ItemStruct {path, ..} | ItemTuple {path, ..} |
      ItemTrait {path, ..} | ItemIface {path, ..} |
      ItemEnum {path, ..} | ItemFlag {path, ..} => path.dump(ctx, f, 0)?,

      Int(len, sig) => write!(f, "{}", format!("{}{len}", if sig {"i"} else {"u"}))?,
      ArchInt(sig) => write!(f, "{}", ty(if sig {"isize"} else {"usize"}))?,
      Float(len) => write!(f, "{}{}", ty("f"), ty(len))?,
      Bit(len) => write!(f, "{}{}", ty("b"), ty(len))?,
      Bool => write!(f, "{}", ty("bool"))?,
      Char => write!(f, "{}", ty("char"))?,
      Str => write!(f, "{}", ty("str"))?,
      

      Ref(kind, ism) => {
        write!(f, "{}{}", punct("&"), punct(if ism {"mut "} else {""}))?;
        kind.dump(ctx, f, indent)?;
      }

      Ptr(kind, ism) => {
        write!(f, "{}{}", punct("^"), punct(if ism {"mut "} else {""}))?;
        kind.dump(ctx, f, indent)?;
      }
      

      Range(kind) => {
        write!(f, "{}", ty(".."))?;
        kind.dump(ctx, f, indent)?;
      }

      Option(kind) => {
        write!(f, "{}", ty("?"))?;
        kind.dump(ctx, f, indent)?;
      }


      Meta(kind) => {
        write!(f, "{}", punct("#"))?;
        kind.dump(ctx, f, indent)?;
      }
    

      Vector(kind, len) => {
        write!(f, "{}", punct("["))?;
        kind.dump(ctx, f, indent)?;
        write!(f, " {} ", punct("*"))?;
        len.dump(ctx, f, indent)?;
        write!(f, "{}", punct("]"))?;
      }

      VScale(kind) => {
        write!(f, "{}", punct("["))?;
        kind.dump(ctx, f, indent)?;
        write!(f, " {}", punct("*]"))?;
      }
      

      Array(kind, len) => {
        write!(f, "{}", punct("["))?;
        kind.dump(ctx, f, indent)?;
        write!(f, "{} ", punct(";"))?;
        len.dump(ctx, f, indent)?;
        write!(f, "{}", punct("]"))?;
      }

      Slice(kind) => {
        write!(f, "{}", punct("["))?;
        kind.dump(ctx, f, indent)?;
        write!(f, " {}", punct("]"))?;
      }
    

      Struct(fields) => {
        write!(f, "{}", punct("{"))?;
        
        let mut first = true;
        for id in ctx.extra_get(fields) {
          let Thing::NamedType(name, kind) = *ctx.get(id) else { unreachable!() };

          if !first { write!(f, " {}", punct(","))? }
          first = false;

          write!(f, "{}{} ", ctx.sin.str(name), punct(":"))?;
          kind.dump(ctx, f, indent)?;
        }
        
        write!(f, "{}", punct("}"))?;
      }

      Tuple(fields) => {
        write!(f, "{}", punct("("))?;
        
        let mut first = true;
        for id in ctx.extra_get(fields) {
          if !first { write!(f, " {}", punct(","))? }
          first = false;

          id.dump(ctx, f, indent)?;
        }
        
        write!(f, "{}", punct(")"))?;
      }


      TraitFrom{trait_ty, hidden} => {
        trait_ty.dump(ctx, f, indent)?;
        write!(f, "{}", punct("$("))?;
        hidden.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }


      Fun{self_kind, args, ret} => {
        write!(f, "{}", punct("fun("))?;

        if let Some(self_kind) = self_kind {
          self_kind.dump(ctx, f, indent)?;
        }
        
        let mut first = true;
        for id in ctx.extra_get(args) {
          let Thing::NamedType(name, kind) = *ctx.get(id) else { unreachable!() };

          if !first { write!(f, " {}", punct(","))? }
          first = false;

          write!(f, "{}{} ", ctx.sin.str(name), punct(":"))?;
          kind.dump(ctx, f, indent)?;
        }
        
        write!(f, "{} {} ", punct(")"), punct("->"))?;

        ret.dump(ctx, f, indent)?;
      }

    }

    Ok(())
  }
}

impl DumpHandler for Expr {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      ExprKind::GenericExpr{generic, idx} => {
        let it = ctx.get(generic);
        if let ItemKind::GenericNS { args, .. } = it.kind {
          let thing = ctx.extra_get(args)[idx as usize];
          if let Thing::Name(name) | Thing::NamedType(name, _) = *ctx.get(thing) {
            write!(f, "{}", ty(ctx.sin.str(name)))?;
          } else {
            write!(f, "{}", ty(format!("<generic expr {}>", idx)))?;
          }
        } else {
          write!(f, "{}", ty(format!("<generic expr {}>", idx)))?;
        }
      }

      ExprKind::GenericRaw{generic, kind} => {
        write!(f, "{}<", punct("raw_generic"))?;
        let it = ctx.get(generic);
        if let Some(name_sid) = it.name {
          write!(f, "{}, ", name(ctx.sin.str(name_sid)))?;
        }
        kind.dump(ctx, f, indent)?;
        write!(f, ">")?;
      }

      ExprKind::Error => write!(f, "{}", "{error}".red().bold())?,

      ExprKind::Const(c) => c.dump(ctx, f, indent)?,

      ExprKind::Ref(e, ism) => {
        e.dump(ctx, f, indent)?;
        write!(f, "{}{}", op("&"), kw(if ism {"mut"} else {""}))?;
      }

      ExprKind::Deref(e) => {
        e.dump(ctx, f, indent)?;
        write!(f, "{}", op("^"))?;
      }


      ExprKind::GlobalRef(item_id) => {
        let it = ctx.get(item_id);
        write!(f, "{}", punct("@"))?;
        if let Some(name_sid) = it.name {
          write!(f, "{}", name(ctx.sin.str(name_sid)))?;
        } else {
          write!(f, "item_{}", item_id.idx())?;
        }
      }

      ExprKind::LocalRef(local) => {
        write!(f, "{}", tmpval(format!("_{}", local)))?;
      }

      
      ExprKind::Let { local, init } => {
        write!(f, "{} {}{} ", kw("let"), tmpval(format!("_{}", local)), punct(":"))?;
        let init_expr = ctx.get(init);
        init_expr.ety.dump(ctx, f, indent)?;
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
            let ex = ctx.get(id);
            if matches!(ex.kind, ExprKind::Block { .. } | ExprKind::Loop { .. } | ExprKind::If { .. }) {
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


      // Loop
      ExprKind::Loop { blok, elsb } => {
        write!(f, "{} ", kw2("loop"))?;
        blok.dump(ctx, f, indent)?;
        if let Some(el) = elsb {
          write!(f, " {} ", kw2("else"))?;
          el.dump(ctx, f, indent)?;
        }
      }


      // Route
      ExprKind::Return (val) => {
        write!(f, "{} ", kw2("ret"))?;
        val.dump(ctx, f, indent)?;
      }

      ExprKind::Break (val) => {
        write!(f, "{}", kw2("break"))?;
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(ctx, f, indent)?;
        }
      }

      ExprKind::Continue => {
        write!(f, "{}", kw2("continue"))?;
      }


      // Condition
      ExprKind::If { cond, then, elsb } => {
        write!(f, "{} ", kw2("if"))?;
        cond.dump(ctx, f, indent)?;
        write!(f, " ")?;
        then.dump(ctx, f, indent)?;
        
        if let Some(elsb) = elsb {
          write!(f, " {} ", kw2("else"))?;
          elsb.dump(ctx, f, indent)?;
        }
      }
    

      // TypeOf
      ExprKind::TypeOf { kind } => {
        ctx.get(kind).dump(ctx, f, indent)?;
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
        ctx.get(kind).dump(ctx, f, indent)?;
        let items = ctx.extra_get(fields);
        if items.is_empty() {
          write!(f, "{{}}")?;
        } else {
          write!(f, "{{ ")?;
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
        write!(f, "{}", punct("("))?;
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntArithmeticOp::*;
        use crate::IntArithmeticFlg::*;
        
        let op_ch = match aop { Add => '+', Sub => '-', Mul => '*', Div => '/', Rem => '%' };
        let flg_str = match flg { Overflow => "", Checked => "?", Saturating => "|" };
        
        write!(f, " {}{} ", op(op_ch), punct(flg_str))?;
        
        rhs.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
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
        write!(f, "{}", punct("("))?;
        lhs.dump(ctx, f, indent)?;
        
        use crate::IntConditionOp::*;
        
        let op_str = match cop { Gt => ">", Lt => "<", GtEq => ">=", LtEq => "<=", Eq => "==", Ne => "!=" };
        
        write!(f, " {} ", op(op_str))?;
        
        rhs.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }

      ExprKind::IntNeg(val) => {
        write!(f, "{}", punct("-"))?;
        val.dump(ctx, f, indent)?;
      }


      // Floating
      ExprKind::FloatArithmetic { op: fop, lhs, rhs } => {
        write!(f, "{}", punct("("))?;
        lhs.dump(ctx, f, indent)?;
        
        use crate::FloatArithmeticOp::*;
        
        let op_str = match fop { Add => "+", Sub => "-", Mul => "*", Div => "/", Rem => "%" };
        
        write!(f, " {} ", op(op_str))?;
        
        rhs.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }

      ExprKind::FloatNeg(val) => {
        write!(f, "{}", punct("-"))?;
        val.dump(ctx, f, indent)?;
      }


      // Logic
      ExprKind::BoolLogic { op: bop, lhs, rhs } => {
        write!(f, "{}", punct("("))?;
        lhs.dump(ctx, f, indent)?;
        
        use crate::BoolLogicOp::*;
        
        let op_str = match bop { And => "&&", Or => "||", Xor => "^^" };
        
        write!(f, " {} ", op(op_str))?;
        
        rhs.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }

      ExprKind::BoolNot (val) => {
        write!(f, "{}{}", op("!"), punct("("))?;
        val.dump(ctx, f, indent)?;
        write!(f, "{}", punct(")"))?;
      }


      // Cast
      ExprKind::IntCast(expr) => {
        expr.dump(ctx, f, indent)?;
        write!(f, " {} ", kw("as"))?;
        self.ety.dump(ctx, f, indent)?;
      }

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
        write!(f, "{}", ident(ctx.sin.str(name_sid)))?;
      }
      
      Thing::NamedType(name_sid, kind) => {
        write!(f, "{}{}", ident(ctx.sin.str(name_sid)), punct(": "))?;
        kind.dump(ctx, f, indent)?;
      }
      
      Thing::NamedExpr(name_sid, expr) => {
        write!(f, "{}{}", ident(ctx.sin.str(name_sid)), op(" = "))?;
        expr.dump(ctx, f, indent)?;
      }

      Thing::NamedConst(name_sid, cons) => {
        write!(f, "{}{}", ident(ctx.sin.str(name_sid)), op(" = "))?;
        cons.dump(ctx, f, indent)?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Const {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      Const::Unit => write!(f, "{}", punct("()"))?,
      Const::Bool(b) => write!(f, "{}", lit_bool(b))?,
      
      Const::Str(s) => write!(f, "{}", lit_str(format!("\"{}\"", ctx.sin.str(s))))?,
      Const::Char(c) => write!(f, "{}", lit_str(format!("\'{}\'", c)))?,
      
      Const::U8(i) => write!(f, "{}{}{}{}", punct("{u8}"), punct("("), lit_num(i), punct(")"))?,
      Const::U16(i) => write!(f, "{}{}{}{}", punct("{u16}"), punct("("), lit_num(i), punct(")"))?,
      Const::U32(i) => write!(f, "{}{}{}{}", punct("{u32}"), punct("("), lit_num(i), punct(")"))?,
      Const::U64(i) => write!(f, "{}{}{}{}", punct("{u64}"), punct("("), lit_num(i), punct(")"))?,
      Const::U128(i) => write!(f, "{}{}{}{}", punct("{u128}"), punct("("), lit_num(i), punct(")"))?,

      Const::I8(i) => write!(f, "{}{}{}{}", punct("{i8}"), punct("("), lit_num(i), punct(")"))?,
      Const::I16(i) => write!(f, "{}{}{}{}", punct("{i16}"), punct("("), lit_num(i), punct(")"))?,
      Const::I32(i) => write!(f, "{}{}{}{}", punct("{i32}"), punct("("), lit_num(i), punct(")"))?,
      Const::I64(i) => write!(f, "{}{}{}{}", punct("{i64}"), punct("("), lit_num(i), punct(")"))?,
      Const::I128(i) => write!(f, "{}{}{}{}", punct("{i128}"), punct("("), lit_num(i), punct(")"))?,

      Const::F16(fp) => write!(f, "{}{}{}{}", punct("{f16}"), punct("("), lit_num(fp), punct(")"))?,
      Const::F32(fp) => write!(f, "{}{}{}{}", punct("{f32}"), punct("("), lit_num(fp), punct(")"))?,
      Const::F64(fp) => write!(f, "{}{}{}{}", punct("{f64}"), punct("("), lit_num(fp), punct(")"))?,
      Const::F128(fp) => write!(f, "{}{}{}{}", punct("{f128"), punct("}("), lit_num(fp), punct(")"))?,

      Const::Array(kind, rng) => {
        write!(f, "{}", punct("{["))?;
        kind.dump(ctx, f, indent)?;
        write!(f, "{} {}{}", punct(";"), rng.count(), punct("]}"))?;

        write!(f, "{}", punct("["))?;
        let mut first = true;
        for id in ctx.cre.extra_get(rng) {
          if !first { write!(f, "{} ", punct(","))?; }
          first = false;
          id.dump(ctx, f, indent)?;
        }
        write!(f, "{}", punct("]"))?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for DefPath {
  fn dump(&self, ctx: &DumpCtx, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match *self {
      DefPath::Root(name_sid) => write!(f, "{}", ty(ctx.sin.str(name_sid)))?,
      
      DefPath::Path{base, name: name_sid} => {
        base.dump(ctx, f, indent)?;
        write!(f, "{}", punct("::"))?;
        write!(f, "{}", ty(ctx.sin.str(name_sid)))?;
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
        write!(f, "{}", punct("<"))?;
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
      LayoutKind::Static => write!(f, "{}", punct("static"))?,
      LayoutKind::Meta   => write!(f, "{}", punct("meta"))?,
      LayoutKind::DST    => write!(f, "{}", punct("dst"))?,
      LayoutKind::DSAT   => write!(f, "{}", punct("dsat"))?,
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
        let krate = ctx.get_krate(self.cid());
        let node: &$node_ty = ctx.get(*self);
        let sub_ctx = ctx.with_krate(krate);
        node.dump(&sub_ctx, f, indent)
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
