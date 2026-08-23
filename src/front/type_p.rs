use crate::ast::{AnyId, AstId, DeclId, FunAttrs, Item, ItemVari, Rng, Thing};
use crate::front::expr_p::ExprParser;
use crate::front::type_p::CtnRet::*;
use crate::lexer::{Span, WK};
use crate::{ast::{AccessKind, DeclVari, Type, TypeId, Visibility}, diagnostic::Message, front::{ParserContext, attr_p::AttrParser, decl_p::DeclParser, meta_p::MetaParser}};


enum CtnRet {
  Null,
  Fun(DeclId),
  Init(DeclId),
  Fini(DeclId),
  Impl(Item),
  Vars(Vec<AnyId>),
  Let(DeclId),
}

struct CtnMask {
  impl_: bool,
  vars_: bool,
  lets_: bool,
}


pub struct TypeParser {}

impl TypeParser {

  pub fn read_type(ctx: &mut ParserContext, indecl: bool) -> Result<TypeId, Message> {
    let attrs = AttrParser::read_attr(ctx)?;
    let _n = ctx.lex.get()?;

    let tyid = match _n.kind {
      WK::Struct => if indecl { return Err(Message::error(_n, "advanced structures are not permitted within the decl", vec![])); } else { Self::read_struct(ctx)? },
      WK::Fun    => if indecl { return Err(Message::error(_n, "advanced structures are not permitted within the decl", vec![])); } else { Self::read_fun(ctx)? },
      WK::Iface  => if indecl { return Err(Message::error(_n, "advanced structures are not permitted within the decl", vec![])); } else { Self::read_iface(ctx)? },
      WK::Trait  => if indecl { return Err(Message::error(_n, "advanced structures are not permitted within the decl", vec![])); } else { Self::read_trait(ctx)? },
      WK::Enum   => if indecl { return Err(Message::error(_n, "advanced structures are not permitted within the decl", vec![])); } else { Self::read_enum(ctx)? },
      WK::Flags  => if indecl { return Err(Message::error(_n, "advanced structures are not permitted within the decl", vec![])); } else { Self::read_flags(ctx)? },
      
      WK::Dot2 => {
        let sub = Self::read_type(ctx, indecl)?;
        
        ctx.cre.new_type(Type::Range{ sub })
      }

      WK::Question => {
        let sub = Self::read_type(ctx, indecl)?;

        ctx.cre.new_type(Type::Option{ sub })
      }

      WK::BitwiseAnd => {
        let nxt = ctx.lex.get()?;
        let acc = match nxt.kind {
          WK::Mut => AccessKind::MUT,
          WK::Imm => AccessKind::IMM,
          _ => {
            ctx.lex.store(nxt);
            AccessKind::IMM
          }
        };
        
        let sub = Self::read_type(ctx, indecl)?;
        
        ctx.cre.new_type(Type::Ref{ sub, acc })
      }
      
      WK::BitwiseXor => {
        let nxt = ctx.lex.get()?;
        let acc = match nxt.kind {
          WK::Mut => AccessKind::MUT,
          WK::Imm => AccessKind::IMM,
          _ => {
            ctx.lex.store(nxt);
            AccessKind::IMM
          }
        };

        let sub = Self::read_type(ctx, indecl)?;
        
        ctx.cre.new_type(Type::Ptr{ sub, acc })
      }
      
      WK::SquareBracketBeg => {
        let sub = Self::read_type(ctx, indecl)?;

        let _c = ctx.lex.get()?;
        match _c.str(ctx.far) {
          "]" => { // dyn array
            let ty = Type::Array{ sub, ext: None };
            ctx.cre.new_type(ty)
          }

          "," => { // sized array
            let mut ext = vec![ ExprParser::read_expr(ctx, 0)?.to_any() ];

            loop {
              let _c = ctx.lex.get()?;

              if _c.kind == WK::SquareBracketEnd { break; }
              else if _c.kind == WK::Comma {
                ext.push( ExprParser::read_expr(ctx, 0)?.to_any() );
              }
              else {
                _c.expect_kind2(WK::Comma, WK::SquareBracketEnd)?;
              }
            }

            let ty = Type::Array{ sub, ext: Some(ctx.cre.new_extra(ext)) };
            ctx.cre.new_type(ty)
          }

          "x" => { // vector
            let ext = ExprParser::read_expr(ctx, 0)?;

            ctx.lex.get()?.expect_kind(WK::SquareBracketEnd)?;

            let ty = Type::Vector{ sub, ext };
            ctx.cre.new_type(ty)
          }

          _ => {
            ctx.lex.store(_c);
            _c.expect_kind3(WK::SquareBracketEnd, WK::Comma, WK::Word)?;
            panic!()
          }
        }
      }

      WK::ParenBeg => {
        let mut vars = vec![];
        let mut is_tuple = false;
        
        loop {
          let _c = ctx.lex.get()?;
          if _c.kind == WK::ParenEnd { break; }
          else { ctx.lex.store(_c); }

          let sub = Self::read_type(ctx, indecl)?;
          vars.push(sub.to_any());
          
          let _c = ctx.lex.get()?;
          if _c.kind == WK::ParenEnd { break; }
          else if _c.kind == WK::Comma { is_tuple = true; continue; }
          else {
            _c.expect_kind2(WK::Comma, WK::ParenEnd)?;
          }
        }

        if is_tuple || vars.len() != 1 {
          let ty = Type::Tuple{ vars: ctx.cre.new_extra(vars) };
          ctx.cre.new_type(ty)
        } else {
          TypeId::new_from(vars[0])
        }
      }

      _ => {
        let nick = Type::Nick{pos: _n.expect_word(ctx.far)?.save(ctx), idx: ctx.cre.get_str(_n.str(ctx.far))};
        let mut path = vec![ ctx.cre.new_type(nick).to_any() ];

        loop {
          let _c = ctx.lex.get()?;
          
          match _c.kind {
            WK::Scope => {
              let _n = ctx.lex.get()?.expect_word(ctx.far)?;
              let sub = Type::Nick{pos: _n.save(ctx), idx: ctx.cre.get_str(_n.str(ctx.far))};
              
              path.push( ctx.cre.new_type(sub).to_any() );
            }

            WK::AngleBeg => {
              let mut args = vec![];
              loop {
                let _c = ctx.lex.get()?;
                ctx.lex.store(_c);

                let aarg = match _c.kind {
                  WK::Number | WK::String => ExprParser::read_atom(ctx)?.to_any(),
                  WK::CurlyBracketBeg => ExprParser::read_block(ctx)?.to_any(),
                  _ => TypeParser::read_type(ctx, true)?.to_any(),
                };

                args.push(aarg);
                
                let sep = ctx.lex.get()?;
                if sep.kind == WK::AngleEnd { break; }
                if sep.kind == WK::Comma { continue; }
                else {
                  sep.expect_kind2(WK::Comma, WK::AngleEnd)?;
                }
              }

              let base = Type::Path( ctx.cre.new_extra(path) );
              let base_id = ctx.cre.new_type(base);

              let ty = Type::Specialize{base: base_id, args: ctx.cre.new_extra(args)};
              path = vec![ ctx.cre.new_type(ty).to_any() ];
            }

            _ => {ctx.lex.store(_c); break},
          }
        }


        if path.len() == 1 { TypeId::new_from(path[0]) } else { let rng = ctx.cre.new_extra(path);  ctx.cre.new_type(Type::Path(rng)) }
      }
    };

    if let Some(a) = attrs { AttrParser::attach_attr(ctx, tyid, a); }

    Ok(tyid)
  }


  fn read_bases(ctx: &mut ParserContext) -> Result<Option<Rng>, Message> {
    let _c = ctx.lex.get()?;
    if _c.kind == WK::Colon {
      let mut arr = vec![];
      
      loop {
        arr.push( TypeParser::read_type(ctx, true)?.to_any() );

        let _c = ctx.lex.get()?;
        if _c.kind == WK::Comma { continue; }
        else {
          ctx.lex.store(_c);
          break;
        }
      }

      Ok(Some(ctx.cre.new_extra(arr)))
    }
    else {
      ctx.lex.store(_c);
      Ok(None)
    }
  }

  
  fn read_ctn(ctx: &mut ParserContext, vis: Visibility, mask: CtnMask) -> Result<CtnRet, Message> {
    let t = ctx.lex.get()?;
      
    let res = match t.kind {
      WK::Fun  => Fun(DeclParser::read_fun(ctx, vis)?),
      WK::Init => Init(DeclParser::read_init(ctx, vis)?),
      WK::Fini => Fini(DeclParser::read_fini(ctx, vis)?),

      WK::Impl => {
        if !mask.impl_ { return Err(Message::error(t, "not allowed here", vec![])) }

        ctx.lex.get()?.expect_kind(WK::Colon)?;

        let trait_ty = TypeParser::read_type(ctx, true)?;
        
        let _c = ctx.lex.get()?;
        let onedef = if _c.kind == WK::CurlyBracketBeg { false } else { ctx.lex.store(_c); true };

        let mut impls = Vec::new();
        let mut defvis = vis;

        loop {
          if !onedef {
            let t = ctx.lex.get()?;
            if t.kind == WK::CurlyBracketEnd { break; } else { ctx.lex.store(t); }
          }

          MetaParser::read_scpvis(ctx, &mut defvis)?;
          let attrs = AttrParser::read_attr(ctx)?;
          let v = MetaParser::read_vis(ctx, defvis)?;
          
          let next_kw = ctx.lex.get()?;
          if next_kw.kind == WK::Fun {
            let fun = DeclParser::read_fun(ctx, v)?;
            
            if let DeclVari::Fun { kind: _, blok } = ctx.cre.get_decl(fun).vari {
              if blok.is_none() {
                return Err(Message::error(next_kw, "extend methods must have a body (cannot end with `;`)", vec![]));
              }
            }
            
            if let Some(a) = attrs { AttrParser::attach_attr(ctx, fun, a); }
            impls.push(fun.to_any());
          } else {
            return Err(Message::error(next_kw, "expected `fun`, got `{}`", vec![ next_kw.string(ctx.far) ]));
          }

          if onedef { break; }
        }

        let imp = Item{vis, vari: ItemVari::Impl{
          trait_ty: Some(trait_ty),
          type_ty: AstId::null(),
          ctn: ctx.cre.new_extra(impls),
        }};

        Impl(imp)
      }

      WK::Var | WK::Word => {
        if !mask.vars_ { return Err(Message::error(t, "not allowed here", vec![])) }

        if t.kind == WK::Word { ctx.lex.store(t); }

        let mut names: Vec<Span> = vec![];

        loop {
          names.push(ctx.lex.get()?.expect_word(ctx.far)?.save(ctx));

          let c = ctx.lex.get()?;

          if c.kind == WK::Comma { continue; }
          else if c.kind == WK::Colon { break; }
          else {
            c.expect_kind2(WK::Colon, WK::Comma)?;
          }
        }

        let kind = TypeParser::read_type(ctx, true)?;
        
        ctx.lex.get()?.expect_kind(WK::Semicolon)?;

        
        let mut res = vec![];
        
        for x in names {
          let it = Thing::NamedTypeVis(x, vis, kind);
          res.push(ctx.cre.new_thing(it).to_any());
        }

        Vars(res)
      }

      WK::Let => {
        if !mask.lets_ { return Err(Message::error(t, "not allowed here", vec![])) }

        let dl = DeclParser::read_let(ctx, vis)?;
        Let(dl)
      }

      _ => {
        if t.kind == WK::CurlyBracketEnd {
          ctx.lex.store(t);
          Null
        } else {
          return Err(Message::error(t, "unexpected token inside definition: `{}`", vec![t.string(ctx.far)]));
        }
      }
    };

    Ok(res)
  }


  pub fn read_fun(ctx: &mut ParserContext) -> Result<TypeId, Message> {
    // Args
    let args = MetaParser::read_fun_args(ctx)?;

    // IF: Attribute
    let mut attr: u8 = 0;
    let mut _c = ctx.lex.get()?;

    while _c.kind != WK::ArrowRigh && _c.kind != WK::CurlyBracketBeg && _c.kind != WK::Backtick && _c.kind != WK::Semicolon {

      if _c.str(ctx.far) == "static" {
        if attr & FunAttrs::Static as u8 != 0 {
          ctx.sum.add(Message::warn(_c, "duplicated attribute: `static`", vec![]));
        }

        attr |= FunAttrs::Static as u8;
      }
      
      else if _c.str(ctx.far) == "const" {
        if attr & FunAttrs::Const as u8 != 0 {
          ctx.sum.add(Message::warn(_c, "duplicated attribute: `const`", vec![]));
        }

        attr |= FunAttrs::Const as u8;
      }

      else if _c.str(ctx.far) == "pure" {
        if attr & FunAttrs::Pure as u8 != 0 {
          ctx.sum.add(Message::warn(_c, "duplicated attribute: `pure`", vec![]));
        }

        attr |= FunAttrs::Pure as u8;
      }
      
      else {
        return Err(Message::error(_c, "unknown function modifier: `{}`", vec![_c.string(ctx.far)]));
      }
      _c = ctx.lex.get()?;
    }

    // IF: Return
    let ret = if _c.kind == WK::ArrowRigh {
      Some(Self::read_type(ctx, true)?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let this = Type::Fun{args, ret, attr};

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_struct(ctx: &mut ParserContext) -> Result<TypeId, Message> {
    let bases = Self::read_bases(ctx)?;

    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut vars = vec![];
    let mut funs = vec![];
    let mut imps = vec![];
    let mut lets = vec![];

    let mut defvis = Visibility::Private;

    loop {
      let t = ctx.lex.get()?;
      
      if t.kind == WK::CurlyBracketEnd { break; } else { ctx.lex.store(t); }

      MetaParser::read_scpvis(ctx, &mut defvis)?;
      let attrs = AttrParser::read_attr(ctx)?;
      let vis = MetaParser::read_vis(ctx, defvis)?;
      
      match Self::read_ctn(ctx, vis, CtnMask { impl_: true, vars_: true, lets_: true })? {
        Null => {},

        Fun(v) | Init(v) | Fini(v) => {
          if let Some(attr) = attrs { AttrParser::attach_attr(ctx, v, attr); }

          funs.push(v.to_any())
        }
      
        Impl(v) => imps.push(v),

        Vars(v) => vars.extend(v),

        Let(v) => lets.push(v.to_any()),
      }
    }


    let mut members = vars.clone();
    members.extend(&funs);
    members.extend(&lets);
    let scp = crate::ast::Scope::from_anys(ctx.cre, &members);
    let it = Type::Struct{ vars: ctx.cre.new_extra(vars), bases };
    let ty = ctx.cre.new_type(it);
    ctx.cre.attach_scp(ty.to_any(), scp);

    // local func impl
    if !funs.is_empty() {
      let vari = ItemVari::Impl{ type_ty: ty, ctn: ctx.cre.new_extra(funs), trait_ty: None };
      let imp = Item{vis: Visibility::Public, vari};

      let it = ctx.cre.new_item(imp);

      ctx.sides.push(it.to_any());
    }

    // inline impls
    for mut x in imps {
      if let ItemVari::Impl{type_ty, ..} = &mut x.vari {
        *type_ty = ty;
      } else { panic!() }

      ctx.sides.push( ctx.cre.new_item(x).to_any() );
    }

    Ok(ty)
  }

  pub fn read_iface(ctx: &mut ParserContext) -> Result<TypeId, Message> {
    let bases = Self::read_bases(ctx)?;
    
    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut defvis = Visibility::Public;

    let mut funs = vec![];

    loop {
      let t = ctx.lex.get()?;
      
      if t.kind == WK::CurlyBracketEnd { break; } else { ctx.lex.store(t); }

      MetaParser::read_scpvis(ctx, &mut defvis)?;
      let attrs = AttrParser::read_attr(ctx)?;
      let vis = MetaParser::read_vis(ctx, defvis)?;
      
      match Self::read_ctn(ctx, vis, CtnMask { impl_: false, vars_: false, lets_: false })? {
        Null => {},

        Fun(v) | Init(v) | Fini(v) => {
          if let Some(attr) = attrs { AttrParser::attach_attr(ctx, v, attr); }

          funs.push(v.to_any())
        }
      
        Impl(..) | Vars(..) | Let(..) => panic!(),
      }
    }

    let scp = crate::ast::Scope::from_anys(ctx.cre, &funs);
    let this = Type::Iface{ funs: ctx.cre.new_extra(funs), bases };
    let ty = ctx.cre.new_type(this);
    ctx.cre.attach_scp(ty.to_any(), scp);

    Ok(ty)
  }

  pub fn read_trait(ctx: &mut ParserContext) -> Result<TypeId, Message> {
    let bases = Self::read_bases(ctx)?;

    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut defvis = Visibility::Public;

    let mut funs = vec![];

    loop {
      let t = ctx.lex.get()?;
      
      if t.kind == WK::CurlyBracketEnd { break; } else { ctx.lex.store(t); }

      MetaParser::read_scpvis(ctx, &mut defvis)?;
      let attrs = AttrParser::read_attr(ctx)?;
      let vis = MetaParser::read_vis(ctx, defvis)?;
      
      match Self::read_ctn(ctx, vis, CtnMask { impl_: false, vars_: false, lets_: false })? {
        Null => {},

        Fun(v) | Init(v) | Fini(v) => {
          if let Some(attr) = attrs { AttrParser::attach_attr(ctx, v, attr); }

          funs.push(v.to_any())
        }
      
        Impl(..) | Vars(..) | Let(..) => panic!(),
      }
    }

    let scp = crate::ast::Scope::from_anys(ctx.cre, &funs);
    let this = Type::Trait{ funs: ctx.cre.new_extra(funs), bases };
    let ty = ctx.cre.new_type(this);
    ctx.cre.attach_scp(ty.to_any(), scp);

    Ok(ty)
  }

  pub fn read_enum(ctx: &mut ParserContext) -> Result<TypeId, Message> {
    let bases = Self::read_bases(ctx)?;

    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut vals = vec![];

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break; }
      
      t.expect_word(ctx.far)?;
      
      let _c = ctx.lex.get()?;
      match _c.kind {

        WK::Assign => {
          let val = ExprParser::read_expr(ctx, 0)?;

          let this = Thing::NamedExpr(t.save(ctx), val);

          vals.push( ctx.cre.new_thing(this).to_any() );

          let _c = ctx.lex.get()?;
          if _c.kind == WK::Comma { continue; }
          else { ctx.lex.store(_c); }
        }

        WK::Comma | WK::CurlyBracketEnd => {
          let this = Thing::Name(t.save(ctx));

          vals.push( ctx.cre.new_thing(this).to_any() );

          if _c.kind == WK::CurlyBracketEnd { break; }
        }

        _ => { _c.expect_kind3(WK::Assign, WK::Comma, WK::CurlyBracketEnd)?; },
      }
    }

    let scp = crate::ast::Scope::from_anys(ctx.cre, &vals);
    let this = Type::Enum{ vals: ctx.cre.new_extra(vals), bases };
    let ty = ctx.cre.new_type(this);
    ctx.cre.attach_scp(ty.to_any(), scp);

    Ok(ty)
  }

  pub fn read_flags(ctx: &mut ParserContext) -> Result<TypeId, Message> {
    let bases = Self::read_bases(ctx)?;

    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut vals = vec![];

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break; }
      
      t.expect_word(ctx.far)?;
      
      let _c = ctx.lex.get()?;
      match _c.kind {

        WK::Assign => {
          let val = ExprParser::read_expr(ctx, 0)?;

          let this = Thing::NamedExpr(t.save(ctx), val);

          vals.push( ctx.cre.new_thing(this).to_any() );

          let _c = ctx.lex.get()?;
          if _c.kind == WK::Comma { continue; }
          else { ctx.lex.store(_c); }
        }

        WK::Comma | WK::CurlyBracketEnd => {
          let this = Thing::Name(t.save(ctx));

          vals.push( ctx.cre.new_thing(this).to_any() );

          if _c.kind == WK::CurlyBracketEnd { break; }
        }

        _ => { _c.expect_kind3(WK::Assign, WK::Comma, WK::CurlyBracketEnd)?; },
      }
    }

    let scp = crate::ast::Scope::from_anys(ctx.cre, &vals);
    let this = Type::Flags{ vals: ctx.cre.new_extra(vals), bases };
    let ty = ctx.cre.new_type(this);
    ctx.cre.attach_scp(ty.to_any(), scp);

    Ok(ty)
  }

}

