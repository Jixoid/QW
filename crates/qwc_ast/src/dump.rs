use std::fmt;

use owo_colors::OwoColorize;
use qwc_arena::Files;
use qwc_string_interner::StrInterner;

use crate::{
  AnyId, Attribute, BinaryOp, Expr, ExprId, ExprKind, FunAttrs, Item, ItemId, ItemKind, Krate, Patt, PattId, Thing, ThingId, Type, TypeId, TypeKind, UnaryOp, Visibility, attrs::AttrKind, id::{AstId, NodeKind, SpecAny},
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



fn write_indent(f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
  for _ in 0..indent { write!(f, "  ")? }
  Ok(())
}

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

impl DumpHandler for ItemId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    dump_attrs(*self, cre, sin, far, f, indent)?;
    //dump_scope(*self, cre, sin, far, f, indent)?;
    let item: &Item = cre.get(*self);
    item.dump(cre, sin, far, f, indent)
  }
}

impl DumpHandler for TypeId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    if let Some(attrs) = cre.get_attached::<Vec<Attribute>>(*self) {
      if !attrs.is_empty() {
        write!(f, "{}", "![".bright_magenta())?;
        let mut first = true;
        for attr in attrs {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          attr.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "{} ", "]".bright_magenta())?;
      }
    }
    let ty: &Type = cre.get(*self);
    ty.dump(cre, sin, far, f, indent)
  }
}

impl DumpHandler for ExprId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    if let Some(attrs) = cre.get_attached::<Vec<Attribute>>(*self) {
      if !attrs.is_empty() {
        write!(f, "{}", "![".bright_magenta())?;
        let mut first = true;
        for attr in attrs {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          attr.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "{} ", "]".bright_magenta())?;
      }
    }
    let expr: &Expr = cre.get(*self);
    expr.dump(cre, sin, far, f, indent)
  }
}

impl DumpHandler for PattId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let patt: &Patt = cre.get(*self);
    patt.dump(cre, sin, far, f, indent)
  }
}

impl DumpHandler for ThingId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    let thing: &Thing = cre.get(*self);
    thing.dump(cre, sin, far, f, indent)
  }
}

impl DumpHandler for (AstId<SpecAny>, NodeKind) {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.1 {
      NodeKind::Item => ItemId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Type => TypeId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Expr => ExprId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Patt => PattId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Thing => ThingId::new_from(*self).dump(cre, sin, far, f, indent),
      NodeKind::Any => write!(f, "<any>"),
    }
  }
}

impl DumpHandler for AnyId {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    (self.id(), self.kind()).dump(cre, sin, far, f, indent)
  }
}

impl DumpHandler for Visibility {
  fn dump(&self, _cre: &Krate, _sin: &StrInterner, _far: &Files, f: &mut fmt::Formatter, _indent: usize) -> fmt::Result {
    match self {
      Visibility::Inherited => Ok(()),
      Visibility::Public => write!(f, "{} ", "pub".blue()),
      Visibility::Private => write!(f, "{} ", "priv".blue()),
      Visibility::Protected => write!(f, "{} ", "prot".blue()),
      Visibility::Crate => write!(f, "{} ", "crate".blue()),
      Visibility::Super => write!(f, "{} ", "super".blue()),
      Visibility::Group => write!(f, "{} ", "group".blue()),
    }
  }
}

impl fmt::Display for Visibility {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Visibility::Inherited => write!(f, "{}", "<inherited>".red()),
      Visibility::Public => write!(f, "{}", "pub".blue()),
      Visibility::Private => write!(f, "{}", "priv".blue()),
      Visibility::Protected => write!(f, "{}", "prot".blue()),
      Visibility::Crate => write!(f, "{}", "crate".blue()),
      Visibility::Super => write!(f, "{}", "super".blue()),
      Visibility::Group => write!(f, "{}", "group".blue()),
    }
  }
}

impl DumpHandler for Item {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    write_indent(f, indent)?;
    self.vis.dump(cre, sin, far, f, indent)?;

    match self.kind {
      ItemKind::Let { kind, value, ism } => {
        let kw = if ism { "var".blue() } else { "let".blue() };
        let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
        write!(f, "{} {}", kw, name.white().bold())?;
        if let Some(ty) = kind {
          write!(f, ": ")?;
          ty.dump(cre, sin, far, f, indent)?;
        }
        
        write!(f, " = ")?;
        value.dump(cre, sin, far, f, indent)?;
        
        writeln!(f, ";")?;
      }

      ItemKind::Member { kind } => {
        let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
        
        write!(f, "{} {}", "mem".blue(), name.white().bold())?;
        
        write!(f, ": ")?;
        kind.dump(cre, sin, far, f, indent)?;
        
        writeln!(f, ";")?;
      }

      ItemKind::Fun { kind, blok } => {
        write!(f, "{} ", "fun".blue())?;
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

      ItemKind::Init { kind, blok, ils } => {
        write!(f, "{} ", "init".blue())?;
        if let Some(name) = self.name {
          write!(f, "{}", name.str(far).yellow().bold())?;
        }
        kind.dump(cre, sin, far, f, indent)?;
        if ils.0 != ils.1 {
          write!(f, " : ")?;
          let mut first = true;
          for (tid, _) in cre.extra_get(ils) {
            if !first {
              write!(f, ", ")?;
            }
            first = false;
            let thing_id = ThingId::new_from((tid, NodeKind::Thing));
            thing_id.dump(cre, sin, far, f, indent)?;
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

      ItemKind::Fini { kind, blok } => {
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

      ItemKind::Using(ty) => {
        let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
        write!(f, "{} {} = ", "using".blue(), name.white().bold())?;
        ty.dump(cre, sin, far, f, indent)?;
        writeln!(f, ";")?;
      }

      ItemKind::ItemTy(ty) => {
        let ty_obj: &Type = cre.get(ty);
        match ty_obj.kind {
          TypeKind::Struct(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "struct".blue(), name.green().bold())?;
            for (cid, _) in cre.extra_get(rng) {
              let item_id = ItemId::new_from((cid, NodeKind::Item));
              item_id.dump(cre, sin, far, f, indent + 1)?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Iface(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "iface".blue(), name.green().bold())?;
            for (cid, _) in cre.extra_get(rng) {
              let item_id = ItemId::new_from((cid, NodeKind::Item));
              item_id.dump(cre, sin, far, f, indent + 1)?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Trait(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "trait".blue(), name.green().bold())?;
            for (cid, _) in cre.extra_get(rng) {
              let item_id = ItemId::new_from((cid, NodeKind::Item));
              item_id.dump(cre, sin, far, f, indent + 1)?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Enum(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "enum".blue(), name.green().bold())?;
            for (cid, _) in cre.extra_get(rng) {
              let thing_id = ThingId::new_from((cid, NodeKind::Thing));
              write_indent(f, indent + 1)?;
              thing_id.dump(cre, sin, far, f, indent + 1)?;
              writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Flags(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "flags".blue(), name.green().bold())?;
            for (cid, _) in cre.extra_get(rng) {
              let thing_id = ThingId::new_from((cid, NodeKind::Thing));
              write_indent(f, indent + 1)?;
              thing_id.dump(cre, sin, far, f, indent + 1)?;
              writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          TypeKind::Variant(rng) => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            writeln!(f, "{} {} {{", "variant".blue(), name.green().bold())?;
            for (cid, _) in cre.extra_get(rng) {
              let thing_id = ThingId::new_from((cid, NodeKind::Thing));
              write_indent(f, indent + 1)?;
              thing_id.dump(cre, sin, far, f, indent + 1)?;
              writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            writeln!(f, "}}")?;
          }
          _ => {
            let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
            write!(f, "{} {} = ", "type".blue(), name.green().bold())?;
            ty.dump(cre, sin, far, f, indent)?;
            writeln!(f, ";")?;
          }
        }
      }

      ItemKind::Krate(rng) => {
        writeln!(f, "{} {{", "crate".magenta().bold())?;
        for (cid, _) in cre.extra_get(rng) {
          let item_id = ItemId::new_from((cid, NodeKind::Item));
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::Module(rng) => {
        let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
        writeln!(f, "{} {} {{", "mod".blue(), name.magenta().bold())?;
        for (cid, _) in cre.extra_get(rng) {
          let item_id = ItemId::new_from((cid, NodeKind::Item));
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::ModuleUnloaded => {
        let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
        writeln!(f, "{} {};", "mod".blue(), name.magenta().bold())?;
      }

      ItemKind::ModuleFile(rng, fid) => {
        let name = self.name.map(|n| n.str(far)).unwrap_or("<anon>");
        writeln!(
          f,
          "{} {} {} {{",
          "mod".blue(),
          name.magenta().bold(),
          format!("/* fid: {} */", fid).bright_black()
        )?;
        for (cid, _) in cre.extra_get(rng) {
          let item_id = ItemId::new_from((cid, NodeKind::Item));
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::Generic { params, reqs, ctn } => {
        write!(f, "{}<", "generic".blue())?;
        let mut first = true;
        for (pid, _) in cre.extra_get(params) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((pid, NodeKind::Thing));
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ">")?;

        if reqs.0 != reqs.1 {
          write!(f, " {} ", "requires".blue())?;
          let mut rfirst = true;
          for (rid, _) in cre.extra_get(reqs) {
            if !rfirst {
              write!(f, ", ")?;
            }
            rfirst = false;
            let thing_id = ThingId::new_from((rid, NodeKind::Thing));
            thing_id.dump(cre, sin, far, f, indent)?;
          }
        }

        writeln!(f, " {{")?;
        for (cid, _) in cre.extra_get(ctn) {
          let item_id = ItemId::new_from((cid, NodeKind::Item));
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
        for (cid, _) in cre.extra_get(ctn) {
          let item_id = ItemId::new_from((cid, NodeKind::Item));
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::ImplIn { trait_ty, ctn } => {
        write!(f, "{} : ", "impl".blue())?;
        trait_ty.dump(cre, sin, far, f, indent)?;
        writeln!(f, " {{")?;
        for (cid, _) in cre.extra_get(ctn) {
          let item_id = ItemId::new_from((cid, NodeKind::Item));
          item_id.dump(cre, sin, far, f, indent + 1)?;
        }
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
      }

      ItemKind::Import(rng) => {
        write!(f, "{} ", "use".blue())?;
        let mut first = true;
        for (tid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, "::")?;
          }
          first = false;
          let thing_id = ThingId::new_from((tid, NodeKind::Thing));
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        writeln!(f, ";")?;
      }
    }

    Ok(())
  }
}


// Type
impl DumpHandler for Type {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self.kind {
      TypeKind::Type()  => write!(f, "{}", "type".cyan())?,
      TypeKind::SelfT() => write!(f, "{}", "Self".cyan())?,

      TypeKind::Nick(name) => {
        write!(f, "{}", name.str(far).cyan())?;
      }

      TypeKind::Path(rng) => {
        let mut first = true;
        for (tid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, "::")?;
          }
          first = false;
          let type_id = TypeId::new_from((tid, NodeKind::Type));
          type_id.dump(cre, sin, far, f, indent)?;
        }
      }

      TypeKind::Ptr(sub, ism) => {
        write!(f, "^{}", if ism { "mut " } else { "" })?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Ref(sub, ism) => {
        write!(f, "&{}", if ism { "mut " } else { "" })?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::LVa(sub, ism) => {
        write!(f, "lval {}{}", if ism { "mut " } else { "" }, "")?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Array(sub, len) => {
        write!(f, "[")?;
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, "; ")?;
        len.dump(cre, sin, far, f, indent)?;
        write!(f, "]")?;
      }

      TypeKind::Slice(sub) => {
        write!(f, "[")?;
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, "]")?;
      }

      TypeKind::Vector(sub, len) => {
        write!(f, "[")?;
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, " * ")?;
        len.dump(cre, sin, far, f, indent)?;
        write!(f, "]")?;
      }

      TypeKind::Unit => {
        write!(f, "()")?;
      }

      TypeKind::Range(sub) => {
        write!(f, "..")?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Option(sub) => {
        write!(f, "?")?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Fail(sub) => {
        write!(f, "!")?;
        sub.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Result { sub, err } => {
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, " ! ")?;
        err.dump(cre, sin, far, f, indent)?;
      }

      TypeKind::Variant(rng) => {
        write!(f, "{} {{ ", "variant".blue())?;
        let mut first = true;
        for (tid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((tid, NodeKind::Thing));
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Enum(rng) => {
        write!(f, "{} {{ ", "enum".blue())?;
        let mut first = true;
        for (tid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((tid, NodeKind::Thing));
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Flags(rng) => {
        write!(f, "{} {{ ", "flags".blue())?;
        let mut first = true;
        for (tid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((tid, NodeKind::Thing));
          thing_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, " }}")?;
      }

      TypeKind::Struct(rng) => {
        write!(f, "{} {{ ... }}", "struct".blue())?;
        let _ = rng;
      }

      TypeKind::Tuple(rng) => {
        write!(f, "(")?;
        let mut first = true;
        for (tid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let type_id = TypeId::new_from((tid, NodeKind::Type));
          type_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }

      TypeKind::Iface(rng) => {
        write!(f, "{} {{ ... }}", "iface".blue())?;
        let _ = rng;
      }

      TypeKind::Trait(rng) => {
        write!(f, "{} {{ ... }}", "trait".blue())?;
        let _ = rng;
      }

      TypeKind::Fun { args, ret, attr } => {
        write!(f, "(")?;
        let mut first = true;
        for (aid, _) in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((aid, NodeKind::Thing));
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
        for (aid, _) in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((aid, NodeKind::Thing));
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
        for (aid, _) in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((aid, NodeKind::Thing));
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
        for id in cre.extra_get(args) {
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


// Expr
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
        for (eid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, "::")?;
          }
          first = false;
          let expr_id = ExprId::new_from((eid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Member(rng) => {
        let mut first = true;
        for (eid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ".")?;
          }
          first = false;
          let expr_id = ExprId::new_from((eid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
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
        for (eid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let expr_id = ExprId::new_from((eid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }

      ExprKind::Array(rng) => {
        write!(f, "[")?;
        let mut first = true;
        for (eid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let expr_id = ExprId::new_from((eid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "]")?;
      }

      ExprKind::Propagate(rng, count) => {
        write!(f, "[")?;
        let mut first = true;
        for (eid, _) in cre.extra_get(rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let expr_id = ExprId::new_from((eid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "; ")?;
        count.dump(cre, sin, far, f, indent)?;
        write!(f, "]")?;
      }

      ExprKind::Block { label, rng, expr } => {
        if let Some(lbl) = label {
          write!(f, "{}: ", lbl.str(far).bright_black())?;
        }
        writeln!(f, "{{")?;
        for (eid, _) in cre.extra_get(rng) {
          let expr_id = ExprId::new_from((eid, NodeKind::Expr));
          write_indent(f, indent + 1)?;
          expr_id.dump(cre, sin, far, f, indent + 1)?;
          writeln!(f, ";")?;
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
        write!(f, "{} ", "if".blue())?;
        cond.dump(cre, sin, far, f, indent)?;
        write!(f, " ")?;
        then.dump(cre, sin, far, f, indent)?;
        if let Some(el) = elsb {
          write!(f, " {} ", "else".blue())?;
          el.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Match { cond, arms } => {
        write!(f, "{} ", "match".blue())?;
        cond.dump(cre, sin, far, f, indent)?;
        writeln!(f, " {{")?;
        for (aid, _) in cre.extra_get(arms) {
          let thing_id = ThingId::new_from((aid, NodeKind::Thing));
          write_indent(f, indent + 1)?;
          thing_id.dump(cre, sin, far, f, indent + 1)?;
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

      ExprKind::Unary { op, val } => {
        let op_str = match op {
          UnaryOp::Neg => "-",
          UnaryOp::Poz => "+",
          UnaryOp::Not => "!",
          UnaryOp::Ref => "&",
          UnaryOp::Addr => "@",
          UnaryOp::Deref => "*",
        };
        write!(f, "{}", op_str.red())?;
        val.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::Binary { op, lhs, rhs } => {
        let op_str = match op {
          BinaryOp::Add => "+",
          BinaryOp::Sub => "-",
          BinaryOp::Mul => "*",
          BinaryOp::Div => "/",
          BinaryOp::Rem => "%",
          BinaryOp::Eq => "==",
          BinaryOp::Ne => "!=",
          BinaryOp::Lt => "<",
          BinaryOp::Gt => ">",
          BinaryOp::Lte => "<=",
          BinaryOp::Gte => ">=",
          BinaryOp::And => "&&",
          BinaryOp::Or => "||",
          BinaryOp::Xor => "^",
          BinaryOp::Shl => "<<",
          BinaryOp::Shr => ">>",
        };
        write!(f, "(")?;
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", op_str.red())?;
        rhs.dump(cre, sin, far, f, indent)?;
        write!(f, ")")?;
      }

      ExprKind::Assign { lhs, rhs } => {
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", "=".red())?;
        rhs.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::AssignOp { op, lhs, rhs } => {
        let op_str = match op {
          BinaryOp::Add => "+=",
          BinaryOp::Sub => "-=",
          BinaryOp::Mul => "*=",
          BinaryOp::Div => "/=",
          BinaryOp::Rem => "%=",
          BinaryOp::Eq => "==",
          BinaryOp::Ne => "!=",
          BinaryOp::Lt => "<=",
          BinaryOp::Gt => ">=",
          BinaryOp::Lte => "<=",
          BinaryOp::Gte => ">=",
          BinaryOp::And => "&=",
          BinaryOp::Or => "|=",
          BinaryOp::Xor => "^=",
          BinaryOp::Shl => "<<=",
          BinaryOp::Shr => ">>=",
        };
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", op_str.red())?;
        rhs.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::Exchange { lhs, rhs } => {
        lhs.dump(cre, sin, far, f, indent)?;
        write!(f, " {} ", "<-".red())?;
        rhs.dump(cre, sin, far, f, indent)?;
      }

      ExprKind::Call { callee, args } => {
        callee.dump(cre, sin, far, f, indent)?;
        write!(f, "(")?;
        let mut first = true;
        for (aid, _) in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let expr_id = ExprId::new_from((aid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }

      ExprKind::Index { callee, args } => {
        callee.dump(cre, sin, far, f, indent)?;
        write!(f, "[")?;
        let mut first = true;
        for (aid, _) in cre.extra_get(args) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let expr_id = ExprId::new_from((aid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "]")?;
      }

      ExprKind::Let { item, kind, init, ism } => {
        let kw = if ism { "var".blue() } else { "let".blue() };
        write!(f, "{} ", kw)?;
        item.dump(cre, sin, far, f, indent)?;
        if let Some(ty) = kind {
          write!(f, ": ")?;
          ty.dump(cre, sin, far, f, indent)?;
        }
        if let Some(ini) = init {
          write!(f, " = ")?;
          ini.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Return { label, val } => {
        write!(f, "{}", "return".blue())?;
        if let Some(lbl) = label {
          write!(f, " {}", lbl.str(far).bright_black())?;
        }
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Break { label, val } => {
        write!(f, "{}", "break".blue())?;
        if let Some(lbl) = label {
          write!(f, " {}", lbl.str(far).bright_black())?;
        }
        if let Some(v) = val {
          write!(f, " ")?;
          v.dump(cre, sin, far, f, indent)?;
        }
      }

      ExprKind::Continue { label } => {
        write!(f, "{}", "continue".blue())?;
        if let Some(lbl) = label {
          write!(f, " {}", lbl.str(far).bright_black())?;
        }
      }

      ExprKind::Die { lvar } => {
        write!(f, "{} {}", "die".red().bold(), lvar.str(far))?;
      }

      ExprKind::Try(sub) => {
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, "{}", "?".red())?;
      }

      ExprKind::Unwrap(sub) => {
        sub.dump(cre, sin, far, f, indent)?;
        write!(f, "{}", "!".red())?;
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
        for id in cre.extra_get(args) {
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


// Patt
impl DumpHandler for Patt {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self {
      Patt::One(ident) => {
        write!(f, "{}", ident.str(far).white().bold())?;
      }
      
      Patt::Under => {
        write!(f, "_")?;
      }
      
      Patt::Rest => {
        write!(f, "..")?;
      }
      
      Patt::Tuple(rng) => {
        write!(f, "(")?;
        let mut first = true;
        for (pid, _) in cre.extra_get(*rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let patt_id = PattId::new_from((pid, NodeKind::Patt));
          patt_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, ")")?;
      }
      
      Patt::Array(rng) => {
        write!(f, "[")?;
        let mut first = true;
        for (pid, _) in cre.extra_get(*rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let patt_id = PattId::new_from((pid, NodeKind::Patt));
          patt_id.dump(cre, sin, far, f, indent)?;
        }
        write!(f, "]")?;
      }
    }

    Ok(())
  }
}


// Thing
impl DumpHandler for Thing {
  fn dump(&self, cre: &Krate, sin: &StrInterner, far: &Files, f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
    match self {
      Thing::Name(ident) => {
        write!(f, "{}", ident.str(far).white())?;
      }

      Thing::List(rng) => {
        write!(f, "(")?;
        let mut first = true;
        for (tid, _) in cre.extra_get(*rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let thing_id = ThingId::new_from((tid, NodeKind::Thing));
          thing_id.dump(cre, sin, far, f, indent)?;
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
        for (tid, _) in cre.extra_get(*rng) {
          if !first {
            write!(f, " | ")?;
          }
          first = false;
          let type_id = TypeId::new_from((tid, NodeKind::Type));
          type_id.dump(cre, sin, far, f, indent)?;
        }
      }

      Thing::NamedExprList(ident, rng) => {
        write!(f, "{}(", ident.str(far).white())?;
        let mut first = true;
        for (eid, _) in cre.extra_get(*rng) {
          if !first {
            write!(f, ", ")?;
          }
          first = false;
          let expr_id = ExprId::new_from((eid, NodeKind::Expr));
          expr_id.dump(cre, sin, far, f, indent)?;
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
