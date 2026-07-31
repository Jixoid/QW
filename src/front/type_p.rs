use crate::{ast::*, control::identy::IdentyId, diagnostic::Message, front::{ParserContext, meta_p::MetaParser}, lexer::Word};


pub struct TypeParser {}

impl TypeParser {

  pub fn read_type<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, _indecl: bool) -> Result<IdentyId, Message<'a>> {
    let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
    let n = ctx.lex.get()?;

    let tyid = match n.str() {
      "fun"    => Self::read_fun(ctx)?,
      "struct" => Self::read_struct(ctx)?,
      "iface"  => Self::read_iface(ctx)?,
      "enum"   => Self::read_enum(ctx)?,
      "flags"  => Self::read_flags(ctx)?,
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
        
        let sub = Self::read_type(ctx, _indecl)?;
        let ty = Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::ReferenceOf{sub, acc}};

        ctx.mol.new_type(ty)
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

        let sub = Self::read_type(ctx, _indecl)?;
        let ty = Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::PointerOf{sub, acc}};

        ctx.mol.new_type(ty)
      }
      _ => {
        let mut path = vec![NickType::new(ctx.mol, n)];
        loop {
          let next = ctx.lex.get()?;
          if next.str() == "::" {
            let nxt_word = ctx.lex.get()?;
            path.push(NickType::new(ctx.mol, nxt_word));
          } else {
            ctx.lex.store(next);
            break;
          }
        }
        
        let ty = if path.len() == 1 {
          Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::Nick(path.pop().unwrap())}
        } else {
          Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::UnresolvedPath(path)}
        };

        ctx.mol.new_type(ty)
      }
    };

    crate::front::attr_p::AttrParser::attach_attributes(ctx, tyid, attrs);

    Ok(tyid)
  }


  pub fn read_fun<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    let args = MetaParser::read_fun_args(ctx)?;

    let _c = ctx.lex.get()?;

    let ret = if _c.str() == "->" {
      Self::read_type(ctx, true)?
    } else {
      ctx.mol.localize(&ctx.mi.sys.mol, ctx.mi.sys.ty_void)
    };


    let this = TypeVari::Function(FunType{args, is_static: false, is_const: false, ret});
    let ty = Type{state: crate::ast::TypeState::Unresolved, vari: this};

    Ok(ctx.mol.new_type(ty))
  }

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    MetaParser::expect_equal(ctx, "{")?;

    let base = Vec::new();
    let mut vars: Vec<FieldType<'a>> = Vec::new();
    let mut funs: Vec<IdentyId> = Vec::new();

    let mut defvis = Visibility::Public;

    'ml: loop {
      let t = ctx.lex.get()?;
      
      if t.str() == "}" { break 'ml; } else { ctx.lex.store(t); }

      let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
      let vis = MetaParser::read_visibility(ctx, &mut defvis)?;
      
      let kw = ctx.lex.get()?;
      if kw.str() == "fun" {
        let fun_decl = crate::front::decl_p::DeclParser::read_fun(ctx, vis, true)?;
        if let crate::ast::DeclVari::Fun(ref f) = fun_decl.vari {
          if f.blok == crate::control::identy::IdentyId::null() {
            return Err(Message::error(kw, "struct methods must have a body (cannot end with `;`)".to_string(), vec![]));
          }
        }
        let fun_id = ctx.mol.new_decl(fun_decl);
        crate::front::attr_p::AttrParser::attach_attributes(ctx, fun_id, attrs);
        funs.push(fun_id);
        continue 'ml;
      } else {
        ctx.lex.store(kw);
      }

      let mut names: Vec<Word> = vec![];

      're: loop {
        let name = ctx.lex.get()?;
        names.push(name);

        let c = ctx.lex.get()?;

        if c.str() == "," { continue 're; }
        else if c.str() == ":" { break 're; }
        else {
          MetaParser::expect_equal2_w(c, ":", ",")?;
        }
      }

      let kind = TypeParser::read_type(ctx, true)?;

      for x in names {
        vars.push(FieldType{name: x, kind, vis, attrs: attrs.clone()});
      }
      
      let e = ctx.lex.get()?;

      if e.str() == ";" { continue 'ml; }
      else if e.str() == "}" { break 'ml; }
      else {
        MetaParser::expect_equal2_w(e, ";", "}")?;
      }
    }

    let this = TypeVari::Struct(StructType{base, vars, funs});
    let ty = Type{state: crate::ast::TypeState::Unresolved, vari: this};

    let tyid = ctx.mol.new_type(ty);


    if let TypeVari::Struct(s) = &ctx.mol.get_type(tyid).vari {
      let funs = s.funs.clone();
      for fun_id in funs {
        let decl_word = ctx.mol.get_decl(fun_id).name.pos().cloned();
        let f_kind = {
          let decl = ctx.mol.get_decl(fun_id);
          if let DeclVari::Fun(f) = &decl.vari {
            Some(f.kind)
          } else { None }
        };

        if let Some(f_kind) = f_kind {
          let (is_static, is_const) = {
            let fun_ty = ctx.mol.get_type(f_kind);
            if let TypeVari::Function(ft) = &fun_ty.vari {
              (ft.is_static, ft.is_const)
            } else { (true, false) }
          };
          
          if !is_static {
            let acc = if is_const { AccessKind::IMM } else { AccessKind::MUT };
            let ref_ty = ctx.mol.new_type(Type{
              state: crate::ast::TypeState::Unresolved,
              vari: TypeVari::ReferenceOf{sub: tyid, acc}
            });
            let self_word = if let Some(w) = decl_word {
              crate::lexer::Word::new(w.mol, w.off, 4, crate::lexer::WordKind::Word)
            } else {
              crate::lexer::Word::new(ctx.lex.mol, 0, 4, crate::lexer::WordKind::Word)
            };
            
            let fun_ty = ctx.mol.get_mut_type(f_kind);
            if let TypeVari::Function(ft) = &mut fun_ty.vari {
              ft.args.insert(0, FieldType{
                name: self_word,
                kind: ref_ty,
                vis: Visibility::Private,
                attrs: vec![],
              });
            }
          }
        }
      }
    }

    Ok(tyid)
  }

  pub fn read_iface<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    MetaParser::expect_equal(ctx, "{")?;

    let mut funs: Vec<FieldType<'a>> = Vec::new();

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.str() == "}" { break 'ml; } else { ctx.lex.store(t); }
      
      let fn_kw = ctx.lex.get()?;
      if fn_kw.str() != "fun" {
        return Err(Message::error(fn_kw, String::from("expected `fun` keyword in iface, found `{}`"), vec![fn_kw.string()]));
      }

      let name = ctx.lex.get()?;
      let args = MetaParser::read_fun_args(ctx)?;
      
      let _c = ctx.lex.get()?;
      let ret = if _c.str() == "->" {
        TypeParser::read_type(ctx, true)?
      } else {
        ctx.lex.store(_c);
        ctx.mol.localize(&ctx.mi.sys.mol, ctx.mi.sys.ty_void)
      };

      MetaParser::expect_equal(ctx, ";")?;

      let ty = TypeVari::Function(FunType{ args, is_static: false, is_const: false, ret });
      let kind = ctx.mol.new_type(Type{state: crate::ast::TypeState::Unresolved, vari: ty});

      funs.push(FieldType{name, kind, vis: Visibility::Public, attrs: vec![]});
    }

    let this = TypeVari::Iface(IfaceType{ funs });
    let ty = Type{state: crate::ast::TypeState::Unresolved, vari: this};

    Ok(ctx.mol.new_type(ty))
  }

  pub fn read_enum<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    MetaParser::expect_equal(ctx, "{")?;

    let mut vals: Vec<FieldCons> = Vec::new();
    let mut current_val: i64 = 0 ;

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.str() == "}" { break 'ml; }
      
      if t.kind != crate::lexer::WordKind::Word {
        return Err(Message::error(t, String::from("expected identifier in enum, found `{}`"), vec![t.string()]));
      }
      
      let nxt = ctx.lex.get()?;
      if nxt.str() == "=" {
        let val_word = ctx.lex.get()?;
        if val_word.kind != crate::lexer::WordKind::Number {
          return Err(Message::error(val_word, String::from("expected number for enum value, found `{}`"), vec![val_word.string()]));
        }
        current_val = val_word.string().parse::<i64>().unwrap_or(0);
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val += 1;
        
        let comma_or_brace = ctx.lex.get()?;
        if comma_or_brace.str() == "}" {
          break 'ml;
        } else if comma_or_brace.str() != "," {
          return Err(Message::error(comma_or_brace, String::from("expected `,` or `}` after enum value, found `{}`"), vec![comma_or_brace.string()]));
        }
      } else if nxt.str() == "," {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val += 1;
      } else if nxt.str() == "}" {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        break 'ml;
      } else {
        return Err(Message::error(nxt, String::from("expected `=`, `,` or `}` after enum identifier, found `{}`"), vec![nxt.string()]));
      }
    }

    let this = TypeVari::Enum(EnumType{ vals });
    let ty = Type{ state: crate::ast::TypeState::Unresolved, vari: this };

    Ok(ctx.mol.new_type(ty))
  }

  pub fn read_flags<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    MetaParser::expect_equal(ctx, "{")?;

    let mut vals: Vec<FieldCons> = Vec::new();
    let mut current_val: i64 = 1;

    'ml: loop {
      let t = ctx.lex.get()?;
      if t.str() == "}" { break 'ml; }
      
      if t.kind != crate::lexer::WordKind::Word {
        return Err(Message::error(t, String::from("expected identifier in enum, found `{}`"), vec![t.string()]));
      }
      
      let nxt = ctx.lex.get()?;
      if nxt.str() == "=" {
        let val_word = ctx.lex.get()?;
        if val_word.kind != crate::lexer::WordKind::Number {
          return Err(Message::error(val_word, String::from("expected number for enum value, found `{}`"), vec![val_word.string()]));
        }
        current_val = val_word.string().parse::<i64>().unwrap_or(0);
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val = if current_val == 0 { 1 } else { current_val << 1 };
        
        let comma_or_brace = ctx.lex.get()?;
        if comma_or_brace.str() == "}" {
          break 'ml;
        } else if comma_or_brace.str() != "," {
          return Err(Message::error(comma_or_brace, String::from("expected `,` or `}` after enum value, found `{}`"), vec![comma_or_brace.string()]));
        }
      } else if nxt.str() == "," {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        current_val = if current_val == 0 { 1 } else { current_val << 1 };
      } else if nxt.str() == "}" {
        vals.push(FieldCons { val: IntegerValue::SIG(current_val), name: t });
        break 'ml;
      } else {
        return Err(Message::error(nxt, String::from("expected `=`, `,` or `}` after enum identifier, found `{}`"), vec![nxt.string()]));
      }
    }

    let this = TypeVari::Flags(FlagsType{ vals });
    let ty = Type{ state: crate::ast::TypeState::Unresolved, vari: this };

    Ok(ctx.mol.new_type(ty))
  }

}
