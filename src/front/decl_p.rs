use crate::{ast::{AccessKind, AstId, Decl, DeclId, DeclVari, Thing, Type, TypeId, Visibility}, diagnostic::Message, front::{ParserContext, expr_p::ExprParser, meta_p::MetaParser, type_p::TypeParser}, lexer::WK};


pub struct DeclParser {}

impl DeclParser {

  pub fn read_fun(ctx: &mut ParserContext, vis: Visibility, in_struct: bool) -> Result<DeclId,  Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;

    let args = MetaParser::read_fun_args(ctx)?;

    let mut is_static = false;
    let mut is_const = false;
    let mut _c = ctx.lex.get()?;
    
    // Parse function modifiers
    while _c.kind != WK::ArrowRigh && _c.kind != WK::CurlyBracketBeg && _c.kind != WK::Backtick && _c.kind != WK::Semicolon {
      if _c.str(ctx.far) == "static" {
        if !in_struct {
          return Err(Message::error(_c, String::from("`static` modifier is only allowed inside structs"), vec![]));
        }
        is_static = true;
      } else if _c.str(ctx.far) == "const" {
        if !in_struct {
          return Err(Message::error(_c, String::from("`const` modifier is only allowed inside structs"), vec![]));
        }
        if is_static {
          return Err(Message::error(_c, String::from("a function cannot be both `static` and `const`"), vec![]));
        }
        is_const = true;
      } else {
        return Err(Message::error(_c, String::from("unknown function modifier: `{}`"), vec![_c.string(ctx.far)]));
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
      _c.expect_kind2(WK::CurlyBracketBeg, WK::Semicolon)?;
      AstId::null()
    };


    let this = Type::Fun{args, ret};

    let kind = ctx.cre.new_type(this);


    let this = DeclVari::Fun{kind, blok};

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_init(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId,  Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;

    let args = MetaParser::read_fun_args(ctx)?;


    let mut ils = vec![];

    let _c = ctx.lex.get()?;
    if _c.kind == WK::Colon {

      loop {
        let t = ctx.lex.get()?;
        
        if t.kind != WK::Word { ctx.lex.store(t); break; }

        ctx.lex.get()?.expect_kind(WK::ParenBeg)?;
        
        let v = ExprParser::read_expr(ctx, 0)?;
        
        ctx.lex.get()?.expect_kind(WK::ParenEnd)?;
        

        let it = Thing::NamedExpr(t.save(), v);

        ils.push( ctx.cre.new_thing(it).to_any() );

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
      _c.expect_kind2(WK::CurlyBracketBeg, WK::Semicolon)?;
      AstId::null()
    };


    let rng = if ils.is_empty() { None } else { Some(ctx.cre.new_extra(ils)) };

    let this = Type::Init{args, ils: rng};

    let kind = ctx.cre.new_type(this);


    let this = DeclVari::Fun{kind, blok};

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_using(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;
    
    ctx.lex.get()?.expect_kind(WK::Assign)?;
    let kind = TypeParser::read_type(ctx, false)?;
    ctx.lex.get()?.expect_kind(WK::Semicolon)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis}; 

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_struct(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;
    let kind = TypeParser::read_struct(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_iface(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;
    let kind = TypeParser::read_iface(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_trait(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;
    let kind = TypeParser::read_trait(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_enum(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;
    let kind = TypeParser::read_enum(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_flags(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;
    let kind = TypeParser::read_flags(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_var(ctx: &mut ParserContext, vis: Visibility, acck: AccessKind) -> Result<DeclId,  Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?;

    let t1 = ctx.lex.get()?;
    
    let kind = if t1.kind == WK::Colon {
      TypeParser::read_type(ctx, true)?
    } else {
      ctx.lex.store(t1);
      TypeId::null()
    };

    let mut init = None;
    let t2 = ctx.lex.get()?;

    if t2.kind == WK::Assign {
      init = Some(ExprParser::read_expr(ctx, 0)?);
      ctx.lex.get()?.expect_kind(WK::Semicolon)?;
    }
    else if t2.kind == WK::Semicolon { /* no init */ }
    else {
      t2.expect_kind2(WK::Assign, WK::Semicolon)?;
    }

    let this = DeclVari::Var{kind, acck, init};

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

}
