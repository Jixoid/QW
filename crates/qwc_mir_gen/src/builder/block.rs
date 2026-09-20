use qwc_mir as mir;


pub struct BlockBuilder {
  pub insts: Vec<mir::Inst>,
  pub stack: Vec<mir::TypeId>,
  next_ssa: u32,
}


impl BlockBuilder {

  pub fn new() -> Self {
    Self { insts: vec![], stack: vec![], next_ssa: 0 }
  }

  pub fn emit(&mut self, expr: mir::Expr) -> Option<mir::SSA> {
    let ret = if expr.have_result() {
      let dest = mir::SSA::new(self.next_ssa);
      self.next_ssa += 1;
      Some(dest)
    } else {
      None
    };

    self.insts.push(mir::Inst { kind: expr, dest: ret });

    ret
  }

  pub fn push_stack(&mut self, ty: mir::TypeId) -> usize {
    let idx = self.stack.len();
    self.stack.push(ty);
    idx
  }

}
