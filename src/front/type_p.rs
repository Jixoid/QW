use crate::ast::{AstId, Item, ItemVari, Thing};
use crate::front::expr_p::ExprParser;
use crate::lexer::WK;
use crate::{ast::{AccessKind, DeclVari, FieldType, NickType, Type, TypeId, Visibility}, diagnostic::Message, front::{ParserContext, attr_p::AttrParser, decl_p::DeclParser, meta_p::MetaParser}, lexer::Word};


pub struct TypeParser {}

impl TypeParser {

  pub fn read_type<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, indecl: bool) -> Result<TypeId, Message> {
    let attrs = AttrParser::read_attr(ctx)?;
    let _n = ctx.lex.get()?;

    let tyid = match _n.str(ctx.far) {
      "struct" => Self::read_struct(ctx)?,
      "fun"    => Self::read_fun(ctx)?,
      "iface"  => Self::read_iface(ctx)?,
      "trait"  => Self::read_trait(ctx)?,
      "enum"   => Self::read_enum(ctx)?,
      "flags"  => Self::read_flags(ctx)?,
      
      ".." => {
        let sub = Self::read_type(ctx, indecl)?;
        
        ctx.cre.new_type(Type::Range{ sub })
      }

      "?" => {
        let sub = Self::read_type(ctx, indecl)?;

        ctx.cre.new_type(Type::Option{ sub })
      }

      "&" => {
        let nxt = ctx.lex.get()?;
        let acc = match nxt.str(ctx.far) {
          "mut" => AccessKind::MUT,
          "imm" => AccessKind::IMM,
          _ => {
            ctx.lex.store(nxt);
            AccessKind::IMM
          }
        };
        
        let sub = Self::read_type(ctx, indecl)?;
        
        ctx.cre.new_type(Type::Ref{ sub, acc })
      }
      
      "^" => {
        let nxt = ctx.lex.get()?;
        let acc = match nxt.str(ctx.far) {
          "mut" => AccessKind::MUT,
          "imm" => AccessKind::IMM,
          _ => {
            ctx.lex.store(nxt);
            AccessKind::IMM
          }
        };

        let sub = Self::read_type(ctx, indecl)?;
        
        ctx.cre.new_type(Type::Ptr{ sub, acc })
      }
      
      "[" => {
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

      "(" => {
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
        let nick = Type::Nick(NickType{pos: _n.expect_word(ctx.far)?.save(), idx: ctx.cre.get_str(_n.str(ctx.far))});
        let mut path = vec![ ctx.cre.new_type(nick).to_any() ];

        loop {
          let _c = ctx.lex.get()?;
          
          match _c.str(ctx.far) {
            "::" => {
              let _n = ctx.lex.get()?.expect_word(ctx.far)?;
              let sub = Type::Nick(NickType{pos: _n.save(), idx: ctx.cre.get_str(_n.str(ctx.far))});
              
              path.push( ctx.cre.new_type(sub).to_any() );
            }

            "<" => {
              let mut args = vec![];
              loop {
                let sub = Self::read_type(ctx, indecl)?;
                args.push(sub);
                
                let sep = ctx.lex.get()?;
                if sep.kind == WK::AngleEnd { break; }
                if sep.kind == WK::Comma { continue; }
                else {
                  sep.expect_kind2(WK::Comma, WK::AngleEnd)?;
                }
              }

              let base = Type::Path( ctx.cre.new_extra(path) );
              let base_id = ctx.cre.new_type(base);

              let ty = Type::Specialize{base: base_id, args};
              path = vec![ ctx.cre.new_type(ty).to_any() ];
            }

            _ => {ctx.lex.store(_c); break},
          }
        }


        if path.len() == 1 { TypeId::new_from(path[0]) } else { let rng = ctx.cre.new_extra(path);  ctx.cre.new_type(Type::Path(rng)) }
      }
    };

    AttrParser::attach_attr(ctx, tyid, attrs);

    Ok(tyid)
  }


  pub fn read_fun<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message> {
    let args = MetaParser::read_fun_args(ctx)?;

    let _c = ctx.lex.get()?;

    let ret = if _c.kind == WK::ArrowRigh {
      Some(Self::read_type(ctx, true)?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let this = Type::Fun{args, ret};

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message> {
    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut vars = vec![];
    let mut funs = vec![];
    let mut imps = vec![];

    let mut defvis = Visibility::Public;

    'ml: loop {
      let t = ctx.lex.get()?;
      
      if t.kind == WK::CurlyBracketEnd { break 'ml; } else { ctx.lex.store(t); }

      //let attrs = AttrParser::read_attr(ctx)?;
      let vis = MetaParser::read_visibility(ctx, &mut defvis)?;
      

      let kw = ctx.lex.get()?;
      if kw.kind == WK::Fun { // object fun
        let fun = DeclParser::read_fun(ctx, vis, true)?;

        if let DeclVari::Fun { kind: _, blok } = ctx.cre.get_decl(fun).vari {
          if blok.is_null() {
            return Err(Message::error(kw, "struct methods must have a body (cannot end with `;`)".to_string(), vec![]));
          }
        } else {
          panic!("must be decl::fun")
        }
        
        funs.push(fun);
        continue 'ml;
      } else if kw.kind == WK::Init { // object init
        let fun = DeclParser::read_init(ctx, vis)?;

        if let DeclVari::Fun { kind: _, blok } = ctx.cre.get_decl(fun).vari {
          if blok.is_null() {
            return Err(Message::error(kw, "struct methods must have a body (cannot end with `;`)".to_string(), vec![]));
          }
        } else {
          panic!("must be decl::fun")
        }
        
        funs.push(fun);
        continue 'ml;
      } else if kw.kind == WK::Impl { // trait impl
        ctx.lex.get()?.expect_kind(WK::Colon)?;

        let trait_ty = TypeParser::read_type(ctx, true)?;
        
        let _c = ctx.lex.get()?;
        let onedef = if _c.kind == WK::CurlyBracketBeg { false } else { ctx.lex.store(_c); true };

        let mut impls = Vec::new();
        let mut defvis = defvis;

        loop {
          if !onedef {
            let t = ctx.lex.get()?;
            if t.kind == WK::CurlyBracketEnd { break; } else { ctx.lex.store(t); }
          }

          let attrs = AttrParser::read_attr(ctx)?;
          let v = MetaParser::read_visibility(ctx, &mut defvis)?;
          
          let next_kw = ctx.lex.get()?;
          if next_kw.kind == WK::Fun {
            let fun = DeclParser::read_fun(ctx, v, true)?;
            
            if let DeclVari::Fun { kind: _, blok } = ctx.cre.get_decl(fun).vari {
              if blok.is_null() {
                return Err(Message::error(next_kw, "extend methods must have a body (cannot end with `;`)".to_string(), vec![]));
              }
            }
            
            AttrParser::attach_attr(ctx, fun, attrs);
            impls.push(fun);
          } else {
            return Err(Message::error(next_kw, format!("expected `fun`, got `{}`", next_kw.str(ctx.far)), vec![]));
          }

          if onedef { break; }
        }

        let imp = Item{vis, vari: ItemVari::Impl{
          trait_ty,
          type_ty: AstId::null(),
          ctn: impls,
        }};

        imps.push(imp);

        continue 'ml;
      } else {
        ctx.lex.store(kw);
      }

      let mut names: Vec<Word> = vec![];

      're: loop {
        let name = ctx.lex.get()?.expect_word(ctx.far)?;
        names.push(name);

        let c = ctx.lex.get()?;

        if c.kind == WK::Comma { continue 're; }
        else if c.kind == WK::Colon { break 're; }
        else {
          c.expect_kind2(WK::Colon, WK::Comma)?;
        }
      }

      let kind = TypeParser::read_type(ctx, true)?;
      
      for x in names {
        vars.push(FieldType{name: x.save(), kind, vis, attrs: Vec::new()}); // attrs.clone()});
      }
      
      let e = ctx.lex.get()?;

      if e.kind == WK::Semicolon { continue 'ml; }
      else if e.kind == WK::CurlyBracketEnd { break 'ml; }
      else {
        e.expect_kind2(WK::Semicolon, WK::CurlyBracketEnd)?;
      }
    }


    let ty = ctx.cre.new_type(Type::Struct{vars});

    // local func impl
    if !funs.is_empty() {
      let vari = ItemVari::Impl{type_ty: ty, ctn: funs, trait_ty: AstId::null()};
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

  pub fn read_iface<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message> {
    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut funs = vec![];

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break 'ml; } else { ctx.lex.store(t); }
      
      let fn_kw = ctx.lex.get()?;
      if fn_kw.kind != WK::Fun {
        return Err(Message::error(fn_kw, String::from("expected `fun` keyword in iface, found `{}`"), vec![fn_kw.string(ctx.far)]));
      }

      let name = ctx.lex.get()?;
      let args = MetaParser::read_fun_args(ctx)?;
      
      let _c = ctx.lex.get()?;
      let ret = if _c.kind == WK::ArrowRigh {
        Some(TypeParser::read_type(ctx, true)?)
      } else {
        ctx.lex.store(_c);
        None
      };

      ctx.lex.get()?.expect_kind(WK::Semicolon)?;

      let this = Type::Fun{ args, ret };
      let kind = ctx.cre.new_type(this);

      funs.push(FieldType{name: name.save(), kind, vis: Visibility::Public, attrs: vec![]});
    }

    let this = Type::Iface{ funs };

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_trait<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message> {
    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut funs = vec![];

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break 'ml; } else { ctx.lex.store(t); }
      
      let fn_kw = ctx.lex.get()?;
      if fn_kw.kind != WK::Fun {
        return Err(Message::error(fn_kw, String::from("expected `fun` keyword in iface, found `{}`"), vec![fn_kw.string(ctx.far)]));
      }

      let name = ctx.lex.get()?;
      let args = MetaParser::read_fun_args(ctx)?;
      
      let _c = ctx.lex.get()?;
      let ret = if _c.kind == WK::ArrowRigh {
        Some(TypeParser::read_type(ctx, true)?)
      } else {
        ctx.lex.store(_c);
        None
      };

      ctx.lex.get()?.expect_kind(WK::Semicolon)?;

      let ty = Type::Fun{ args, ret };
      let kind = ctx.cre.new_type(ty);

      funs.push(FieldType{name: name.save(), kind, vis: Visibility::Public, attrs: vec![]});
    }

    let this = Type::Trait{ funs };

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_enum<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message> {
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

          vals.push( ctx.cre.new_thing(Thing::NamedExpr(t.save(), val)).to_any() );
        }

        WK::Comma | WK::CurlyBracketEnd => {
          vals.push( ctx.cre.new_thing(Thing::Name(t.save())).to_any() );

          if _c.kind == WK::CurlyBracketEnd { break; }
        }

        _ => { _c.expect_kind3(WK::Assign, WK::Comma, WK::CurlyBracketEnd)?; },
      }
    }

    let this = Type::Enum{ vals: ctx.cre.new_extra(vals) };

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_flags<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message> {
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

          vals.push( ctx.cre.new_thing(Thing::NamedExpr(t.save(), val)).to_any() );
        }

        WK::Comma | WK::CurlyBracketEnd => {
          vals.push( ctx.cre.new_thing(Thing::Name(t.save())).to_any() );

          if _c.kind == WK::CurlyBracketEnd { break; }
        }

        _ => { _c.expect_kind3(WK::Assign, WK::Comma, WK::CurlyBracketEnd)?; },
      }
    }

    let this = Type::Flags{ vals: ctx.cre.new_extra(vals) };

    Ok(ctx.cre.new_type(this))
  }

}
