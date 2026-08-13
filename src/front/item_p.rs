use crate::ast::AstId;
use crate::lexer::WK;
use crate::{ast::{DeclVari, GenericItem, Item, ItemId, ItemVari, Module, ModuleItem, Visibility}, diagnostic::Message, front::{Front, ParserContext, attr_p::AttrParser, decl_p::DeclParser, meta_p::MetaParser, type_p::TypeParser}, lexer::{Lexer, WordKind}};


pub struct ItemParser {}

impl ItemParser {

  pub fn read_generic<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<ItemId, Message<'a>> {
    ctx.lex.get()?.expect_equal_str("<")?;
    
    let mut params = Vec::new();
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
        else { c.expect_equal_str2(":", ",")?; }
      }
      
      let kind = TypeParser::read_type(ctx, true)?;
      
      for x in names { params.push(crate::ast::types::FieldType{name: x, kind, vis: Visibility::Private, attrs: vec![]}); }
      
      let e = ctx.lex.get()?;
      if e.kind == WK::Comma || e.kind == WK::Semicolon { continue 'ml; }
      else if e.kind == WK::AngleEnd { break 'ml; }
      else { return Err(Message::error(e, String::from("expected `,`, `;` or `>`"), vec![])); }
    }


    let mut reqs = vec![];
    
    let _c = ctx.lex.get()?;
    if _c.str() == "requires" {

      loop {
        let t = ctx.lex.get()?;

        if t.kind != WK::Word {
          ctx.lex.store(t);
          break;
        }

        ctx.lex.get()?.expect_equal_str(":")?;

        let mut tys = vec![];

        loop {
          let sub = TypeParser::read_type(ctx, true)?;
          tys.push(sub);

          let _c = ctx.lex.get()?;
          if _c.kind == WK::BitwiseOr { continue; }
          else if _c.kind == WK::Semicolon { break; }
          else {
            _c.expect_equal_str2("|", ";")?;
          }
        }

        reqs.push((t, tys));
      }

    } else {
      ctx.lex.store(_c);
    }

    
    let mut ctn = Vec::new();
    
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
    

    let this = ItemVari::Generic(GenericItem{params, ctn, reqs});
    
    let it = Item{vari: this, vis};

    Ok(ctx.cre.new_item(it))
  }

  pub fn read_impl<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<ItemId, Message<'a>> {
    let type_ty = TypeParser::read_type(ctx, false)?;
    
    let _c = ctx.lex.get()?;
    let trait_ty = if _c.kind == WK::Colon {
      TypeParser::read_type(ctx, false)?
    } else {
      ctx.lex.store(_c);
      AstId::null()
    };

    
    ctx.lex.get()?.expect_equal_str("{")?;

    let mut impls = Vec::new();
    let mut defvis = Visibility::Private;

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break; } else { ctx.lex.store(t); }

      let attrs = AttrParser::read_attr(ctx)?;
      let v = MetaParser::read_visibility(ctx, &mut defvis)?;
      
      let next_kw = ctx.lex.get()?;
      if next_kw.kind == WK::Fun {
        let fun = DeclParser::read_fun(ctx, v, true)?;
        
        if let DeclVari::Fun(ref f) = ctx.cre.get_decl(fun).vari {
          if f.blok.is_null() {
            return Err(Message::error(next_kw, "extend methods must have a body (cannot end with `;`)".to_string(), vec![]));
          }
        }
        
        AttrParser::attach_attr(ctx, fun, attrs);
        impls.push(fun);
      } else {
        return Err(Message::error(next_kw, format!("expected `fun`, got `{}`", next_kw.str()), vec![]));
      }
    }

    let this = ItemVari::Impl{
      type_ty,
      trait_ty,
      ctn: impls,
    };
    
    let it = Item{vis, vari: this};

    Ok(ctx.cre.new_item(it))
  }
  
  pub fn read_use<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Vec<ItemId>, Message<'a>> {
    let mut base_path = Vec::new();
    let mut w = ctx.lex.get()?;
    
    // Parse the base path
    loop {
      let is_word = match w.kind { WordKind::Word => true, _ => false };
      

      if !is_word && w.kind != WK::Mul && w.kind != WK::CurlyBracketBeg {
        return Err(Message::error(w, String::from("expected identifier, `*` or `{` in use path"), vec![]));
      }
      
      if w.kind == WK::Mul {
        // wildcard import
        ctx.lex.get()?.expect_equal_str(";")?;
        
        let this = Item{vis, vari: ItemVari::ImportWildcard(base_path, None)};
        
        let id = ctx.cre.new_item(this);
        return Ok(vec![id]);
      }
      
      if w.kind == WK::CurlyBracketBeg {
        // multiple imports block
        let mut ids = Vec::new();
        loop {
          let end_w = ctx.lex.get()?;
          if end_w.kind == WK::CurlyBracketEnd { break; }
          
          if end_w.kind != crate::lexer::WordKind::Word {
            return Err(Message::error(end_w, String::from("expected identifier in use block"), vec![]));
          }
          
          let name = end_w;
          let mut full_path = base_path.clone();
          
          full_path.push((ctx.cre.get_str(name.str()), name));
          
          let this = Item{vis, vari: ItemVari::Import(full_path, None)};
          ids.push(ctx.cre.new_item(this));
          
          let next_w = ctx.lex.get()?;
          if next_w.kind == WK::CurlyBracketEnd { break; }
          if next_w.kind != WK::Comma {
            return Err(Message::error(next_w, String::from("expected `,` or `}`"), vec![]));
          }
        }
        ctx.lex.get()?.expect_equal_str(";")?;
        return Ok(ids);
      }
      
      // it's an identifier
      base_path.push((ctx.cre.get_str(w.str()), w));
      
      let next_w = ctx.lex.get()?;
      if next_w.kind == WK::Semicolon {
        let this = Item{vis, vari: ItemVari::Import(base_path, None)};

        let id = ctx.cre.new_item(this);
        return Ok(vec![id]);
      } else if next_w.kind == WK::Scope {
        w = ctx.lex.get()?;
      } else {
        return Err(Message::error(next_w, String::from("expected `::` or `;` in use path"), vec![]));
      }
    }
  }

  pub fn read_mod<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Option<ItemId>, Message<'a>> {
    let name = ctx.lex.get()?;
    ctx.lex.get()?.expect_equal_str(";")?;

    let mod_name = name.string();
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
      Err(..) => return Err(Message::error(name, format!("module file not found `{}.qw` or `{}/mod.qw`", mod_name, mod_name), vec![])),
    };

    let mfd_static = ctx.far.alloc(mfd);

    let mut lex = Lexer::new_module(mfd_static);
    
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
      let t = new_ctx.lex.lex();
      if t.kind == crate::lexer::WordKind::EOF { break; }
      else {
        new_ctx.lex.store(t);
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
    }

    let this = Item{vis, vari: ItemVari::Module(ModuleItem{name: mod_name, ctn: anys})};
    let mod_id = ctx.cre.new_item(this);

    Ok(Some(mod_id))
  }

}
