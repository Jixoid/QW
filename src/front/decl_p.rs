use crate::{ast::{AccessKind, AstId, Decl, DeclId, DeclVari, FunDecl, Type, TypeId, VarDecl, Visibility}, diagnostic::Message, front::{ParserContext, expr_p::ExprParser, meta_p::MetaParser, type_p::TypeParser}, lexer::WK};


pub struct DeclParser {}

impl DeclParser {

  pub fn read_fun<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility, in_struct: bool) -> Result<DeclId,  Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;

    let args = MetaParser::read_fun_args(ctx)?;

    let mut is_static = false;
    let mut is_const = false;
    let mut _c = ctx.lex.get()?;
    
    // Parse function modifiers
    while _c.kind != WK::ArrowRigh && _c.kind != WK::CurlyBracketBeg && _c.kind != WK::Backtick && _c.kind != WK::Semicolon {
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
    let _ = is_const;
    
    let ret = if _c.kind == WK::ArrowRigh {
      Some(TypeParser::read_type(ctx, true)?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let _c = ctx.lex.get()?;

    let blok = if _c.kind == WK::Semicolon {
      AstId::null()
    }
    else if _c.kind == WK::CurlyBracketBeg || _c.kind == WK::Backtick {
      ctx.lex.store(_c);
      ExprParser::read_block(ctx)?
    }
    else {
      _c.expect_equal_str2("{", ";")?;
      AstId::null()
    };


    let this = Type::Fun{args, ret};

    let kind = ctx.cre.new_type(this);


    let this = DeclVari::Fun(FunDecl{
      kind, blok
    });

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_init<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<DeclId,  Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;

    let args = MetaParser::read_fun_args(ctx)?;


    let mut ils = vec![];

    let _c = ctx.lex.get()?;
    if _c.kind == WK::Colon {

      loop {
        let t = ctx.lex.get()?;
        
        if t.kind != WK::Word { ctx.lex.store(t); break; }

        ctx.lex.get()?.expect_equal_str("(")?;
        
        let v = ExprParser::read_expr(ctx, 0)?;
        
        ctx.lex.get()?.expect_equal_str(")")?;
        

        let it = Type::InitParam{ name: t, expr: v };

        ils.push( ctx.cre.new_type(it).to_any() );

        let _c = ctx.lex.get()?;
        if _c.kind == WK::Comma { continue; }
        else {
          ctx.lex.store(_c);
          break;
        }
      }
      
    } else {
      ctx.lex.store(_c);
    }


    let _c = ctx.lex.get()?;
    let blok = if _c.kind == WK::Semicolon {
      AstId::null()
    }
    else if _c.kind == WK::CurlyBracketBeg || _c.kind == WK::Backtick {
      ctx.lex.store(_c);
      ExprParser::read_block(ctx)?
    }
    else {
      _c.expect_equal_str2("{", ";")?;
      AstId::null()
    };


    let rng = if ils.is_empty() { None } else { Some(ctx.cre.new_extra(ils)) };

    let this = Type::Init{args, ils: rng};

    let kind = ctx.cre.new_type(this);


    let this = DeclVari::Fun(FunDecl{
      kind, blok
    });

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_using<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<DeclId, Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;
    ctx.lex.get()?.expect_equal_str("=")?;
    let ty = TypeParser::read_type(ctx, false)?;
    ctx.lex.get()?.expect_equal_str(";")?;

    let this = DeclVari::Using(
      ty
    );

    let dl = Decl::new(name, this, vis); 

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<DeclId, Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;
    let tyid = TypeParser::read_struct(ctx)?;

    let this = DeclVari::Using(tyid);

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_iface<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<DeclId, Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;
    let tyid = TypeParser::read_iface(ctx)?;

    let this = DeclVari::Using(tyid);

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_trait<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<DeclId, Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;
    let tyid = TypeParser::read_trait(ctx)?;

    let this = DeclVari::Using(tyid);

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_enum<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<DeclId, Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;
    let tyid = TypeParser::read_enum(ctx)?;

    let this = DeclVari::Using(tyid);

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_flags<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility) -> Result<DeclId, Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;
    let tyid = TypeParser::read_flags(ctx)?;

    let this = DeclVari::Using(tyid);

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_var<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, vis: Visibility, acck: AccessKind) -> Result<DeclId,  Message<'a>> {
    let name = ctx.lex.get()?.expect_word()?;

    let t1 = ctx.lex.get()?;
    
    let ty = if t1.kind == WK::Colon {
      TypeParser::read_type(ctx, true)?
    } else {
      ctx.lex.store(t1);
      TypeId::null()
    };

    let mut init = None;
    let t2 = ctx.lex.get()?;

    if t2.kind == WK::Assign {
      init = Some(ExprParser::read_expr(ctx, 0)?);
      ctx.lex.get()?.expect_equal_str(";")?;
    }
    else if t2.kind == WK::Semicolon { /* no init */ }
    else {
      t2.expect_equal_str2("=", ";")?;
    }

    let this = DeclVari::Var(VarDecl{
      kind: ty,
      init,
      acck,
      comptime: false,
    });

    let dl = Decl::new(name, this, vis);

    Ok(ctx.cre.new_decl(dl))
  }

}
