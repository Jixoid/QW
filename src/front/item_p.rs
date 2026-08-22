use crate::ast::{self, AstId, Thing};
use crate::lexer::WK;
use crate::{ast::{DeclVari, Item, ItemId, ItemVari, Module, Visibility}, diagnostic::Message, front::{Front, ParserContext, attr_p::AttrParser, decl_p::DeclParser, meta_p::MetaParser, type_p::TypeParser}, lexer::Lexer};


pub struct ItemParser {}

impl ItemParser {

  pub fn read_generic(ctx: &mut ParserContext, vis: Visibility) -> Result<ItemId, Message> {
    ctx.lex.get()?.expect_kind(WK::AngleBeg)?;
    
    let mut params = vec![];
    'ml: loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::AngleEnd { break 'ml; } else { ctx.lex.store(t); }
      
      let mut names = Vec::new();
      're: loop {
        let name = ctx.lex.get()?;
        names.push(name);
        
        let c = ctx.lex.get()?;
        if c.kind == WK::Comma { continue 're; }
        else if c.kind == WK::Colon { break 're; }
        else { c.expect_kind2(WK::Colon, WK::Comma)?; }
      }
      
      let kind = TypeParser::read_type(ctx, true)?;
      
      for x in names { params.push( ctx.cre.new_thing(Thing::NamedType(x.save(), kind)).to_any() ); }
      
      let e = ctx.lex.get()?;
      if e.kind == WK::Comma || e.kind == WK::Semicolon { continue 'ml; }
      else if e.kind == WK::AngleEnd { break 'ml; }
      else { return Err(Message::error(e.save(), "expected `,`, `;` or `>`", vec![])); }
    }


    let mut reqs = vec![];
    
    let _c = ctx.lex.get()?;
    if _c.str(ctx.far) == "requires" {

      loop {
        let t = ctx.lex.get()?;

        if t.kind != WK::Word {
          ctx.lex.store(t);
          break;
        }

        ctx.lex.get()?.expect_kind(WK::Colon)?;

        let mut tys = vec![];

        loop {
          tys.push(TypeParser::read_type(ctx, true)?.to_any());

          let _c = ctx.lex.get()?;
          if _c.kind == WK::BitwiseOr { continue; }
          else if _c.kind == WK::Semicolon { break; }
          else {
            _c.expect_kind2(WK::BitwiseOr, WK::Semicolon)?;
          }
        }

        let rng = ctx.cre.new_extra(tys);

        reqs.push( ctx.cre.new_thing(Thing::NamedTypeList(t.save(), rng)).to_any() );
      }

    } else {
      ctx.lex.store(_c);
    }

    
    let mut ctn = vec![];
    
    let _c = ctx.lex.get()?;
    if _c.kind == WK::CurlyBracketBeg {
      loop {
        let nxt = ctx.lex.get()?;
        if nxt.kind == WK::CurlyBracketEnd { break; }
        ctx.lex.store(nxt);
        
        let mut defvis = vis.clone();
        let id = Front::read(ctx, &mut defvis)?;
        
        ctn.push(id);
        ctn.extend(ctx.sides.drain(..));
      }
    } else {
      ctx.lex.store(_c);
      let mut defvis = vis.clone();
      let id = Front::read(ctx, &mut defvis)?;

      ctn.push(id);
      ctn.extend(ctx.sides.drain(..));
    }
    

    let this = ItemVari::Generic{
      params: ctx.cre.new_extra(params),
      ctn: ctx.cre.new_extra(ctn),
      reqs: ctx.cre.new_extra(reqs),
    };
    
    let it = Item{vari: this, vis};

    Ok(ctx.cre.new_item(it))
  }

  pub fn read_impl(ctx: &mut ParserContext, vis: Visibility) -> Result<ItemId, Message> {
    let type_ty = TypeParser::read_type(ctx, false)?;
    
    let _c = ctx.lex.get()?;
    let trait_ty = if _c.kind == WK::Colon {
      TypeParser::read_type(ctx, false)?
    } else {
      ctx.lex.store(_c);
      AstId::null()
    };

    
    ctx.lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let mut impls = Vec::new();
    let mut defvis = Visibility::Private;

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break; } else { ctx.lex.store(t); }

      MetaParser::read_scpvis(ctx, &mut defvis)?;
      let attrs = AttrParser::read_attr(ctx)?;
      let vis = MetaParser::read_vis(ctx, defvis)?;
      
      let next_kw = ctx.lex.get()?;
      if next_kw.kind == WK::Fun {
        let fun = DeclParser::read_fun(ctx, vis)?;
        
        if let DeclVari::Fun { kind: _, blok } = ctx.cre.get_decl(fun).vari {
          if blok.is_none() {
            return Err(Message::error(next_kw.save(), "extend methods must have a body (cannot end with `;`)", vec![]));
          }
        }
        
        if let Some(a) = attrs { AttrParser::attach_attr(ctx, fun, a); }

        impls.push(fun.to_any());
      } else {
        return Err(Message::error(next_kw.save(), "expected `fun`, got `{}`", vec![ next_kw.string(ctx.far) ]));
      }
    }

    let this = ItemVari::Impl{
      type_ty,
      trait_ty: Some(trait_ty),
      ctn: ctx.cre.new_extra(impls),
    };
    
    let it = Item{vis, vari: this};

    Ok(ctx.cre.new_item(it))
  }
  
  pub fn read_use(ctx: &mut ParserContext, vis: Visibility) -> Result<ItemId, Message> {
    let _c = ctx.lex.get()?;
    
    let start = match _c.kind {
      WK::Crate => ctx.cre.new_thing(Thing::Crate),
      WK::Super => ctx.cre.new_thing(Thing::Super),

      WK::Word => ctx.cre.new_thing(Thing::Name(_c.save())),

      _ => return Err(Message::error(_c.save(), "unknown use starter: {}", vec![ _c.string(ctx.far) ])),
    };

    let mut all = vec![ start.to_any() ];
    
    loop {
      let _c = ctx.lex.get()?;
      if _c.kind == WK::Scope {}
      else if _c.kind == WK::Semicolon { break; }
      else {
        _c.expect_kind2(WK::Scope, WK::Semicolon)?;
      }

      all.push(Self::read_use_sub(ctx)?.to_any());
    }


    let this = Item{vis, vari: ItemVari::Import( ctx.cre.new_extra(all) )};

    Ok(ctx.cre.new_item(this))
  }

  fn read_use_sub(ctx: &mut ParserContext) -> Result<ast::ThingId, Message> {
    let _c = ctx.lex.get()?;

    let ret = match _c.kind {
      WK::Crate => ctx.cre.new_thing(Thing::Crate),
      WK::Super => ctx.cre.new_thing(Thing::Super),

      WK::Word => ctx.cre.new_thing(Thing::Name(_c.save())),

      _ => return Err(Message::error(_c.save(), "unknown use starter: {}", vec![ _c.string(ctx.far) ])),
    };

    Ok(ret)
  }


  pub fn read_mod(ctx: &mut ParserContext, vis: Visibility) -> Result<Option<ItemId>, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;
    ctx.lex.get()?.expect_kind(WK::Semicolon)?;

    let mod_name = name.string(ctx.far);
    let current_dir = std::path::Path::new(&ctx.lex.mol.fpath).parent().unwrap_or(std::path::Path::new(""));
    let mut mod_path = current_dir.join(format!("{}.qw", mod_name));

    if !mod_path.exists() {
      mod_path = current_dir.join(&mod_name).join("mod.qw");
    }

    let fpath_str = mod_path.to_str().unwrap_or("").to_string();
    if ctx.far.is_loaded(&fpath_str) {
      return Ok(None);
    }

    let mfd = match Module::new(mod_path.to_str().unwrap()) {
      Ok(s) => s,
      Err(..) => return Err(Message::error(name.save(), "module file not found `{}.qw` or `{}/mod.qw`", vec![ mod_name.clone(), mod_name ])),
    };

    let mfd_static = ctx.far.alloc(mfd);

    let mut lex = Lexer::new(mfd_static);
    
    let mut new_ctx = ParserContext {
      lex: &mut lex,
      sum: ctx.sum,
      cre: ctx.cre,
      far: ctx.far,
      sides: vec![],
    };
    
    let mut anys = Vec::new();
    let mut defvis = Visibility::Private;
    
    loop {
      match new_ctx.lex.lex() {
        None => break,
        Some(t) => new_ctx.lex.store(t),
      }

      match Front::read(&mut new_ctx, &mut defvis) {
        Ok(id) => {
          anys.push(id);
          anys.extend(new_ctx.sides.drain(..));
        }
        Err(e) => {
          new_ctx.sum.add(e);
          let _ = MetaParser::pmr_global(&mut new_ctx);
        }
      }
    }

    let this = Item{vis, vari: ItemVari::Module{name: mod_name, ctn: ctx.cre.new_extra(anys)}};
    let mod_id = ctx.cre.new_item(this);

    Ok(Some(mod_id))
  }

}
