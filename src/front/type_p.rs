use crate::{ast::*, control::identy::IdentyId, diagnostic::Message, front::{ParserContext, meta_p::MetaParser}, lexer::Word};


pub struct TypeParser {}

impl TypeParser {

  pub fn read_type<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, _indecl: bool) -> Result<IdentyId, Message<'a>> {
    let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
    let n = ctx.lex.get()?;

    let ty = match n.str() {
      "fun"    => Self::read_fun(ctx)?,
      "struct" => Self::read_struct(ctx)?,
      "iface"  => Self::read_iface(ctx)?,
      "&" => {
        let nxt = ctx.lex.get()?;
        let acc = if nxt.str() == "mut" {
          AccessKind::MUT
        } else if nxt.str() == "imm" {
          AccessKind::IMM
        } else {
          ctx.lex.store(nxt);
          AccessKind::IMM
        };
        let sub = Self::read_type(ctx, _indecl)?;
        Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::ReferenceOf{sub, acc}}
      }
      "^" => {
        let nxt = ctx.lex.get()?;
        let acc = if nxt.str() == "mut" {
          AccessKind::MUT
        } else if nxt.str() == "imm" {
          AccessKind::IMM
        } else {
          ctx.lex.store(nxt);
          AccessKind::IMM
        };
        let sub = Self::read_type(ctx, _indecl)?;
        Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::PointerOf{sub, acc}}
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
        if path.len() == 1 {
          Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::Nick(path.pop().unwrap())}
        } else {
          Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::UnresolvedPath(path)}
        }
      }
    };

    let tyid = ctx.mol.new_type(ty);
    crate::front::attr_p::AttrParser::attach_attributes(ctx, tyid, attrs);

    Ok(tyid)
  }


  pub fn read_fun<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Type<'a>, Message<'a>> {
    let args = MetaParser::read_fun_args(ctx)?;

    let _c = ctx.lex.get()?;

    let ret = if _c.str() == "->" {
      Self::read_type(ctx, true)?
    } else {
      ctx.mol.localize(&ctx.mi.sys.mol, ctx.mi.sys.ty_void)
    };


    let this = TypeVari::Function(FunType{
      args, is_static: false, is_const: false, ret
    });

    Ok(Type{state: crate::ast::TypeState::Unresolved, vari: this})
  }

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Type<'a>, Message<'a>> {
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

    let this = TypeVari::Struct(StructType{
      base, vars, funs
    });

    Ok(Type{state: crate::ast::TypeState::Unresolved, vari: this})
  }

  pub fn read_iface<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Type<'a>, Message<'a>> {
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

    let this = TypeVari::Iface(IfaceType { funs });
    Ok(Type{state: crate::ast::TypeState::Unresolved, vari: this})
  }

}
