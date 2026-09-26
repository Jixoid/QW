use std::fmt;

use owo_colors::OwoColorize;
use qwc_arena::Files;
use qwc_dump::{kw, name, op, punct, write_indent};
use qwc_string_interner::StrInterner;

use crate::{
  AnyId, Attribute, BinaryOp, Expr, ExprId, ExprKind, Field, FieldId, FieldKind, FunAttrs, Item, ItemId, ItemKind, Krate, Patt, PattId, Thing, ThingId, Type, TypeId, TypeKind, UnaryOp, Visibility, attrs::AttrKind, id::{AstId, NodeKind, SpecAny},
};


trait DumpHandler {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result;
}

pub struct Dump<'a> {
  pub cre: &'a Krate,
  pub sin: &'a StrInterner,
  pub far: &'a Files,
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

    writeln!(f, "{}", "AST Crate Dump".cyan().bold())?;
    root.dump(self.cre, self.sin, self.far, f, 0)?;
    
    Ok(())
  }
}



// Object
impl DumpHandler for Item {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write_indent(f, indent)?;
    self.vis.dump(cre, sin, far, f, indent)?;

    match self.kind {
      ItemKind::Let {kind, value, ism} => {
        let kw_sym = if ism { kw("var") } else { kw("let") };
        
        let name_str = self.name.map(|n| n.str(far)).unwrap();
        
        write!(f, "{} {}", kw_sym, name(name_str))?;
        
        if let Some(ty) = kind {
          write!(f, "{} ", punct(":"))?;
          ty.dump(cre, sin, far, f, indent)?;
        }
        
        write!(f, " {} ", op("="))?;
        value.dump(cre, sin, far, f, indent)?;
        
        writeln!(f, "{}", punct(";"))?;
      }

      ItemKind::Fun {kind, blok} => {
        write!(f, "{} ", "fun".blue().bold())?;

        if let Some(name) = self.name {
          write!(f, "{}", name.str(far).yellow().bold())?;
        }
        
        kind.dump(cre, sin, far, f, indent)?;
        if let Some(b) = blok {
          write!(f, " ")?;
          b.dump(cre, sin, far, f, indent)?;
          writeln!(f)?;
        } else {
          writeln!(f, "{}", ";".bright_black())?;
        }
      }

      ItemKind::Using(ty) => {
        let name = self.name.map(|n| n.str(far)).unwrap();
        write!(f, "{} {} {} ", "using".blue().bold(), name.yellow().bold(), "=".bright_black())?;
        ty.dump(cre, sin, far, f, indent)?;
        writeln!(f, "{}", ";".bright_black())?;
      }

      ItemKind::ItemTy(ty) => {
        let ty_obj: &Type = cre.get(ty);
        match ty_obj.kind {
          TypeKind::Struct(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "struct".blue().bold(), name.yellow().bold())?;
            for field_id in cre.extra_get(rng) {
              field_id.dump(cre, sin, far, f, indent + 1)?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Iface(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "iface".blue().bold(), name.yellow().bold())?;
            for field_id in cre.extra_get(rng) {
              field_id.dump(cre, sin, far, f, indent + 1)?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Trait(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "trait".blue().bold(), name.yellow().bold())?;
            for field_id in cre.extra_get(rng) {
              field_id.dump(cre, sin, far, f, indent + 1)?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Enum(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "enum".blue().bold(), name.yellow().bold())?;
            for thing_id in cre.extra_get(rng) {
              write_indent(f, indent + 1)?;
              thing_id.dump(cre, sin, far, f, indent + 1)?;
              writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Flags(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "flags".blue().bold(), name.yellow().bold())?;
            for thing_id in cre.extra_get(rng) {
              write_indent(f, indent + 1)?;
              thing_id.dump(cre, sin, far, f, indent + 1)?;
              writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Variant(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "variant".blue().bold(), name.yellow().bold())?;
            for thing_id in cre.extra_get(rng) {
              write_indent(f, indent + 1)?;
              thing_id.dump(cre, sin, far, f, indent + 1)?;
              writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          _ => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            write!(f, "{} {} = ", "type".blue().bold(), name.yellow().bold())?;
            ty.dump(cre, sin, far, f, indent)?;
            writeln!(f, ";")?;
          }
        }
      }


      // Krate
      ItemKind::Krate(rng) => {
        writeln!(f, "{} {{", "crate".blue().bold())?;

        for item_id in cre.extra_get(rng) {
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }

        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }


      // Module
      ItemKind::Module(rng) => {
        let name = self.name.map(|n| n.str(far)).unwrap();
        writeln!(f, "{} {} {{", "mod".blue().bold(), name.yellow().bold())?;

        for item_id in cre.extra_get(rng) {
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }

        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::ModuleUnloaded => panic!(),

      ItemKind::ModuleFile(rng, fid) => {
        let name = self.name.map(|n| n.str(far)).unwrap();

        writeln!(
          f,
          "{} {} {{ {}",
          "mod".blue().bold(),
          name.yellow().bold(),

          format!("/* fid: {}, fpath: \"{}\" */", fid, far.get(fid).fpath()).bright_black()
        )?;

        for item_id in cre.extra_get(rng) {
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }

        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }


      ItemKind::Generic { params, reqs, ctn } => {
        write!(f, "{}{}", "generic".blue().bold(), "<".bright_black())?;
        let mut first = true;
        for type_id in cre.extra_get(params) {
          if !first {
            write!(f, "{} ", ",".bright_black())?;
          }
          first = false;
          type_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "{}", ">".bright_black())?;

        if !reqs.is_empty() {
          write!(f, " {} ", "requires".blue())?;
          for type_id in cre.extra_get(reqs) {
            type_id.dump(cre, sin, far, f, indent)?;
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        writeln!(f, " {{")?;
        for item_id in cre.extra_get(ctn) {
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::Impl { type_ty, trait_ty, ctn } => {
        write!(f, "{} ", "impl".blue())?;
        type_ty.dump(cre, sin, far, f, indent)?;
        if let Some(trt) = trait_ty {
          write!(f, " : ")?;
          trt.dump(cre, sin, far, f, indent)?;
        }
        writeln!(f, " {{")?;
        for item_id in cre.extra_get(ctn) {
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::Import(rng) => {
        write!(f, "{} ", "use".blue().bold())?;
        let mut first = true;
        for thing_id in cre.extra_get(rng) {
          if !first {
            write!(f, "{}", "::".bright_black())?;
          }
          first = false;
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        writeln!(f, "{}", ";".bright_black())?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Type {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      TypeKind::Type()  => write!(f, "{}", "type".green().bold())?,
      TypeKind::SelfT() => write!(f, "{}", "Self".green().bold())?,

      TypeKind::Nick(name) => write!(f, "{}", name.str(far).green().bold())?,

      TypeKind::Path(rng) => {
        let mut first = true;
        for type_id in cre.extra_get(rng) {
          if !first {
            write!(f, "{}", "::".bright_black())?;
          }
          first = false;
          type_id.dump(cre, sin, far, f, indent)?;
        }
      }

      TypeKind::Ptr(sub, ism) => {
        write!(f, "{}{}", "^".bright_black(), if ism { "mut " } else { "" })?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Ref(sub, ism) => {
        write!(f, "{}{}", "&".bright_black(), if ism { "mut " } else { "" })?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Array(sub, len) => {
        write!(f, "{}", "[".bright_black())?;
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, "{} ", ";".bright_black())?;
        len.dump(cre, sin, far, f, indent)?;
        write!(f, "{}", "]".bright_black())?;
      }

      TypeKind::Slice(sub) => {
        write!(f, "{}", "[".bright_black())?;
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, "{}", "]".bright_black())?;
      }

      TypeKind::Vector(sub, len) => {
        write!(f, "{}", "[".bright_black())?;
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", "*".bright_black())?;
        len.dump(cre, sin, far, f, indent)?;
        write!(f, "{}", "]".bright_black())?;
      }

      TypeKind::Unit => {
        write!(f, "{}", "()".bright_black())?;
      }

      TypeKind::Range(sub) => {
        write!(f, "{}", "..".bright_black())?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Option(sub) => {
        write!(f, "{}", "?".bright_black())?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Fail(sub) => {
        write!(f, "{}", "!".bright_black())?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Result { sub, err } => {
        write!(f, "{}", "<".bright_black())?;
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", "!".bright_black())?;
        err.dump(cre, sin, far, f, indent)?;
        write!(f, "{}", ">".bright_black())?;
      }

      TypeKind::Variant(rng) => {
        write!(f, "{} {{ ", "variant".blue().bold())?;
        let mut first = true;
        for thing_id in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Enum(rng) => {
        write!(f, "{} {{ ", "enum".blue().bold())?;
        let mut first = true;
        for thing_id in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Flags(rng) => {
        write!(f, "{} {{ ", "flags".blue().bold())?;
        let mut first = true;
        for thing_id in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Struct(rng) => {
        writeln!(f, "{} {{", "struct".blue().bold())?;
        for field_id in cre.extra_get(rng) {
          field_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")?;
      }

      TypeKind::Tuple(rng) => {
        write!(f, "{}", "(".bright_black())?;
        let mut first = true;
        for field_id in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          field_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "{}", ")".bright_black())?;
      }

      TypeKind::Iface(rng) => {
        writeln!(f, "{} {{", "iface".blue())?;
        for field_id in cre.extra_get(rng) {
          field_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")?;
      }

      TypeKind::Trait(rng) => {
        writeln!(f, "{} {{", "trait".blue().bold())?;
        for field_id in cre.extra_get(rng) {
          field_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")?;
      }

      TypeKind::Fun { args, ret, attr } => {
        write!(f, "(")?;
        let mut first = true;
        for thing_id in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;

        if attr & FunAttrs::Static as u8 != 0 {
          write!(f, " {}", "static".blue())?;
        }
        if attr & FunAttrs::Const as u8 != 0 {
          write!(f, " {}", "const".blue())?;
        }
        if attr & FunAttrs::Pure as u8 != 0 {
          write!(f, " {}", "pure".blue())?;
        }

        if let Some(r) = ret {
          write!(f, " -> ")?;
          r.dump(cre, sin, far, f, indent)?;
        }
      }

      TypeKind::Init { args, attr } => {
        write!(f, "(")?;
        let mut first = true;
        for thing_id in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;

        if attr & FunAttrs::Static as u8 != 0 {
          write!(f, " {}", "static".blue())?;
        }
        if attr & FunAttrs::Const as u8 != 0 {
          write!(f, " {}", "const".blue())?;
        }
        if attr & FunAttrs::Pure as u8 != 0 {
          write!(f, " {}", "pure".blue())?;
        }
      }

      TypeKind::Fini { args, attr } => {
        write!(f, "(")?;
        let mut first = true;
        for thing_id in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;

        if attr & FunAttrs::Static as u8 != 0 {
          write!(f, " {}", "static".blue())?;
        }
        if attr & FunAttrs::Const as u8 != 0 {
          write!(f, " {}", "const".blue())?;
        }
        if attr & FunAttrs::Pure as u8 != 0 {
          write!(f, " {}", "pure".blue())?;
        }
      }

      TypeKind::Spec { base, args } => {
        base.dump(cre, sin, far, f, indent)?;
        write!(f, "<")?;
        let mut first = true;
        for id in cre.extra_any_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ">")?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Expr {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      ExprKind::SelfB() => write!(f, "{}", "Self".white())?,
      ExprKind::SelfS() => write!(f, "{}", "self".white())?,

      ExprKind::Nick(name) => {
        write!(f, "{}", name.str(far).white())?;
      }

      ExprKind::Path(rng) => {
        let mut first = true;
        for id in cre.extra_get(rng) {
          if !first {
            write!(f, "::")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Member(rng) => {
        let mut first = true;
        for id in cre.extra_get(rng) {
          if !first {
            write!(f, ".")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Unit => {
        write!(f, "()")?;
      }

      ExprKind::Bool(span, ..) => {
        write!(f, "{}", span.str(far).yellow())?;
      }

      ExprKind::Number(span) => {
        write!(f, "{}", span.str(far).yellow())?;
      }

      ExprKind::String(span) => {
        write!(f, "{}", span.str(far).green())?;
      }

      ExprKind::Tuple(rng) => {
        write!(f, "(")?;
        let mut first = true;
        
        for id in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }

      ExprKind::Array(rng) => {
        write!(f, "[")?;
        let mut first = true;
        
        for id in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "]")?;
      }

      ExprKind::Propagate(rng, count) => {
        write!(f, "[")?;
        let mut first = true;
        
        for id in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "; ")?;
        count.dump(cre, sin, far, f, indent)?;
        write!(f, "]")?;
      }

      ExprKind::Block { label, rng, expr } => {
        if let Some(lbl) = label {
          write!(f, "`{}: ", lbl.str(far).bright_black())?;
        }
        writeln!(f, "{{")?;
        
        for id in cre.extra_get(rng) {
          write_indent(f, indent + 1)?;
          
          id.dump(cre, sin, far, f, indent + 1)?;
          
          if cre.get::<Expr>(id).is_like_blok() {
            writeln!(f)?;
          } else {
            writeln!(f, "{}", punct(";"))?;
          }
        }

        if let Some(last) = expr {
          write_indent(f, indent + 1)?;
          last.dump(cre, sin, far, f, indent + 1)?;
          writeln!(f)?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")?;
      }

      ExprKind::If { cond, then, elsb } => {
        write!(f, "{} ", "if".blue().bold())?;
        cond.dump(cre, sin, far, f, indent)?;
        write!(f, " ")?;
        then.dump(cre, sin, far, f, indent)?;
        let mut curr_elsb = elsb;
        while let Some(el) = curr_elsb {
          let el_expr: &Expr = cre.get(el);
          match el_expr.kind {
            ExprKind::If { cond: ef_cond, then: ef_then, elsb: next_elsb } => {
              write!(f, " {} ", "ef".blue().bold())?;
              ef_cond.dump(cre, sin, far, f, indent)?;
              write!(f, " ")?;
              ef_then.dump(cre, sin, far, f, indent)?;
              curr_elsb = next_elsb;
            }
            _ => {
              write!(f, " {} ", "else".blue().bold())?;
              el.dump(cre, sin, far, f, indent)?;
              break;
            }
          }
        }
      }

      ExprKind::Match { cond, arms } => {
        write!(f, "{} ", "match".blue())?;
        cond.dump(cre, sin, far, f, indent)?;
        writeln!(f, " {{")?;
        
        for id in cre.extra_get(arms) {
          write_indent(f, indent + 1)?;
          
          id.dump(cre, sin, far, f, indent + 1)?;
          writeln!(f, ",")?;
        }
        write_indent(f, indent)?;
        write!(f, "}}")?;
      }

      ExprKind::While { cond, blok, elsb } => {
        write!(f, "{} ", "while".blue())?;
        cond.dump(cre, sin, far, f, indent)?;
        write!(f, " ")?;
        blok.dump(cre, sin, far, f, indent)?;
        if let Some(el) = elsb {
          write!(f, " {} ", "else".blue())?;
          el.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Loop { blok, elsb } => {
        write!(f, "{} ", "loop".blue())?;
        blok.dump(cre, sin, far, f, indent)?;
        if let Some(el) = elsb {
          write!(f, " {} ", "else".blue())?;
          el.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::ForIn { vars, iter, blok, elsb } => {
        write!(f, "{} ", "for".blue())?;
        vars.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", "in".blue())?;
        iter.dump(cre, sin, far, f, indent)?;
        write!(f, " ")?;
        blok.dump(cre, sin, far, f, indent)?;
        if let Some(el) = elsb {
          write!(f, " {} ", "else".blue())?;
          el.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Unary { op: uop, val } => {
        let op_str = match uop {
          UnaryOp::Neg => "-",
          UnaryOp::Poz => "+",
          UnaryOp::Not => "!",
          UnaryOp::Addr => "@",
          UnaryOp::Ref => "&",
          UnaryOp::Try => "?",
          UnaryOp::Deref => "^",
          UnaryOp::Unwrap => "!!",
        };
        match uop {
          UnaryOp::Neg | UnaryOp::Poz | UnaryOp::Not | UnaryOp::Addr => {
            write!(f, "{}", op(op_str))?;
            val.dump(cre, sin, far, f, indent)?;
          }
          UnaryOp::Ref | UnaryOp::Try | UnaryOp::Deref | UnaryOp::Unwrap => {
            val.dump(cre, sin, far, f, indent)?;
            write!(f, "{}", op(op_str))?;
          }
        }
      }

      ExprKind::Binary { op: bop, lhs, rhs } => {
        let op_str = match bop {
          BinaryOp::Add => "+",
          BinaryOp::Sub => "-",
          BinaryOp::Mul => "*",
          BinaryOp::Div => "/",
          BinaryOp::Rem => "%",

          BinaryOp::Eq => "==",
          BinaryOp::Ne => "!=",
          
          BinaryOp::Lt => "<",
          BinaryOp::Gt => ">",
          BinaryOp::LtEq => "<=",
          BinaryOp::GtEq => ">=",
          
          BinaryOp::And => "&&",
          BinaryOp::Or  => "||",
          BinaryOp::Xor => "^^",

          BinaryOp::Shl => "<<",
          BinaryOp::Shr => ">>",

          BinaryOp::Pipe => "|",
        };
        write!(f, "(")?;
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", op(op_str))?;
        rhs.dump(cre, sin, far, f, indent)?;
        write!(f, ")")?;
      }

      ExprKind::Assign { lhs, rhs, op_span: _ } => {
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", op("="))?;
        rhs.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::AssignOp { op: aop, lhs, rhs } => {
        let op_str = match aop {
          BinaryOp::Add => "+=",
          BinaryOp::Sub => "-=",
          BinaryOp::Mul => "*=",
          BinaryOp::Div => "/=",
          BinaryOp::Rem => "%=",

          BinaryOp::And => "&&=",
          BinaryOp::Or  => "||=",
          BinaryOp::Xor => "^^=",
          
          BinaryOp::Shl => "<<=",
          BinaryOp::Shr => ">>=",

          _ => unreachable!("invalid assign op: {:?}", aop),
        };
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", op(op_str))?;
        rhs.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::Exchange { lhs, rhs } => {
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", op("<-"))?;
        rhs.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::Call { callee, args } => {
        callee.dump(cre, sin, far, f, indent)?;
        write!(f, "(")?;
        let mut first = true;
        
        for id in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }

      ExprKind::Index { callee, args } => {
        callee.dump(cre, sin, far, f, indent)?;
        write!(f, "[")?;
        let mut first = true;
        for id in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "]")?;
      }

      ExprKind::Let { item, kind, init, ism } => {
        let kw_sym = if ism { kw("var") } else { kw("let") };
        write!(f, "{} ", kw_sym)?;
        item.dump(cre, sin, far, f, indent)?;
        if let Some(ty) = kind {
          write!(f, "{} ", punct(":"))?;
          ty.dump(cre, sin, far, f, indent)?;
        }
        if let Some(ini) = init {
          write!(f, " {} ", op("="))?;
          ini.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Return { label, val } => {
        write!(f, "{}", "ret".blue().bold())?;
        if let Some(lbl) = label {
          write!(f, " `{}", lbl.str(far).bright_black())?;
        }
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Break { label, val } => {
        write!(f, "{}", "break".blue())?;
        if let Some(lbl) = label {
          write!(f, " `{}", lbl.str(far).bright_black())?;
        }
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Continue { label } => {
        write!(f, "{}", "continue".blue())?;
        if let Some(lbl) = label {
          write!(f, " `{}", lbl.str(far).bright_black())?;
        }
      }

      ExprKind::Die { lvar } => {
        write!(f, "{} {}", "die".red().bold(), lvar.str(far))?;
      }

      ExprKind::Unsafe(sub) => {
        write!(f, "{} ", "unsafe".red())?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::Relaxed(sub) => {
        write!(f, "{} ", "relaxed".yellow())?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::Spec { callee, args } => {
        callee.dump(cre, sin, far, f, indent)?;
        write!(f, "<")?;
        let mut first = true;
        for id in cre.extra_any_get(args) {
          if !first { write!(f, ", ")?; }
          
          first = false;
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ">")?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Patt {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self {
      Patt::One(ident) => {
        write!(f, "{}", name(ident.str(far)))?;
      }
      
      Patt::Under => {
        write!(f, "{}", punct("_"))?;
      }
      
      Patt::Rest => {
        write!(f, "{}", op(".."))?;
      }
      
      Patt::Tuple(rng) => {
        write!(f, "(")?;
        let mut first = true;
        for id in cre.extra_get(*rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }
      
      Patt::Array(rng) => {
        write!(f, "[")?;
        let mut first = true;
        for id in cre.extra_get(*rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;

          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "]")?;
      }
    }

    Ok(())
  }
}

impl DumpHandler for Field {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write_indent(f, indent)?;
    self.vis.dump(cre, sin, far, f, indent)?;

    match self.kind {
      FieldKind::MemberVar {kind} => {
        let name_str = self.name.map(|n| n.str(far)).unwrap();
        
        write!(f, "{}{} ", name(name_str), punct(":"))?;
        kind.dump(cre, sin, far, f, indent)?;
        
        writeln!(f, "{}", punct(";"))?;
      }


      FieldKind::Fun {kind, blok} => {
        write!(f, "{} ", "fun".blue().bold())?;

        if let Some(name) = self.name {
          write!(f, "{}", name.str(far).yellow().bold())?;
        }
        
        kind.dump(cre, sin, far, f, indent)?;
        if let Some(b) = blok {
          write!(f, " ")?;
          b.dump(cre, sin, far, f, indent)?;
          writeln!(f)?;
        } else {
          writeln!(f, "{}", ";".bright_black())?;
        }
      }

      FieldKind::Init {kind, blok, ils} => {
        write!(f, "{} ", "init".blue())?;
        if let Some(name) = self.name {
          write!(f, "{}", name.str(far).yellow().bold())?;
        }

        kind.dump(cre, sin, far, f, indent)?;

        if !ils.is_empty() {
          write!(f, " : ")?;
          let mut first = true;

          for id in cre.extra_get(ils) {
            if !first {
              write!(f, ", ")?;
            }
            first = false;
            let thing: &Thing = cre.get(id);
            match thing {
              Thing::NamedExpr(ident, expr_id) => {
                write!(f, "{}(", ident.str(far).white())?;
                expr_id.dump(cre, sin, far, f, indent)?;
                write!(f, ")")?;
              }
              _ => id.dump(cre, sin, far, f, indent)?,
            }
          }
        }
        if let Some(b) = blok {
          write!(f, " ")?;
          b.dump(cre, sin, far, f, indent)?;
          writeln!(f)?;
        } else {
          writeln!(f, ";")?;
        }
      }

      FieldKind::Fini {kind, blok} => {
        write!(f, "{} ", "fini".blue())?;
        if let Some(name) = self.name {
          write!(f, "{}", name.str(far).yellow().bold())?;
        }
        kind.dump(cre, sin, far, f, indent)?;
        if let Some(b) = blok {
          write!(f, " ")?;
          b.dump(cre, sin, far, f, indent)?;
          writeln!(f)?;
        } else {
          writeln!(f, ";")?;
        }
      }


      FieldKind::ImplIn {trait_ty, ctn} => {
        write!(f, "{} : ", "impl".blue())?;
        trait_ty.dump(cre, sin, far, f, indent)?;
        writeln!(f, " {{")?;

        for id in cre.extra_get(ctn) {
          id.dump(cre, sin, far, f, indent + 1)?;
        }

        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }


      FieldKind::Type(ty) => {
        let name = self.name.map(|n| n.str(far)).unwrap();
        write!(f, "{} {}", "type".blue().bold(), name.yellow().bold())?;

        if let Some(ty) = ty {
          write!(f, " {} ", "=".bright_black())?;
          ty.dump(cre, sin, far, f, indent)?;
          writeln!(f, "{}", ";".bright_black())?;
        } else {
          writeln!(f, "{}", ";".bright_black())?;
        }
      }

    }

    Ok(())
  }
}

impl DumpHandler for Thing {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self {
      Thing::Name(ident) => {
        write!(f, "{}", ident.str(far).white())?;
      }

      Thing::List(rng) => {
        write!(f, "(")?;
        let mut first = true;
        for id in cre.extra_get(*rng) {
          if !first { write!(f, ", ")?; }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }

      Thing::Wildcard => {
        write!(f, "*")?;
      }

      Thing::Crate => {
        write!(f, "{}", "crate".magenta())?;
      }

      Thing::Super => {
        write!(f, "{}", "super".magenta())?;
      }

      Thing::NamedExpr(ident, expr_id) => {
        write!(f, "{}", ident.str(far).white())?;
        write!(f, " = ")?;
        expr_id.dump(cre, sin, far, f, indent)?;
      }

      Thing::NamedType(ident, type_id) => {
        write!(f, "{}", ident.str(far).white())?;
        write!(f, ": ")?;
        type_id.dump(cre, sin, far, f, indent)?;
      }

      Thing::TypeVis(type_id, vis) => {
        vis.dump(cre, sin, far, f, indent)?;
        type_id.dump(cre, sin, far, f, indent)?;
      }

      Thing::NamedTypeVis(ident, vis, type_id) => {
        vis.dump(cre, sin, far, f, indent)?;
        write!(f, "{}: ", ident.str(far).white())?;
        type_id.dump(cre, sin, far, f, indent)?;
      }

      Thing::NamedTypeList(ident, rng) => {
        write!(f, "{}: ", ident.str(far).white())?;
        let mut first = true;
        
        for id in cre.extra_get(*rng) {
          if !first { write!(f, " | ")?; }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
      }

      Thing::NamedExprList(ident, rng) => {
        write!(f, "{}(", ident.str(far).white())?;
        let mut first = true;
        for id in cre.extra_get(*rng) {
          if !first { write!(f, ", ")?; }
          first = false;
          
          id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }

      Thing::MatchArm(pat_expr, body_expr) => {
        pat_expr.dump(cre, sin, far, f, indent)?;
        write!(f, " => ")?;
        body_expr.dump(cre, sin, far, f, indent)?;
      }
    }

    Ok(())
  }
}


// Others
fn dump_attrs(id: impl Into<AnyId>, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
  if let Some(attrs) = cre.get_attached::<Vec<Attribute>>(id) {
    if !attrs.is_empty() {
      write_indent(f, indent)?;
      write!(f, "{}", "![".bright_magenta())?;
      let mut first = true;
      for attr in attrs {
        if !first {
          write!(f, ", ")?;
        }
        first = false;
        attr.dump(cre, sin, far, f, indent)?;
      }
      writeln!(f, "{}", "]".bright_magenta())?;
    }
  }
  Ok(())
}

impl DumpHandler for Attribute {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let key = self.ident;
    
    match &self.kind {
      AttrKind::One() => write!(f, "{}", key.str(far).yellow())?,

      AttrKind::Bin(val) => write!(f, "{}: {}", key.str(far).yellow(), val.str(far).yellow())?,
      
      AttrKind::Set(expr) => {
        write!(f, "{} = ", key.str(far).yellow())?;
        expr.dump(cre, sin, far, f, indent)?;
      }

      AttrKind::List(list) => {
        write!(f, "{}(", key.str(far).yellow())?;
        let mut first = true;
        for attr in list {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          attr.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }
    }
    Ok(())
  }
}

impl DumpHandler for Visibility {
  fn dump(&self, _cre: &Krate, _sin: &StrInterner, _far: &Files, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match self {
      // None
      Visibility::Inherited => Ok(()),
      
      // General
      Visibility::Public  => write!(f, "{} ", "pub".blue().bold()),
      Visibility::Private => write!(f, "{} ", "priv".blue().bold()),
      Visibility::Crate => write!(f, "{} ", "crate".blue().bold()),
      Visibility::Super => write!(f, "{} ", "super".blue().bold()),
      
      // Struct
      Visibility::Protected => write!(f, "{} ", "prot".blue().bold()),

      // Special
      Visibility::Group => write!(f, "{} ", "group".blue().bold()),
    }
  }
}


// IDs
macro_rules! impl_dump_id {
  ($id_ty:ident, $node_ty:ident) => {
    impl DumpHandler for $id_ty {
      fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
        dump_attrs(*self, cre, sin, far, f, indent)?;
        (cre.get(*self) as &$node_ty).dump(cre, sin, far, f, indent)
      }
    }
  };
}

impl_dump_id!(ItemId, Item);
impl_dump_id!(TypeId, Type);
impl_dump_id!(ExprId, Expr);
impl_dump_id!(PattId, Patt);
impl_dump_id!(FieldId, Field);
impl_dump_id!(ThingId, Thing);

impl DumpHandler for (AstId<SpecAny>, NodeKind) {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Item => ItemId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Type => TypeId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Expr => ExprId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Patt => PattId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Field => FieldId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Thing => ThingId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Any => panic!(),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(cre, sin, far, f, indent)
  }
}
