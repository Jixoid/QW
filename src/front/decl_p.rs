use crate::{ast::*, control::identy::IdentyId, diagnostic::Message, front::{ParserContext, expr_p::ExprParser, meta_p::MetaParser, type_p::TypeParser}};


pub struct DeclParser {}

impl DeclParser {

  pub fn read_decl<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, defvis: &mut Visibility) -> Result<IdentyId,  Message<'a>> {
    let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
    let v = MetaParser::read_visibility(ctx, defvis)?;
    
    let l = ctx.lex.get()?;
    let d = match l.str() {
      "fun"    => Self::read_fun(ctx, v),
      "using"  => Self::read_using(ctx, v),
      "struct" => Self::read_struct(ctx, v),
      "let"    => Self::read_var(ctx, v, AccessKind::IMM),
      "var"    => Self::read_var(ctx, v, AccessKind::MUT),
      "mod"    => Self::read_mod(ctx, v),
      _ => return Err(Message::error(l, String::from("unknown keyword: `{}`"), vec![l.string()])),
    }?;

    let id = ctx.mol.new_decl(d);
    crate::front::attr_p::AttrParser::attach_attributes(ctx, id, attrs);
    Ok(id)
  }


  pub fn read_fun<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>,  Message<'a>> {
    let name = ctx.lex.get()?;

    let args = MetaParser::read_fun_args(ctx)?;

    let _c = ctx.lex.get()?;
    
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
      ret
    });
    let kind = ctx.mol.new_type(Type{state: crate::ast::TypeState::Unresolved, vari: ty});


    let this = DeclVari::Fun(FunDecl{
      kind, blok
    });

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_using<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>,  Message<'a>> {
    let name = ctx.lex.get()?;
    MetaParser::expect_equal(ctx, "=")?;
    let ty = TypeParser::read_type(ctx, false)?;
    MetaParser::expect_equal(ctx, ";")?;

    let this = DeclVari::Using(
      ty
    );

    Ok(Decl::new(name, this, vis))
  }

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>,  Message<'a>> {
    let name = ctx.lex.get()?;
    let ty = TypeParser::read_struct(ctx)?;

    let tyid = ctx.mol.new_type(ty);

    let this = DeclVari::Using(
      tyid
    );

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

  pub fn read_mod<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<Decl<'a>,  Message<'a>> {
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
      return Err(Message::error(name, format!("circular dependency or multiple definitions of module: {}", mod_name), vec![]));
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
          Ok(id) => decls.push(id),
          Err(e) => {
            new_ctx.sum.add(e);
            let _ = MetaParser::pmr_global(&mut new_ctx);
          }
        }
      }
    }

    let this = DeclVari::Module(ModuleDecl { decls });
    Ok(Decl::new(name, this, vis))
  }

}
