use crate::{ast::{AccessKind, Decl, DeclId, DeclVari, Thing, Type, TypeId, Visibility}, diagnostic::Message, front::{ParserContext, expr_p::ExprParser, meta_p::MetaParser, type_p::TypeParser}, lexer::WK};


pub struct DeclParser {}

impl DeclParser {

  pub fn read_fun(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId,  Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();

    let kind = TypeParser::read_fun(ctx)?;

    let _c = ctx.lex.get()?;

    let blok = if _c.kind == WK::Semicolon {
      None
    }
    else if _c.kind == WK::CurlyBracketBeg || _c.kind == WK::Backtick {
      ctx.lex.store(_c);
      Some(ExprParser::read_block(ctx)?)
    }
    else {
      _c.expect_kind2(WK::CurlyBracketBeg, WK::Semicolon)?;
      None
    };

    
    let dl = Decl{name, vis, vari: DeclVari::Fun{kind, blok}};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_init(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId,  Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();

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
      None
    }
    else if _c.kind == WK::CurlyBracketBeg || _c.kind == WK::Backtick {
      ctx.lex.store(_c);
      Some(ExprParser::read_block(ctx)?)
    }
    else {
      _c.expect_kind2(WK::CurlyBracketBeg, WK::Semicolon)?;
      None
    };


    let rng = if ils.is_empty() { None } else { Some(ctx.cre.new_extra(ils)) };

    let kind = ctx.cre.new_type(Type::Init{args, attr: 0});

    let this = Decl{name, vis, vari: DeclVari::Init{kind, blok, ils: rng}};

    Ok(ctx.cre.new_decl(this))
  }
  
  pub fn read_fini(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId,  Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();

    let args = MetaParser::read_fun_args(ctx)?;

    let _c = ctx.lex.get()?;
    let blok = if _c.kind == WK::Semicolon {
      None
    }
    else if _c.kind == WK::CurlyBracketBeg || _c.kind == WK::Backtick {
      ctx.lex.store(_c);
      Some(ExprParser::read_block(ctx)?)
    }
    else {
      _c.expect_kind2(WK::CurlyBracketBeg, WK::Semicolon)?;
      None
    };


    let kind = ctx.cre.new_type(Type::Fini{args, attr: 0});

    let this = Decl{name, vis, vari: DeclVari::Fini{kind, blok}};

    Ok(ctx.cre.new_decl(this))
  }
  
  pub fn read_using(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();
    
    ctx.lex.get()?.expect_kind(WK::Assign)?;
    let kind = TypeParser::read_type(ctx, false)?;
    ctx.lex.get()?.expect_kind(WK::Semicolon)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis}; 

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_struct(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();
    let kind = TypeParser::read_struct(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_iface(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();
    let kind = TypeParser::read_iface(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_trait(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();
    let kind = TypeParser::read_trait(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_enum(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();
    let kind = TypeParser::read_enum(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_flags(ctx: &mut ParserContext, vis: Visibility) -> Result<DeclId, Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();
    let kind = TypeParser::read_flags(ctx)?;

    let this = DeclVari::Using{ kind };

    let dl = Decl{name, vari: this, vis};

    Ok(ctx.cre.new_decl(dl))
  }

  pub fn read_var(ctx: &mut ParserContext, vis: Visibility, acck: AccessKind) -> Result<DeclId,  Message> {
    let name = ctx.lex.get()?.expect_word(ctx.far)?.save();

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
