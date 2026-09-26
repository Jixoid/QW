use qwc_mir as mir;
use rustc_hash::FxHashMap;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockHandle(pub usize);


#[derive(Debug, Clone)]
pub enum RawTerminator {
  Jump(BlockHandle),
  Branch {
    cond: mir::Value,
    then_bb: BlockHandle,
    else_bb: BlockHandle,
  },
  Return(Option<mir::Value>),
  Unreachable,
}


#[derive(Debug, Clone)]
pub struct BasicBlockData {
  pub insts: Vec<mir::Inst>,
  pub term: Option<RawTerminator>,
}


#[derive(Debug, Clone)]
pub struct LoopFrame {
  pub continue_bb: BlockHandle,
  pub exit_bb: BlockHandle,
  pub result_slot: Option<u32>,
  pub result_ty: mir::TypeId,
}


pub struct FnBuilder {
  pub blocks: Vec<BasicBlockData>,
  pub current_bb: BlockHandle,
  pub stack: Vec<mir::TypeId>,
  next_ssa: u32,
  pub loop_stack: Vec<LoopFrame>,
  pub local_to_slot: FxHashMap<u32, u32>,
}


impl FnBuilder {

  pub fn new() -> Self {
    let entry = BasicBlockData {
      insts: vec![],
      term: None,
    };
    Self {
      blocks: vec![entry],
      current_bb: BlockHandle(0),
      stack: vec![],
      next_ssa: 0,
      loop_stack: vec![],
      local_to_slot: FxHashMap::default(),
    }
  }

  pub fn create_block(&mut self) -> BlockHandle {
    let idx = self.blocks.len();
    self.blocks.push(BasicBlockData {
      insts: vec![],
      term: None,
    });
    BlockHandle(idx)
  }

  pub fn switch_to(&mut self, bb: BlockHandle) {
    self.current_bb = bb;
  }

  pub fn is_current_terminated(&self) -> bool {
    self.blocks[self.current_bb.0].term.is_some()
  }

  pub fn emit(&mut self, expr: mir::Expr) -> Option<mir::SSA> {
    if self.is_current_terminated() {
      return None;
    }

    let ret = if expr.have_result() {
      let dest = mir::SSA::new(self.next_ssa);
      self.next_ssa += 1;
      Some(dest)
    } else {
      None
    };

    self.blocks[self.current_bb.0].insts.push(mir::Inst { kind: expr, dest: ret });
    ret
  }

  pub fn terminate(&mut self, term: RawTerminator) {
    if !self.is_current_terminated() {
      self.blocks[self.current_bb.0].term = Some(term);
    }
  }

  pub fn alloc_stack(&mut self, ty: mir::TypeId) -> u32 {
    let idx = self.stack.len() as u32;
    self.stack.push(ty);
    idx
  }

  pub fn push_loop(&mut self, frame: LoopFrame) {
    self.loop_stack.push(frame);
  }

  pub fn pop_loop(&mut self) -> Option<LoopFrame> {
    self.loop_stack.pop()
  }

  pub fn peek_loop(&self) -> Option<&LoopFrame> {
    self.loop_stack.last()
  }

  pub fn finish(self, cre: &mut mir::Krate, is_ret_unit: bool) -> (mir::BlokId, mir::Rng, mir::Rng) {
    let count = self.blocks.len();

    let mut block_ids: Vec<mir::BlokId> = Vec::with_capacity(count);
    for _ in 0..count {
      let id = cre.push(mir::Block {
        insts: mir::Rng::empty(),
        term: mir::Terminator::Unreachable,
      });
      block_ids.push(id);
    }

    for (i, block_data) in self.blocks.into_iter().enumerate() {
      let mut inst_ids = Vec::with_capacity(block_data.insts.len());
      for inst in block_data.insts {
        let id = cre.push(inst);
        inst_ids.push(id);
      }
      let insts = if inst_ids.is_empty() {
        mir::Rng::empty()
      } else {
        cre.extra(&inst_ids)
      };

      let term = match block_data.term {
        Some(RawTerminator::Jump(target)) => mir::Terminator::Jump(block_ids[target.0]),
        Some(RawTerminator::Branch { cond, then_bb, else_bb }) => mir::Terminator::Branch {
          cond,
          then_bb: block_ids[then_bb.0],
          else_bb: block_ids[else_bb.0],
        },
        Some(RawTerminator::Return(val)) => mir::Terminator::Return(val),
        Some(RawTerminator::Unreachable) => mir::Terminator::Unreachable,
        None => {
          if is_ret_unit {
            mir::Terminator::Return(None)
          } else {
            mir::Terminator::Unreachable
          }
        }
      };

      *cre.get_mut(block_ids[i]) = mir::Block {
        insts,
        term,
      };
    }

    let stack = if self.stack.is_empty() {
      mir::Rng::empty()
    } else {
      cre.extra(&self.stack)
    };

    let blocks = if block_ids.is_empty() {
      mir::Rng::empty()
    } else {
      cre.extra(&block_ids)
    };

    let entry = block_ids[0];
    (entry, blocks, stack)
  }

}


pub type BlockBuilder = FnBuilder;
