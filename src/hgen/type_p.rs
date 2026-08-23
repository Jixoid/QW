use std::assert_matches;

use crate::{ast::{self, AstKind, Rng}, diagnostic::Message, hgen::GenContext, hir, lexer::Span};


pub struct TypeGen;

impl TypeGen {

  pub fn low(ctx: &mut GenContext, id: ast::TypeId) -> Result<hir::TypeId, Message> {
    if let Some(&cached) = ctx.type_cache.get(&id) { return Ok(cached); }

    let it = ctx.ast.get_type(id);

    let res = match it {
      ast::Type::Nick   { pos, .. } => Self::low_nick(ctx, pos)?,
      ast::Type::Ptr    { sub, acc } => Self::low_ptr(ctx, sub, acc)?,
      ast::Type::Ref    { sub, acc } => Self::low_ref(ctx, sub, acc)?,
      ast::Type::Array  { sub, ext } => Self::low_array(ctx, sub, ext)?,
      ast::Type::Struct { vars, bases } => Self::low_struct(ctx, id, vars, bases)?,
      ast::Type::Fun    { args, ret, attr } => Self::low_fun(ctx, args, ret, attr)?,
      ast::Type::Init   { args, attr } => Self::low_fun(ctx, args, &None, attr)?,
      ast::Type::Fini   { args, attr } => Self::low_fun(ctx, args, &None, attr)?,
      ast::Type::Enum   { vals, .. } => Self::low_enum(ctx, id, vals)?,
      ast::Type::Flags  { vals, .. } => Self::low_flags(ctx, id, vals)?,
      ast::Type::Iface  { funs, bases } => Self::low_iface(ctx, id, funs, bases)?,
      ast::Type::Trait  { funs, bases } => Self::low_trait(ctx, id, funs, bases)?,
      ast::Type::Range  { sub } => Self::low_range(ctx, sub)?,
      ast::Type::Option { sub } => Self::low_option(ctx, sub)?,
      ast::Type::Result { sub, err } => Self::low_result(ctx, sub, err)?,
      ast::Type::Vector { sub, ext } => Self::low_vector(ctx, sub, ext)?,
      ast::Type::Tuple  { vars } => Self::low_tuple(ctx, vars)?,
      ast::Type::Path(rng) => Self::low_path(ctx, rng)?,
      ast::Type::Specialize { base, args } => Self::low_specialize(ctx, base, args)?,
    };

    ctx.type_cache.insert(id, res);
    Ok(res)
  }


  fn low_nick(ctx: &mut GenContext, pos: &Span) -> Result<hir::TypeId, Message> {
    let name = pos.str(ctx.far);

    // Primitive types
    let prim = match name {
      "i8"    => Some(hir::Type::Int(8, true)),
      "i16"   => Some(hir::Type::Int(16, true)),
      "i32"   => Some(hir::Type::Int(32, true)),
      "i64"   => Some(hir::Type::Int(64, true)),
      "isize" => Some(hir::Type::ArchInt(true)),

      "u8"    => Some(hir::Type::Int(8, false)),
      "u16"   => Some(hir::Type::Int(16, false)),
      "u32"   => Some(hir::Type::Int(32, false)),
      "u64"   => Some(hir::Type::Int(64, false)),
      "usize" => Some(hir::Type::ArchInt(false)),

      "f32"   => Some(hir::Type::Float(32)),
      "f64"   => Some(hir::Type::Float(64)),
      "fsize" => Some(hir::Type::ArchFloat),

      "bool"  => Some(hir::Type::Bool),
      "char"  => Some(hir::Type::Char),
      "str"   => Some(hir::Type::Str),

      "type"   => Some(hir::Type::MetaType),

      _ => None,
    };

    if let Some(ty) = prim {
      return Ok(ctx.hir.new_type(ty));
    }

    // Resolve `Self` keyword
    if name == "Self" {
      if let Some(self_id) = ctx.self_type {
        return TypeGen::low(ctx, self_id);
      }
    }

    // Scope resolution (user-defined types, generics, aliases)
    if let Some(target_id) = ctx.lookup(pos.sid) {
      match target_id.kind {
        ast::AstKind::Decl => {
          let decl_id = ast::DeclId::new_from(target_id);
          if let Some(&cached) = ctx.decl_cache.get(&decl_id) {
            return Ok(cached);
          }
          let decl = ctx.ast.get_decl(decl_id);
          if let ast::DeclVari::Using { kind } = &decl.vari {
            let placeholder = ctx.hir.new_type(hir::Type::MetaType);
            ctx.decl_cache.insert(decl_id, placeholder);
            ctx.type_cache.insert(*kind, placeholder);

            let real_ty = TypeGen::low(ctx, *kind)?;
            ctx.decl_cache.insert(decl_id, real_ty);
            ctx.type_cache.insert(*kind, real_ty);
            return Ok(real_ty);
          }
        }
        ast::AstKind::Thing => {
          let thing = ctx.ast.get_thing(ast::ThingId::new_from(target_id));
          if let ast::Thing::NamedType(_span, kind) = thing {
            return TypeGen::low(ctx, *kind);
          }
        }
        _ => {}
      }
    }

    let word = crate::lexer::Word {
      off: pos.off,
      size: pos.size,
      fid: pos.fid,
      kind: crate::lexer::WK::Word,
    };

    Err(Message::error(word, "unknown type: `{}`", vec![ name.to_string() ]))
  }

  fn low_ptr(ctx: &mut GenContext, sub: &ast::TypeId, acc: &ast::AccessKind) -> Result<hir::TypeId, Message> {
    let sub_ty = Self::low(ctx, *sub)?;
    let h_acc = match acc {
      ast::AccessKind::IMM => hir::AccessKind::IMM,
      ast::AccessKind::MUT => hir::AccessKind::MUT,
    };
    Ok(ctx.hir.new_type(hir::Type::Ptr(sub_ty, h_acc)))
  }

  fn low_ref(ctx: &mut GenContext, sub: &ast::TypeId, acc: &ast::AccessKind) -> Result<hir::TypeId, Message> {
    let sub_ty = Self::low(ctx, *sub)?;
    let h_acc = match acc {
      ast::AccessKind::IMM => hir::AccessKind::IMM,
      ast::AccessKind::MUT => hir::AccessKind::MUT,
    };
    Ok(ctx.hir.new_type(hir::Type::Ref(sub_ty, h_acc)))
  }

  fn low_struct(ctx: &mut GenContext, id: ast::TypeId, vars: &Rng, _bases: &Option<Rng>) -> Result<hir::TypeId, Message> {
    ctx.enter_scope(id.to_any());

    let mut hir_fields = vec![];

    for id in ctx.ast.get_extra(vars) {
      let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));

      if let ast::Thing::NamedTypeVis(span, _vis, k) = it {
        let field_ty = TypeGen::low(ctx, *k)?;
        let thing = hir::Thing::NamedType(span.sid, field_ty);
        let thing_id = ctx.hir.new_thing(thing);
        hir_fields.push(thing_id.index);
      }
    }

    ctx.leave_scope();

    let rng = if hir_fields.is_empty() {
      0..0
    } else {
      let start = hir_fields[0];
      let end = start + hir_fields.len() as u32;
      start..end
    };

    Ok(ctx.hir.new_type(hir::Type::Struct(rng)))
  }

  fn low_array(ctx: &mut GenContext, sub: &ast::TypeId, ext: &Option<Rng>) -> Result<hir::TypeId, Message> {
    let sub_ty = TypeGen::low(ctx, *sub)?;

    if let Some(ext) = ext { 
      for x in ctx.ast.get_extra(ext) {
        assert_matches!(x.kind, AstKind::Expr);
      }
      Ok(ctx.hir.new_type(hir::Type::DynArr(sub_ty)))
    } else {
      Ok(ctx.hir.new_type(hir::Type::DynArr(sub_ty)))
    }
  }

  fn low_fun(ctx: &mut GenContext, args: &Rng, ret: &Option<ast::TypeId>, _attr: &u8) -> Result<hir::TypeId, Message> {
    let mut hir_args = vec![];

    for id in ctx.ast.get_extra(args) {
      let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));

      if let ast::Thing::NamedType(span, k) = it {
        let arg_ty = TypeGen::low(ctx, *k)?;
        let thing = hir::Thing::NamedType(span.sid, arg_ty);
        let thing_id = ctx.hir.new_thing(thing);
        hir_args.push(thing_id.index);
      }
    }

    let hir_ret = match ret {
      Some(r) => Some(TypeGen::low(ctx, *r)?),
      None => None,
    };

    let rng = if hir_args.is_empty() {
      0..0
    } else {
      let start = hir_args[0];
      let end = start + hir_args.len() as u32;
      start..end
    };

    Ok(ctx.hir.new_type(hir::Type::Fun { args: rng, ret: hir_ret }))
  }

  fn low_enum(_ctx: &mut GenContext, _id: ast::TypeId, vals: &Rng) -> Result<hir::TypeId, Message> {
    Ok(_ctx.hir.new_type(hir::Type::Enum(vals.clone())))
  }

  fn low_flags(_ctx: &mut GenContext, _id: ast::TypeId, vals: &Rng) -> Result<hir::TypeId, Message> {
    Ok(_ctx.hir.new_type(hir::Type::Flags(vals.clone())))
  }

  fn low_iface(ctx: &mut GenContext, id: ast::TypeId, funs: &Rng, _bases: &Option<Rng>) -> Result<hir::TypeId, Message> {
    ctx.enter_scope(id.to_any());

    for x in ctx.ast.get_extra(funs) {
      if x.kind == AstKind::Decl {
        crate::hgen::ItemGen::low_decl(ctx, ast::DeclId::new_from(*x))?;
      }
    }

    ctx.leave_scope();

    Ok(ctx.hir.new_type(hir::Type::Iface(funs.clone())))
  }

  fn low_trait(ctx: &mut GenContext, id: ast::TypeId, funs: &Rng, _bases: &Option<Rng>) -> Result<hir::TypeId, Message> {
    ctx.enter_scope(id.to_any());

    for x in ctx.ast.get_extra(funs) {
      if x.kind == AstKind::Decl {
        crate::hgen::ItemGen::low_decl(ctx, ast::DeclId::new_from(*x))?;
      }
    }

    ctx.leave_scope();

    Ok(ctx.hir.new_type(hir::Type::Trait(funs.clone())))
  }

  fn low_range(ctx: &mut GenContext, sub: &ast::TypeId) -> Result<hir::TypeId, Message> {
    let sub_ty = TypeGen::low(ctx, *sub)?;
    Ok(ctx.hir.new_type(hir::Type::Range(sub_ty)))
  }

  fn low_option(ctx: &mut GenContext, sub: &ast::TypeId) -> Result<hir::TypeId, Message> {
    let sub_ty = TypeGen::low(ctx, *sub)?;
    Ok(ctx.hir.new_type(hir::Type::Option(sub_ty)))
  }

  fn low_result(ctx: &mut GenContext, sub: &ast::TypeId, err: &ast::TypeId) -> Result<hir::TypeId, Message> {
    let sub_ty = TypeGen::low(ctx, *sub)?;
    let err_ty = TypeGen::low(ctx, *err)?;
    Ok(ctx.hir.new_type(hir::Type::Result(sub_ty, err_ty)))
  }

  fn low_vector(ctx: &mut GenContext, sub: &ast::TypeId, _ext: &ast::ExprId) -> Result<hir::TypeId, Message> {
    let sub_ty = TypeGen::low(ctx, *sub)?;
    Ok(ctx.hir.new_type(hir::Type::Vector(sub_ty, 0)))
  }

  fn low_tuple(ctx: &mut GenContext, vars: &Rng) -> Result<hir::TypeId, Message> {
    for id in ctx.ast.get_extra(vars) {
      if id.kind == AstKind::Type {
        let _ = TypeGen::low(ctx, ast::TypeId::new_from(*id))?;
      }
    }
    Ok(ctx.hir.new_type(hir::Type::Tuple(vars.clone())))
  }

  fn low_path(ctx: &mut GenContext, rng: &Rng) -> Result<hir::TypeId, Message> {
    let segs = ctx.ast.get_extra(rng);
    if segs.is_empty() {
      panic!("empty path");
    }

    // First segment
    let first_any = segs[0];
    let mut current_target = match first_any.kind {
      AstKind::Type => {
        let ty_id = ast::TypeId::new_from(first_any);
        let ty = ctx.ast.get_type(ty_id);
        if let ast::Type::Nick { pos, .. } = ty {
          ctx.lookup(pos.sid)
        } else {
          None
        }
      }
      AstKind::Thing => {
        let thing = ctx.ast.get_thing(ast::ThingId::new_from(first_any));
        if let ast::Thing::Name(span) = thing {
          ctx.lookup(span.sid)
        } else {
          None
        }
      }
      _ => None,
    };

    if current_target.is_none() {
      let first_ty_id = ast::TypeId::new_from(first_any);
      return TypeGen::low(ctx, first_ty_id);
    }

    // Traverse remaining segments
    for &seg_any in &segs[1..] {
      let seg_sid = match seg_any.kind {
        AstKind::Type => {
          let ty = ctx.ast.get_type(ast::TypeId::new_from(seg_any));
          if let ast::Type::Nick { pos, .. } = ty {
            pos.sid
          } else {
            panic!("expected nick in path segment");
          }
        }
        AstKind::Thing => {
          let thing = ctx.ast.get_thing(ast::ThingId::new_from(seg_any));
          if let ast::Thing::Name(span) = thing {
            span.sid
          } else {
            panic!("expected name in path segment");
          }
        }
        _ => panic!("unexpected segment kind"),
      };

      if let Some(target) = current_target {
        current_target = ctx.lookup_in_scope(target, seg_sid);
      }
    }

    if let Some(target) = current_target {
      match target.kind {
        AstKind::Decl => {
          let decl = ctx.ast.get_decl(ast::DeclId::new_from(target));
          if let ast::DeclVari::Using { kind } = &decl.vari {
            return TypeGen::low(ctx, *kind);
          }
        }
        AstKind::Type => {
          return TypeGen::low(ctx, ast::TypeId::new_from(target));
        }
        _ => {}
      }
    }

    // Fallback: try lowering the last segment directly
    let last_any = *segs.last().unwrap();
    if last_any.kind == AstKind::Type {
      return TypeGen::low(ctx, ast::TypeId::new_from(last_any));
    }

    panic!("unresolvable path: {:?}", rng);
  }

  fn low_specialize(ctx: &mut GenContext, base: &ast::TypeId, _args: &Rng) -> Result<hir::TypeId, Message> {
    let base_ty = TypeGen::low(ctx, *base)?;
    Ok(base_ty)
  }

}
