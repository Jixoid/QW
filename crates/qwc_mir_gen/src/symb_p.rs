use qwc_comptime::Evaluator;
use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mangling::{Mangler, ManglerQW};
use qwc_mir as mir;
use qwc_string_interner::Sid;

use crate::{Ctx, MayFail, TypeLow, BlokLow, ctx};


pub struct SymbLow;

impl SymbLow {

  pub fn low(ctx: &mut Ctx, id: hir::ItemId) -> MayFail<Message> { ctx!(ctx => cre, sum, src, sin, mgr);
    let it: &hir::Item = src.get(id);

    match *it {
      hir::Item::RootNS{rng} => Self::low_root(ctx, rng)?,

      hir::Item::Variable{kind, name, expr, ism} => Self::low_variable(ctx, name, kind, expr, ism)?,

      hir::Item::Function{kind, name, expr} => Self::low_function(ctx, name, kind, expr)?,

      kind @_ => todo!("{kind:#?}")
    }

    Ok(())
  }


  fn low_root(ctx: &mut Ctx, rng: hir::Rng) -> MayFail<Message> { ctx!(ctx => cre, sum, src, sin, mgr);
    let mgr = &mut vec![];
    
    for id in src.extra_get(rng) {
      let id = hir::ItemId::new_from(id);

      Self::low(ctx!(cre,sum,src,sin,mgr), id)?;
    }

    Ok(())
  }

  fn low_variable(ctx: &mut Ctx, name: Sid, kind: hir::TypeId, expr: hir::ExprId, ism: bool) -> MayFail<Message> { ctx!(ctx => cre, sum, src, sin, mgr);
    let sym = ManglerQW::new(sin, mgr, name);
    
    let ety = TypeLow::low(ctx!(cre,sum,src,sin,mgr), kind)?;

    let value = match expr.evaluate(*src) {
      Some(v) => cre.push(convert_value(v)),
      None => panic!("the values of global variables must be constant"),
    };


    // Post
    let this = mir::Symbol {
      name: cre.sym(&sym),
      stat: mir::SymbolStat::Export,
      ety,
      kind: mir::SymbolKind::Variable{ ism, value }
    };

    cre.push(this);

    Ok(())
  }

  fn low_function(ctx: &mut Ctx, name: Sid, kind: hir::TypeId, expr: hir::ExprId) -> MayFail<Message> { ctx!(ctx => cre, sum, src, sin, mgr);
    let sym = ManglerQW::new(sin, mgr, name);
    
    let ety = TypeLow::low(ctx!(cre,sum,src,sin,mgr), kind)?;

    let blok = BlokLow::low(ctx!(cre,sum,src,sin,mgr), expr)?;


    // Post
    let this = mir::Symbol {
      name: cre.sym(&sym),
      stat: mir::SymbolStat::Export,
      ety,
      kind: mir::SymbolKind::Function{ blok }
    };

    cre.push(this);

    Ok(())
  }
  
}


fn convert_value(val: hir::Value) -> mir::Value {
  match val {
    hir::Value::Unit => mir::Value::Unit,
    
    hir::Value::Bool(v) => mir::Value::Bool(v),
    
    hir::Value::Int(v) => mir::Value::I32(v),
  }
}
