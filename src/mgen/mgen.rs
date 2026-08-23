use crate::{diagnostic::Message, hir, mir};



pub struct GenContext<'a,'h,'d> {
  pub hir: &'a hir::Crate<'d>,
  pub mir: &'h mut mir::Crate,
}


pub struct MGen;

impl MGen {

  pub fn lower<'a,'d>(hir: &'a hir::Crate<'d>) -> Result<mir::Crate, Message> {
    let mut ret = mir::Crate::new();
    
    let mut gctx = GenContext {
      hir,
      mir: &mut ret,
    };

    //let _a = ItemGen::low(&mut gctx, hir.root)?;

    Ok(ret)
  }

}
