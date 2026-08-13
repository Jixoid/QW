use crate::ast::{AstId, Item, ItemVari};
use crate::lexer::WK;
use crate::{ast::{AccessKind, DeclVari, FieldCons, FieldType, IntegerValue, NickType, Type, TypeId, Visibility}, diagnostic::Message, front::{ParserContext, attr_p::AttrParser, decl_p::DeclParser, meta_p::MetaParser}, lexer::Word};


pub struct TypeParser {}

impl TypeParser {

  pub fn read_type<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, indecl: bool) -> Result<TypeId, Message<'a>> {
    let attrs = AttrParser::read_attr(ctx)?;
    let _n = ctx.lex.get()?;

    let tyid = match _n.str() {
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
        let acc = match nxt.str() {
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
        let acc = match nxt.str() {
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
        match _c.str() {
          "]" => { // dyn array
            let ty = Type::Array{ sub, ext: vec![] };
            ctx.cre.new_type(ty)
          }

          "," => { // sized array
            let mut ext = Vec::new();

            let _w = ctx.lex.get()?;

            match _w.str().parse::<u32>() {
              Ok(e) => ext.push(e),
              Err(..) => return Err(Message::error(_w, "cannot convert to integer `{}`".to_string(), vec![_w.string()])),
            }


            loop {
              let _c = ctx.lex.get()?;

              if _c.kind == WK::SquareBracketEnd {
                break;
              }
              else if _c.kind == WK::Comma {
                let _w = ctx.lex.get()?;

                match _w.str().parse::<u32>() {
                  Ok(e) => ext.push(e),
                  Err(..) => return Err(Message::error(_w, "cannot convert to integer `{}`".to_string(), vec![_w.string()])),
                }
              }
              else {
                _c.expect_equal_str2(",", "]")?;
              }
            }

            let ty = Type::Array{ sub, ext };
            ctx.cre.new_type(ty)
          }

          "x" => { // vector
            let _w = ctx.lex.get()?;

            let ext = match _w.str().parse::<u32>() {
              Ok(e) => e,
              Err(..) => return Err(Message::error(_w, "cannot convert to integer `{}`".to_string(), vec![_w.string()])),
            };

            ctx.lex.get()?.expect_equal_str("]")?;

            let ty = Type::Vector{ sub, ext };
            ctx.cre.new_type(ty)
          }

          _ => {
            ctx.lex.store(_c);
            _c.expect_equal_str3("]", ",", "x")?;
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
          vars.push(sub);
          
          let _c = ctx.lex.get()?;
          if _c.kind == WK::ParenEnd { break; }
          else if _c.kind == WK::Comma { is_tuple = true; continue; }
          else {
            _c.expect_equal_str2(",", ")")?;
          }
        }

        if is_tuple || vars.len() != 1 {
          let ty = Type::Tuple{ vars };
          ctx.cre.new_type(ty)
        } else {
          vars[0]
        }
      }

      _ => {
        let nick = Type::Nick(NickType{pos: _n.expect_word()?, idx: ctx.cre.get_str(_n.str())});
        let mut path = vec![ ctx.cre.new_type(nick) ];

        loop {
          let _c = ctx.lex.get()?;
          
          match _c.str() {
            "::" => {
              let _n = ctx.lex.get()?.expect_word()?;
              let sub = Type::Nick(NickType{pos: _n, idx: ctx.cre.get_str(_n.str())});
              
              path.push( ctx.cre.new_type(sub) );
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
                  sep.expect_equal_str2(",", ">")?;
                }
              }

              let base = Type::Path(path);
              let base_id = ctx.cre.new_type(base);

              let ty = Type::Specialize{base: base_id, args};
              path = vec![ ctx.cre.new_type(ty) ];
            }

            _ => {ctx.lex.store(_c); break},
          }
        }


        if path.len() == 1 { path[0] } else { ctx.cre.new_type(Type::Path(path)) }
      }
    };

    AttrParser::attach_attr(ctx, tyid, attrs);

    Ok(tyid)
  }


  pub fn read_fun<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message<'a>> {
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

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message<'a>> {
    ctx.lex.get()?.expect_equal_str("{")?;

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

        if let DeclVari::Fun(ref f) = ctx.cre.get_decl(fun).vari {
          if f.blok.is_null() {
            return Err(Message::error(kw, "struct methods must have a body (cannot end with `;`)".to_string(), vec![]));
          }
        } else {
          panic!("must be decl::fun")
        }
        
        funs.push(fun);
        continue 'ml;
      } else if kw.kind == WK::Init { // object init
        let fun = DeclParser::read_init(ctx, vis)?;

        if let DeclVari::Fun(ref f) = ctx.cre.get_decl(fun).vari {
          if f.blok.is_null() {
            return Err(Message::error(kw, "struct methods must have a body (cannot end with `;`)".to_string(), vec![]));
          }
        } else {
          panic!("must be decl::fun")
        }
        
        funs.push(fun);
        continue 'ml;
      } else if kw.kind == WK::Impl { // trait impl
        ctx.lex.get()?.expect_equal_str(":")?;

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
        let name = ctx.lex.get()?.expect_word()?;
        names.push(name);

        let c = ctx.lex.get()?;

        if c.kind == WK::Comma { continue 're; }
        else if c.kind == WK::Colon { break 're; }
        else {
          c.expect_equal_str2(":", ",")?;
        }
      }

      let kind = TypeParser::read_type(ctx, true)?;
      
      for x in names {
        vars.push(FieldType{name: x, kind, vis, attrs: Vec::new()}); // attrs.clone()});
      }
      
      let e = ctx.lex.get()?;

      if e.kind == WK::Semicolon { continue 'ml; }
      else if e.kind == WK::CurlyBracketEnd { break 'ml; }
      else {
        e.expect_equal_str2(";", "}")?;
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

  pub fn read_iface<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message<'a>> {
    ctx.lex.get()?.expect_equal_str("{")?;

    let mut funs = vec![];

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break 'ml; } else { ctx.lex.store(t); }
      
      let fn_kw = ctx.lex.get()?;
      if fn_kw.kind != WK::Fun {
        return Err(Message::error(fn_kw, String::from("expected `fun` keyword in iface, found `{}`"), vec![fn_kw.string()]));
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

      ctx.lex.get()?.expect_equal_str(";")?;

      let this = Type::Fun{ args, ret };
      let kind = ctx.cre.new_type(this);

      funs.push(FieldType{name, kind, vis: Visibility::Public, attrs: vec![]});
    }

    let this = Type::Iface{ funs };

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_trait<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message<'a>> {
    ctx.lex.get()?.expect_equal_str("{")?;

    let mut funs = vec![];

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break 'ml; } else { ctx.lex.store(t); }
      
      let fn_kw = ctx.lex.get()?;
      if fn_kw.kind != WK::Fun {
        return Err(Message::error(fn_kw, String::from("expected `fun` keyword in iface, found `{}`"), vec![fn_kw.string()]));
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

      ctx.lex.get()?.expect_equal_str(";")?;

      let ty = Type::Fun{ args, ret };
      let kind = ctx.cre.new_type(ty);

      funs.push(FieldType{name, kind, vis: Visibility::Public, attrs: vec![]});
    }

    let this = Type::Trait{ funs };

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_enum<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message<'a>> {
    ctx.lex.get()?.expect_equal_str("{")?;

    let mut vals = Vec::new();
    let mut current_val: i64 = 0 ;

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break 'ml; }
      
      if t.kind != crate::lexer::WordKind::Word {
        return Err(Message::error(t, String::from("expected identifier in enum, found `{}`"), vec![t.string()]));
      }
      
      let nxt = ctx.lex.get()?;
      if nxt.kind == WK::Assign {
        let val_word = ctx.lex.get()?;
        if val_word.kind != crate::lexer::WordKind::Number {
          return Err(Message::error(val_word, String::from("expected number for enum value, found `{}`"), vec![val_word.string()]));
        }
        current_val = val_word.string().parse::<i64>().unwrap_or(0);
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val += 1;
        
        let comma_or_brace = ctx.lex.get()?;
        if comma_or_brace.kind == WK::CurlyBracketEnd {
          break 'ml;
        } else if comma_or_brace.kind != WK::Comma {
          return Err(Message::error(comma_or_brace, String::from("expected `,` or `}` after enum value, found `{}`"), vec![comma_or_brace.string()]));
        }
      } else if nxt.kind == WK::Comma {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val += 1;
      } else if nxt.kind == WK::CurlyBracketEnd {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        break 'ml;
      } else {
        return Err(Message::error(nxt, String::from("expected `=`, `,` or `}` after enum identifier, found `{}`"), vec![nxt.string()]));
      }
    }

    let this = Type::Enum{ vals };

    Ok(ctx.cre.new_type(this))
  }

  pub fn read_flags<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<TypeId, Message<'a>> {
    ctx.lex.get()?.expect_equal_str("{")?;

    let mut vals: Vec<FieldCons> = Vec::new();
    let mut current_val: i64 = 1;

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd { break 'ml; }
      
      if t.kind != crate::lexer::WordKind::Word {
        return Err(Message::error(t, String::from("expected identifier in enum, found `{}`"), vec![t.string()]));
      }
      
      let nxt = ctx.lex.get()?;
      if nxt.kind == WK::Assign {
        let val_word = ctx.lex.get()?;
        if val_word.kind != crate::lexer::WordKind::Number {
          return Err(Message::error(val_word, String::from("expected number for enum value, found `{}`"), vec![val_word.string()]));
        }
        current_val = val_word.string().parse::<i64>().unwrap_or(0);
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val = if current_val == 0 { 1 } else { current_val << 1 };
        
        let comma_or_brace = ctx.lex.get()?;
        if comma_or_brace.kind == WK::CurlyBracketEnd {
          break 'ml;
        } else if comma_or_brace.kind != WK::Comma {
          return Err(Message::error(comma_or_brace, String::from("expected `,` or `}` after enum value, found `{}`"), vec![comma_or_brace.string()]));
        }
      } else if nxt.kind == WK::Comma {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val = if current_val == 0 { 1 } else { current_val << 1 };
      } else if nxt.kind == WK::CurlyBracketEnd {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        break 'ml;
      } else {
        return Err(Message::error(nxt, String::from("expected `=`, `,` or `}` after enum identifier, found `{}`"), vec![nxt.string()]));
      }
    }

    let this = Type::Flags{ vals };

    Ok(ctx.cre.new_type(this))
  }

}
