use crate::{ast::*, control::identy::IdentyId, diagnostic::Message, front::{ParserContext, meta_p::MetaParser}, lexer::Word};


pub struct TypeParser {}

impl TypeParser {

  pub fn read_type<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, _indecl: bool) -> Result<IdentyId, Message<'a>> {
    let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
    let n = ctx.lex.get()?;

    let ty = match n.str() {
      "fun"    => Self::read_fun(ctx)?,
      "struct" => Self::read_struct(ctx)?,
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
      _ => Type{state: crate::ast::TypeState::Unresolved, vari: TypeVari::Nick(NickType::new(ctx.mol, n))},
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
      args, ret
    });

    Ok(Type{state: crate::ast::TypeState::Unresolved, vari: this})
  }

  pub fn read_struct<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Type<'a>, Message<'a>> {
    MetaParser::expect_equal(ctx, "{")?;

    let base = Vec::new();
    let mut vars: Vec<FieldType<'a>> = Vec::new();

    let mut defvis = Visibility::Public;

    'ml: loop {
      let t = ctx.lex.get()?;
      
      if t.str() == "}" { break 'ml; } else { ctx.lex.store(t); }

      let vis = MetaParser::read_visibility(ctx, &mut defvis)?;
      
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
        vars.push(FieldType{name: x, kind, vis});
      }
      
      let e = ctx.lex.get()?;

      if e.str() == ";" { continue 'ml; }
      else if e.str() == "}" { break 'ml; }
      else {
        MetaParser::expect_equal2_w(e, ";", "}")?;
      }
    }

    let this = TypeVari::Struct(StructType{
      base, vars
    });

    Ok(Type{state: crate::ast::TypeState::Unresolved, vari: this})
  }

}
