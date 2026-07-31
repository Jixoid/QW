use crate::{ast::*, control::identy::IdentyId, diagnostic::Message, front::{ParserContext, expr_p::ExprParser, meta_p::MetaParser, type_p::TypeParser}};


pub struct DeclParser {}

impl DeclParser {

  pub fn read_decl<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, defvis: &mut Visibility) -> Result<Vec<IdentyId>,  Message<'a>> {
    let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
    let v = MetaParser::read_visibility(ctx, defvis)?;
    
    let l = ctx.lex.get()?;
    let d = match l.str() {
      "fun"    => Self::read_fun(ctx, v, false),
      "using"  => Self::read_using(ctx, v),
      "struct" => Self::read_struct(ctx, v),
      "iface"  => Self::read_iface(ctx, v),
      "enum"   => Self::read_enum(ctx, v),
      "flags"  => Self::read_flags(ctx, v),
      "let"    => Self::read_var(ctx, v, AccessKind::IMM),
      "var"    => Self::read_var(ctx, v, AccessKind::MUT),
      "use"    => return Self::read_use(ctx, v),
      "mod"    => return Self::read_mod(ctx, v),
      _ => return Err(Message::error(l, String::from("unknown keyword: `{}`"), vec![l.string()])),
    }?;

    let id = ctx.mol.new_decl(d);
    crate::front::attr_p::AttrParser::attach_attributes(ctx, id, attrs);
    Ok(vec![id])
  }


  pub fn read_fun<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility, in_struct: bool) -> Result<Decl<'a>,  Message<'a>> {
    let name = ctx.lex.get()?;

    let args = MetaParser::read_fun_args(ctx)?;

    let mut is_static = false;
    let mut is_const = false;
    let mut _c = ctx.lex.get()?;
    
    // Parse function modifiers
    while _c.str() != "->" && _c.str() != "{" && _c.str() != "`" && _c.str() != ";" {
      if _c.str() == "static" {
        if !in_struct {
          return Err(Message::error(_c, String::from("`static` modifier is only allowed inside structs"), vec![]));
        }
        is_static = true;
      } else if _c.str() == "const" {
        if !in_struct {
          return Err(Message::error(_c, String::from("`const` modifier is only allowed inside structs"), vec![]));
        }
        if is_static {
          return Err(Message::error(_c, String::from("a function cannot be both `static` and `const`"), vec![]));
        }
        is_const = true;
      } else {
        return Err(Message::error(_c, String::from("unknown function modifier: `{}`"), vec![_c.string()]));
      }
      _c = ctx.lex.get()?;
    }
    
    let ret = if _c.str() == "->" {
      TypeParser::read_type(ctx, true)?
    } else {
      ctx.lex.store(_c);
      ctx.mol.localize(&ctx.mi.sys.mol, ctx.mi.sys.ty_void)
    };

    let _c = ctx.lex.get()?;

    let blok = if _c.str() == ";" {
      IdentyId::null()
    }
    else if _c.str() == "{" || _c.str() == "`" {
      ctx.lex.store(_c);
      ExprParser::read_block(ctx)?
    }
    else {
      MetaParser::expect_equal2_w(_c, "{", ";")?;
      IdentyId::null()
    };


    let ty = TypeVari::Function(FunType{
      args,
      is_static,
      is_const,
      ret
    });
    let kind = ctx.mol.new_type(Type{state: crate::ast::TypeState::Unresolved, vari: ty});


    let this = DeclVari::Fun(FunDecl{
      kind, blok
    });

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_using<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>, Message<'a>> {
    let name = ctx.lex.get()?;
    MetaParser::expect_equal(ctx, "=")?;
    let ty = TypeParser::read_type(ctx, false)?;
    MetaParser::expect_equal(ctx, ";")?;

    let this = DeclVari::Using(
      ty
    );

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_use<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Vec<IdentyId>, Message<'a>> {
    let mut base_path = Vec::new();
    let mut w = ctx.lex.get()?;
    
    // Parse the base path
    loop {
      let is_word = match w.kind {
        crate::lexer::WordKind::Word => true,
        _ => false,
      };
      
      if !is_word && w.str() != "*" && w.str() != "{" {
        return Err(Message::error(w, String::from("expected identifier, `*` or `{` in use path"), vec![]));
      }
      
      if w.str() == "*" {
        // wildcard import
        MetaParser::expect_equal(ctx, ";")?;
        
        let name = w;
        let this = DeclVari::ImportWildcard(base_path, None);
        let decl = Decl::new(name, this, vis);
        let id = ctx.mol.new_decl(decl);
        return Ok(vec![id]);
      }
      
      if w.str() == "{" {
        // multiple imports block
        let mut ids = Vec::new();
        loop {
          let end_w = ctx.lex.get()?;
          if end_w.str() == "}" { break; }
          
          if end_w.kind != crate::lexer::WordKind::Word {
            return Err(Message::error(end_w, String::from("expected identifier in use block"), vec![]));
          }
          
          let name = end_w;
          let mut full_path = base_path.clone();
          let name_idx = ctx.mol.nick_map.iter().position(|r| r == name.str()).unwrap_or_else(|| {
            ctx.mol.nick_map.push(name.str().to_string());
            ctx.mol.nick_map.len() - 1
          }) as u32;
          
          full_path.push((name_idx, name));
          
          let this = DeclVari::Import(full_path, None);
          let decl = Decl::new(name, this, vis);
          ids.push(ctx.mol.new_decl(decl));
          
          let next_w = ctx.lex.get()?;
          if next_w.str() == "}" { break; }
          if next_w.str() != "," {
            return Err(Message::error(next_w, String::from("expected `,` or `}`"), vec![]));
          }
        }
        MetaParser::expect_equal(ctx, ";")?;
        return Ok(ids);
      }
      
      // it's an identifier
      let name_idx = ctx.mol.nick_map.iter().position(|r| r == w.str()).unwrap_or_else(|| {
        ctx.mol.nick_map.push(w.str().to_string());
        ctx.mol.nick_map.len() - 1
      }) as u32;
      
      base_path.push((name_idx, w));
      
      let next_w = ctx.lex.get()?;
      if next_w.str() == ";" {
        let name = base_path.last().unwrap().1;
        let this = DeclVari::Import(base_path, None);
        let decl = Decl::new(name, this, vis);
        let id = ctx.mol.new_decl(decl);
        return Ok(vec![id]);
      } else if next_w.str() == "::" {
        w = ctx.lex.get()?;
      } else {
        return Err(Message::error(next_w, String::from("expected `::` or `;` in use path"), vec![]));
      }
    }
  }

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>, Message<'a>> {
    let name = ctx.lex.get()?;
    let tyid = TypeParser::read_struct(ctx)?;

    let this = DeclVari::Using(tyid);

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_iface<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>, Message<'a>> {
    let name = ctx.lex.get()?;
    let tyid = TypeParser::read_iface(ctx)?;

    let this = DeclVari::Using(tyid);

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_enum<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>, Message<'a>> {
    let name = ctx.lex.get()?;
    let tyid = TypeParser::read_enum(ctx)?;

    let this = DeclVari::Using(tyid);

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_flags<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>, Message<'a>> {
    let name = ctx.lex.get()?;
    let tyid = TypeParser::read_flags(ctx)?;

    let this = DeclVari::Using(tyid);

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_var<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility, acck: AccessKind) -> Result<Decl<'a>,  Message<'a>> {
    let name = ctx.lex.get()?;

    let t1 = ctx.lex.get()?;
    let mut ty = IdentyId::new(crate::control::IdentyKind::Null, 0, 0);

    if t1.str() == ":" {
      ty = TypeParser::read_type(ctx, true)?;
    } else {
      ctx.lex.store(t1);
    }

    let mut init = None;
    let t2 = ctx.lex.get()?;

    if t2.str() == "=" {
      init = Some(ExprParser::read_expr(ctx)?);
      MetaParser::expect_equal(ctx, ";")?;
    }
    else if t2.str() == ";" { /* no init */ }
    else {
      MetaParser::expect_equal2_w(t2, "=", ";")?;
    }

    let this = DeclVari::Var(VarDecl{
      kind: ty,
      init,
      acck,
      comptime: false,
    });

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_mod<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Vec<IdentyId>,  Message<'a>> {
    let name = ctx.lex.get()?;
    MetaParser::expect_equal(ctx, ";")?;

    let mod_name = name.string();
    let current_dir = std::path::Path::new(&ctx.lex.mol.fpath).parent().unwrap_or(std::path::Path::new(""));
    let mut mod_path = current_dir.join(format!("{}.qw", mod_name));

    if !mod_path.exists() {
      mod_path = current_dir.join(&mod_name).join("mod.qw");
    }

    let fpath_str = mod_path.to_str().unwrap_or("").to_string();
    if ctx.mi.farena.is_loaded(&fpath_str) {
      return Ok(vec![]);
    }

    let mmap = match std::fs::read_to_string(&mod_path) {
      Ok(s) => s,
      Err(e) => return Err(Message::error(name, format!("failed to load module {}: {}", mod_name, e), vec![])),
    };

    let mfd = crate::control::module::ModuleFile {
      fpath: mod_path.to_str().unwrap().to_string(),
      mmap,
      kind: crate::control::module::ModuleKind::Regular,
    };
    let mfd_static: &'d crate::control::module::ModuleFile = ctx.mi.farena.alloc(mfd);

    let mut lex = crate::lexer::Lexer::new_module(mfd_static);
    
    let mut new_ctx = ParserContext {
      lex: &mut lex,
      mol: ctx.mol,
      sum: ctx.sum,
      mi: ctx.mi,
    };
    
    let mut decls = Vec::new();
    let mut defvis = Visibility::Private;
    
    loop {
      let t = new_ctx.lex.lex();
      if t.kind == crate::lexer::WordKind::EOF { break; }
      else {
        new_ctx.lex.store(t);
        match DeclParser::read_decl(&mut new_ctx, &mut defvis) {
          Ok(ids) => {
            for id in ids {
              decls.push(id);
            }
          },
          Err(e) => {
            new_ctx.sum.add(e);
            let _ = MetaParser::pmr_global(&mut new_ctx);
          }
        }
      }
    }

    let this = DeclVari::Module(ModuleDecl { decls });
    let decl = Decl::new(name, this, vis);
    let mod_id = ctx.mol.new_decl(decl);

    ctx.mol.add_to_module(mod_id);

    Ok(vec![])
  }

}
