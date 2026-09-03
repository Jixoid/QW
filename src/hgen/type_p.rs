use std::assert_matches;

use crate::{ast::{self, AstKind, Rng}, diagnostic::Message, hgen::{ExprGen, GenContext}, hir, lexer::{Span, SrcLoc, SrcSId}};


pub struct TypeGen;

impl TypeGen {

  pub fn low(ctx: &mut GenContext, id: ast::TypeId) -> Result<hir::TypeId, Message> {
    if let Some(&cached) = ctx.type_cache.get(&id) { return Ok(cached); }

    let it = ctx.ast.get_type(id);

    let res = match it {
      // Primitive
      ast::Type::Ptr { sub, acc } => {
        let ty = Self::low(ctx, *sub)?;
        let ac = hir::AccessKind::from(*acc);
        
        ctx.hir.new_type(hir::Type::Ptr(ty, ac))
      }

      ast::Type::Ref { sub, acc } => {
        let ty = Self::low(ctx, *sub)?;
        let ac = hir::AccessKind::from(*acc);
        
        ctx.hir.new_type(hir::Type::Ref(ty, ac))
      }
      
      ast::Type::Option { sub } => {
        let ty = TypeGen::low(ctx, *sub)?;

        ctx.hir.new_type(hir::Type::Option(ty))
      }
      
      ast::Type::Result { sub, err } => {
        let ty = TypeGen::low(ctx, *sub)?;
        let er = TypeGen::low(ctx, *err)?;
        
        ctx.hir.new_type(hir::Type::Result(ty, er))
      }

      ast::Type::Vector { sub, ext } => {
        let ty = TypeGen::low(ctx, *sub)?;
        let ex = ExprGen::low(ctx, *ext)?;

        ctx.hir.new_type(hir::Type::Vector(ty, ex))
      }

      ast::Type::Fun  { args, ret, attr } => Self::low_fun(ctx, args, ret, attr)?,
      ast::Type::Init { args, attr } => Self::low_fun(ctx, args, &None, attr)?,
      ast::Type::Fini { args, attr } => Self::low_fun(ctx, args, &None, attr)?,
      
      ast::Type::Nick   ( pos ) => Self::low_nick(ctx, pos)?,
      ast::Type::Array  { sub, ext } => Self::low_array(ctx, sub, ext)?,
      ast::Type::Struct { vars, bases } => Self::low_struct(ctx, id, vars, bases)?,
      ast::Type::Enum   { vals, .. } => Self::low_enum(ctx, id, vals)?,
      ast::Type::Flags  { vals, .. } => Self::low_flags(ctx, id, vals)?,
      ast::Type::Iface  { funs, bases } => Self::low_iface(ctx, id, funs, bases)?,
      ast::Type::Trait  { funs, bases } => Self::low_trait(ctx, id, funs, bases)?,
      ast::Type::Range  { sub } => Self::low_range(ctx, sub)?,
      ast::Type::Tuple  { vars } => Self::low_tuple(ctx, vars)?,
      ast::Type::Path   ( rng ) => Self::low_path(ctx, rng)?,
      ast::Type::Specialize { base, args } => Self::low_specialize(ctx, base, args)?,
    };

    ctx.type_cache.insert(id, res);
    Ok(res)
  }


  fn low_fun(ctx: &mut GenContext, args: &Rng, ret: &Option<ast::TypeId>, _attr: &u8) -> Result<hir::TypeId, Message> {
    let mut hir_args = vec![];

    for id in ctx.ast.get_extra(args) {
      let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));

      if let ast::Thing::NamedType(span, k) = it {
        let arg_ty = TypeGen::low(ctx, *k)?;
        let thing = hir::Thing::NamedType(*span, arg_ty);
        let thing_id = ctx.hir.new_thing(thing);
        hir_args.push(thing_id);
      }
    }

    let hir_ret = match ret {
      Some(r) => Some(TypeGen::low(ctx, *r)?),
      None => None,
    };

    let rng = ctx.hir.new_extra_from(&hir_args);
    Ok(ctx.hir.new_type(hir::Type::Fun { args: rng, ret: hir_ret }))
  }


  fn low_nick(ctx: &mut GenContext, pos: &Span) -> Result<hir::TypeId, Message> {
    let name = pos.str(ctx.far);

    // Primitive types
    let prim = match name {
      "i8"    => Some(hir::Type::Int(8, true)),
      "i16"   => Some(hir::Type::Int(16, true)),
      "i32"   => Some(hir::Type::Int(32, true)),
      "i64"   => Some(hir::Type::Int(64, true)),
      "i128"  => Some(hir::Type::Int(128, true)),
      "isize" => Some(hir::Type::ArchInt(true)),

      "u8"    => Some(hir::Type::Int(8, false)),
      "u16"   => Some(hir::Type::Int(16, false)),
      "u32"   => Some(hir::Type::Int(32, false)),
      "u64"   => Some(hir::Type::Int(64, false)),
      "u128"  => Some(hir::Type::Int(128, false)),
      "usize" => Some(hir::Type::ArchInt(false)),

      "f16"   => Some(hir::Type::Float(16)),
      "f32"   => Some(hir::Type::Float(32)),
      "f64"   => Some(hir::Type::Float(64)),
      "f80"   => Some(hir::Type::Float(80)),
      "f128"  => Some(hir::Type::Float(128)),

      "bool"  => Some(hir::Type::Bool),
      "char"  => Some(hir::Type::Char),
      "str"   => Some(hir::Type::Str),

      "type"  => Some(hir::Type::MetaType),

      "Self" => {
        if let Some(self_id) = ctx.self_type {
          return TypeGen::low(ctx, self_id);
        } else {
          return Err(Message::error(*pos, "`Self` type used but not found", vec![]));
        }
      }

      _ => None,
    };

    if let Some(ty) = prim { return Ok(ctx.hir.new_type(ty)); }


    // Scope resolution
    if let Some(target_id) = ctx.global.lookup(ctx, pos.sid()) {
      match target_id.kind {
        ast::AstKind::Decl => {
          let decl_id = ast::DeclId::new_from(target_id);
          
          if let Some(&cached) = ctx.decl_cache.get(&decl_id) { return Ok(cached); }

          let decl = ctx.ast.get_decl(decl_id);

          if let ast::DeclVari::Using { kind } = &decl.vari {
            let placeholder = ctx.hir.new_type(hir::Type::MetaType);
            ctx.decl_cache.insert(decl_id, placeholder);

            let real_ty = TypeGen::low(ctx, *kind)?;
            let real_data = ctx.hir.get_type(real_ty).clone();
            ctx.hir.list_type[placeholder.index as usize] = real_data;
            ctx.type_cache.insert(*kind, placeholder);
            return Ok(placeholder);
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

    Err(Message::error(*pos, "unknown type: `{}`", vec![ name.to_string() ]))
  }

  fn low_struct(ctx: &mut GenContext, id: ast::TypeId, vars: &Rng, _bases: &Option<Rng>) -> Result<hir::TypeId, Message> {
    ctx.global.enter(id);

    let mut hir_fields = vec![];
    let mut seen_fields = std::collections::HashSet::new();

    for id in ctx.ast.get_extra(vars) {
      let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));

      if let ast::Thing::NamedTypeVis(span, _vis, k) = it {
        // Duplicate field check
        if !seen_fields.insert(span.sid()) {
          return Err(Message::error(*span, "duplicate field `{}` in struct", vec![ span.str(ctx.far).to_string() ]));
        }

        let field_ty = TypeGen::low(ctx, *k)?;
        let thing = hir::Thing::NamedType(*span, field_ty);
        let thing_id = ctx.hir.new_thing(thing);
        hir_fields.push(thing_id);
      }
    }

    ctx.global.leave();

    let rng = ctx.hir.new_extra_from(&hir_fields);
    let struct_ty = ctx.hir.new_type(hir::Type::Struct(rng));

    // Infinite size check (by-value recursive cycles)
    let mut visited = std::collections::HashSet::new();
    Self::check_infinite_size(ctx, struct_ty, &mut visited)?;

    ctx.type_to_ast.insert(struct_ty, id);
    Ok(struct_ty)
  }

  fn check_infinite_size(ctx: &GenContext, target: hir::TypeId, visited: &mut std::collections::HashSet<hir::TypeId>) -> Result<(), Message> {
    if !visited.insert(target) {
      return Err(Message::nonp_fatal(0, "recursive type has infinite size", vec![]));
    }

    let ty = ctx.hir.get_type(target);
    if let hir::Type::Struct(rng) = ty {
      for id in ctx.hir.get_extra(rng) {
        let thing = ctx.hir.get_thing(hir::ThingId::new_from(*id));
        if let hir::Thing::NamedType(_sid, field_ty) = thing {
          Self::check_infinite_size_field(ctx, *field_ty, visited)?;
        }
      }
    }

    visited.remove(&target);
    Ok(())
  }

  fn check_infinite_size_field(ctx: &GenContext, field_ty: hir::TypeId, visited: &mut std::collections::HashSet<hir::TypeId>) -> Result<(), Message> {
    let ty = ctx.hir.get_type(field_ty);
    match ty {
      hir::Type::Struct(_) => {
        Self::check_infinite_size(ctx, field_ty, visited)?;
      }
      hir::Type::StaArr(sub, _) => {
        Self::check_infinite_size_field(ctx, *sub, visited)?;
      }
      hir::Type::Tuple(rng) => {
        for id in ctx.hir.get_extra(rng) {
          let thing = ctx.hir.get_thing(hir::ThingId::new_from(*id));
          if let hir::Thing::NamedType(_sid, sub_ty) = thing {
            Self::check_infinite_size_field(ctx, *sub_ty, visited)?;
          }
        }
      }
      // Pointer and Reference break by-value cycles (indirection)
      hir::Type::Ptr(..) | hir::Type::Ref(..) | hir::Type::DynArr(..) => {}
      _ => {}
    }
    Ok(())
  }

  fn low_array(ctx: &mut GenContext, sub: &ast::TypeId, ext: &Option<Rng>) -> Result<hir::TypeId, Message> {
    let ty = TypeGen::low(ctx, *sub)?;

    if let Some(ext) = ext { 
      for x in ctx.ast.get_extra(ext) {
        assert_matches!(x.kind, AstKind::Expr);
      }

      Ok(ctx.hir.new_type(hir::Type::DynArr(ty)))
    } else {
      Ok(ctx.hir.new_type(hir::Type::DynArr(ty)))
    }
  }

  fn low_enum(_ctx: &mut GenContext, _id: ast::TypeId, vals: &Rng) -> Result<hir::TypeId, Message> {
    Ok(_ctx.hir.new_type(hir::Type::Enum(vals.clone())))
  }

  fn low_flags(_ctx: &mut GenContext, _id: ast::TypeId, vals: &Rng) -> Result<hir::TypeId, Message> {
    Ok(_ctx.hir.new_type(hir::Type::Flags(vals.clone())))
  }

  fn low_iface(ctx: &mut GenContext, id: ast::TypeId, funs: &Rng, _bases: &Option<Rng>) -> Result<hir::TypeId, Message> {
    if let Some(&cached) = ctx.type_cache.get(&id) {
      return Ok(cached);
    }
    let placeholder = ctx.hir.new_type(hir::Type::MetaType);
    ctx.type_cache.insert(id, placeholder);

    let old_self = ctx.self_type;
    ctx.self_type = Some(id);

    ctx.global.enter(id.to_any());

    let mut hir_funs = vec![];
    for x in ctx.ast.get_extra(funs) {
      if x.kind == AstKind::Decl {
        let decl = ctx.ast.get_decl(ast::DeclId::new_from(*x));
        let method_ty = match &decl.vari {
          ast::DeclVari::Fun { kind, .. } => TypeGen::low(ctx, *kind)?,
          _ => ctx.hir.new_type(hir::Type::Unit),
        };
        let thing = hir::Thing::NamedType(decl.name, method_ty);
        let thing_id = ctx.hir.new_thing(thing);
        hir_funs.push(thing_id);
      }
    }

    ctx.global.leave();
    ctx.self_type = old_self;

    let rng = ctx.hir.new_extra_from(&hir_funs);
    ctx.hir.list_type[placeholder.index as usize] = hir::Type::Iface(rng);
    Ok(placeholder)
  }

  fn low_trait(ctx: &mut GenContext, id: ast::TypeId, funs: &Rng, _bases: &Option<Rng>) -> Result<hir::TypeId, Message> {
    if let Some(&cached) = ctx.type_cache.get(&id) {
      return Ok(cached);
    }
    let placeholder = ctx.hir.new_type(hir::Type::MetaType);
    ctx.type_cache.insert(id, placeholder);

    let old_self = ctx.self_type;
    ctx.self_type = Some(id);

    ctx.global.enter(id.to_any());

    let mut hir_funs = vec![];
    for x in ctx.ast.get_extra(funs) {
      if x.kind == AstKind::Decl {
        let decl = ctx.ast.get_decl(ast::DeclId::new_from(*x));
        let method_ty = match &decl.vari {
          ast::DeclVari::Fun { kind, .. } => TypeGen::low(ctx, *kind)?,
          _ => ctx.hir.new_type(hir::Type::Unit),
        };
        let thing = hir::Thing::NamedType(decl.name, method_ty);
        let thing_id = ctx.hir.new_thing(thing);
        hir_funs.push(thing_id);
      }
    }

    ctx.global.leave();
    ctx.self_type = old_self;

    let rng = ctx.hir.new_extra_from(&hir_funs);
    ctx.hir.list_type[placeholder.index as usize] = hir::Type::Trait(rng);
    Ok(placeholder)
  }

  fn low_range(ctx: &mut GenContext, sub: &ast::TypeId) -> Result<hir::TypeId, Message> {
    let sub_ty = TypeGen::low(ctx, *sub)?;
    Ok(ctx.hir.new_type(hir::Type::Range(sub_ty)))
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
        if let ast::Type::Nick(pos) = ty {
          ctx.global.lookup(ctx, pos.sid())
        } else {
          None
        }
      }
      AstKind::Thing => {
        let thing = ctx.ast.get_thing(ast::ThingId::new_from(first_any));
        if let ast::Thing::Name(span) = thing {
          ctx.global.lookup(ctx, span.sid())
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
          if let ast::Type::Nick(pos) = ty {
            pos.sid()
          } else {
            panic!("expected nick in path segment");
          }
        }
        AstKind::Thing => {
          let thing = ctx.ast.get_thing(ast::ThingId::new_from(seg_any));
          if let ast::Thing::Name(span) = thing {
            span.sid()
          } else {
            panic!("expected name in path segment");
          }
        }
        _ => panic!("unexpected segment kind"),
      };

      if let Some(target) = current_target {
        current_target = ctx.global.lookup_in_scope(ctx, target, seg_sid);
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
