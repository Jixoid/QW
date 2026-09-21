use qwc_ast::{IdentSave, Item, ItemId, ItemKind, Rng, Thing, ThingId, Visibility};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{parse::Ctx, ctx, ExprParser, MetaParser, TypeParser, WordCheck};


pub struct ItemParser;

impl ItemParser {

  // Public
  pub fn read_item(ctx: &mut Ctx, defvis: &mut Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let (vis, attr) = MetaParser::read_start(ctx!(cre, sin, far, lex, sum), defvis)?;

    let id = match lex.peek_k()? {
      (WK::Let |
       WK::Var, _)     => ItemParser::sub_let     (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Using, _)   => ItemParser::sub_using   (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Fun, _)     => ItemParser::sub_fun     (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Init, _)    => ItemParser::sub_init    (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Fini, _)    => ItemParser::sub_fini    (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Impl, _)    => ItemParser::sub_impl    (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Generic, _) => ItemParser::sub_generic (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Use, _)     => ItemParser::sub_use     (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Mod, _)     => ItemParser::sub_mod     (ctx!(cre, sin, far, lex, sum), vis)?,
      
      (WK::Struct | WK::Iface | WK::Trait | WK::Enum | WK::Flags | WK::Variant, _) => ItemParser::sub_itemty(ctx!(cre, sin, far, lex, sum), vis)?,

      (_, c) => return Err(Message::error(UNKNOWN_KEYWORD, Label::new_pos(c))),
    };

    if let Some(a) = attr { cre.attach(id, a); }
    
    Ok(id)
  }

  pub fn read_item_in(ctx: &mut Ctx, defvis: &mut Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let (vis, attr) = MetaParser::read_start(ctx!(cre, sin, far, lex, sum), defvis)?;

    let id = match lex.peek_k()? {
      (WK::Let |
       WK::Var, _)     => ItemParser::sub_let     (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Using, _)   => ItemParser::sub_using   (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Fun, _)     => ItemParser::sub_fun     (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Init, _)    => ItemParser::sub_init    (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Fini, _)    => ItemParser::sub_fini    (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Impl, _)    => ItemParser::sub_impl_in (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Generic, _) => ItemParser::sub_generic (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Use, _)     => ItemParser::sub_use     (ctx!(cre, sin, far, lex, sum), vis)?,
      (WK::Mod, _)     => ItemParser::sub_mod     (ctx!(cre, sin, far, lex, sum), vis)?,
      
      (WK::Struct | WK::Iface | WK::Trait | WK::Enum | WK::Flags | WK::Variant, _) => ItemParser::sub_itemty(ctx!(cre, sin, far, lex, sum), vis)?,

      (WK::Word, _) => ItemParser::sub_let_in(ctx!(cre, sin, far, lex, sum), vis)?,

      (_, c) => return Err(Message::error(UNKNOWN_KEYWORD, Label::new_pos(c))),
    };

    if let Some(a) = attr { cre.attach(id, a); }
    
    Ok(id)
  }


  // Sub
  fn sub_let(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;
      Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };

    lex.get()?.expect_kind(WK::Eq)?;

    let value = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::Semicolon)?;
    

    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Let{kind, value, ism: start.kind() == WK::Var}
    };
    
    Ok(cre.push(this))
  }

  fn sub_let_in(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    let name = lex.get()?.ident(sin, far)?;

    lex.get()?.expect_kind(WK::Colon)?;

    let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::Semicolon)?;


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Member {kind}
    };
    
    Ok(cre.push(this))
  }

  fn sub_using(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;
    
    lex.get()?.expect_kind(WK::Eq)?;
    let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;
    lex.get()?.expect_kind(WK::Semicolon)?;

    
    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Using(kind),
    }; 

    Ok(cre.push(this))
  }

  fn sub_fun(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum))?;

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Fun{ kind, blok },
    };
    
    Ok(cre.push(this))
  }

  fn sub_init(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum))?;

    let ils = if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;
      let mut ils = vec![];
      
      loop {
        let name = if let (WK::Word, c) = lex.peek_k()? { lex.bump()?; c.ident(sin, far)? } else { break };
        
        lex.get()?.expect_kind(WK::ParenL)?;
        
        let v = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;
        
        lex.get()?.expect_kind(WK::ParenR)?;
        
        let it = Thing::NamedExpr(name, v);
        ils.push(cre.push(it));
        
        if lex.peek()?.kind() == WK::Comma { lex.bump()? } else { break }
      }
      
      cre.extra(&ils)
    } else {
      Rng::empty()
    };

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Init{ kind, blok, ils }
    };

    Ok(cre.push(this))
  }
  
  fn sub_fini(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum))?;

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Fini{ kind, blok }
    };

    Ok(cre.push(this))
  }

  fn sub_impl(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let type_ty = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;
    
    let trait_ty = if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;
      Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };

    lex.get()?.expect_kind(WK::BraceL)?;

    let ctn = {
      let mut impls = vec![];
      let defvis = &mut Visibility::Inherited;
      
      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        let (vis, attrs) = MetaParser::read_start(ctx!(cre, sin, far, lex, sum), defvis)?;
        
        let next_kw = lex.peek_k()?;
        if next_kw.0 == WK::Fun {
          let fun = ItemParser::sub_fun(ctx!(cre, sin, far, lex, sum), vis)?;
          
          if let Some(a) = attrs { cre.attach(fun, a); }
          
          impls.push(fun);
        } else {
          let c = lex.get()?;
          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(c)));
        }
      }
      
      cre.extra(&impls)
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: ItemKind::Impl{ type_ty, trait_ty, ctn },
    };
    
    Ok(cre.push(this))
  }
  
  fn sub_impl_in(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    lex.get()?.expect_kind(WK::Colon)?;

    let trait_ty = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::BraceL)?;

    let ctn = {
      let mut impls = vec![];
      let defvis = &mut Visibility::Inherited;
      
      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        let (vis, attrs) = MetaParser::read_start(ctx!(cre, sin, far, lex, sum), defvis)?;
        
        let next_kw = lex.peek_k()?;
        if next_kw.0 == WK::Fun {
          let fun = ItemParser::sub_fun(ctx!(cre, sin, far, lex, sum), vis)?;
          
          if let Some(a) = attrs { cre.attach(fun, a); }
          
          impls.push(fun);
        } else {
          let c = lex.get()?;
          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(c)));
        }
      }
      
      cre.extra(&impls)
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: ItemKind::ImplIn{ trait_ty, ctn },
    };
    
    Ok(cre.push(this))
  }
  
  fn sub_generic(ctx: &mut Ctx, mut vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    lex.get()?.expect_kind(WK::Lt)?;
    
    let params = {
      let mut args = vec![];

      loop {
        if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break; }
        
        let mut names = vec![];
        let hty = loop {
          let name = match lex.get_k()? {
            (WK::Word, c) => c.ident(sin, far)?,
            (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c)))
          };
          names.push(name);

          match lex.peek_k()? {
            (WK::Comma, _) => {lex.bump()?; continue},
            (WK::Colon, _) => {lex.bump()?; break true},
            (WK::Semicolon, _) => {lex.bump()?; break false},
            (WK::Gt, _) => break false,

            (_, c) => c.panic_kind4(WK::Colon, WK::Comma, WK::Gt, WK::Semicolon)?
          }
        };

        let kind = if hty { Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?) } else { None };
        
        for x in names {
          let thing = match kind {
            Some(kind) => Thing::NamedType(x, kind),
            None => Thing::Name(x),
          };

          args.push(cre.push(thing));
        }

        match lex.get_k()? {
          (WK::Gt, _) => break,
          (WK::Comma, _) => continue,

          (_, c) => c.panic_kind2(WK::Comma, WK::Gt)?
        }
      }

      cre.extra(&args)
    };

    let reqs = if { let p = lex.peek()?; p.kind() == WK::Word && lex.str(p) == "requires" } {
      lex.bump()?;
      let mut reqs = vec![];
      
      loop {
        let name = if let (WK::Word, c) = lex.peek_k()? { lex.bump()?; c.ident(sin, far)? } else { break };
        
        lex.get()?.expect_kind(WK::Colon)?;
        
        let mut tys = vec![];
        
        loop {
          tys.push(TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?);
          
          match lex.get_k()? {
            (WK::Pipe, _) => continue,
            (WK::Semicolon, _) => break,

            (_, c) => c.panic_kind2(WK::Pipe, WK::Semicolon)?
          }
        }
        
        let rng = cre.extra(&tys);
        
        let this = Thing::NamedTypeList(name, rng);
        
        reqs.push(cre.push(this));
      }
      
      cre.extra(&reqs)
    } else {
      Rng::empty()
    };
    
    let ctn = if lex.peek()?.kind() == WK::BraceL {
      lex.bump()?;

      let mut ctn = vec![];
      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        let id = Self::read_item(ctx!(cre, sin, far, lex, sum), &mut vis)?;

        ctn.push(id);
      }

      cre.extra(&ctn)
    } else {
      let id = Self::read_item(ctx!(cre, sin, far, lex, sum), &mut vis)?;

      cre.extra(&[id])
    };
    

    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: ItemKind::Generic{ params, reqs, ctn },
    };

    Ok(cre.push(this))
  }

  fn sub_use(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let begin = match lex.get_k()? {
      (WK::Crate, _) => cre.push(Thing::Crate),
      (WK::Super, _) => cre.push(Thing::Super),
      (WK::Word, c)  => { let a = c.ident(sin, far)?; cre.push(Thing::Name(a)) },

      (_, c) => return Err(Message::error(UNKNOWN_USE_STARTER, Label::new_pos(c))),
    };

    let rng = {      
      let mut all = vec![ begin ];
      
      loop {
        match lex.get_k()? {
          (WK::Semicolon, _) => break,
          (WK::Colon2, _) => all.push(Self::read_use_sub(ctx!(cre, sin, far, lex, sum))?),
        
          (_, c) => c.panic_kind2(WK::Colon2, WK::Semicolon)?,
        }
      }

      cre.extra(&all)
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: ItemKind::Import( rng )
    };

    Ok(cre.push(this))
  }

  fn read_use_sub(ctx: &mut Ctx) -> Result<ThingId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let ret = match lex.get_k()? {
      (WK::Crate, _) => cre.push(Thing::Crate),
      (WK::Super, _) => cre.push(Thing::Super),
      (WK::Mul, _)   => cre.push(Thing::Wildcard),

      (WK::Word, c)  => { let a = c.ident(sin, far)?; cre.push(Thing::Name(a)) },

      (_, c) => return Err(Message::error(UNKNOWN_USE_SEGMENT, Label::new_pos(c))),
    };

    Ok(ret)
  }

  fn sub_mod(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let ctn = match lex.get_k()? {
      (WK::Semicolon, _) => None,

      (WK::BraceL, _) => {
        let mut ctn = vec![];
        let defvis = &mut Visibility::Inherited;

        loop {
          if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
          
          let id = Self::read_item(ctx!(cre, sin, far, lex, sum), defvis)?;

          ctn.push(id);
        }

        Some(cre.extra(&ctn))
      }

      (_, c) => c.panic_kind2(WK::Semicolon, WK::BraceL)?
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: match ctn {
        None => ItemKind::ModuleUnloaded,
        Some(v) => ItemKind::Module(v),
      }
    };
    
    Ok(cre.push(this))
  }

  fn sub_itemty(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let name = lex.get()?.ident(sin, far)?;
    
    let kind = match start.kind() {
      WK::Struct  => TypeParser::sub_struct(ctx!(cre, sin, far, lex, sum))?,
      WK::Iface   => TypeParser::sub_iface(ctx!(cre, sin, far, lex, sum))?,
      WK::Trait   => TypeParser::sub_trait(ctx!(cre, sin, far, lex, sum))?,
      WK::Enum    => TypeParser::sub_enum(ctx!(cre, sin, far, lex, sum))?,
      WK::Flags   => TypeParser::sub_flags(ctx!(cre, sin, far, lex, sum))?,
      WK::Variant => TypeParser::sub_variant(ctx!(cre, sin, far, lex, sum))?,

      _ => panic!(),
    };
    
    
    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::ItemTy(kind),
    }; 

    Ok(cre.push(this))
  }

}
