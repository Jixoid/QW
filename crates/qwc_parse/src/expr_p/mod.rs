/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{expr_p::helper::BinOp, parse::Ctx};

mod condition_p;
mod operator_p;
mod postfix_p;
mod literal_p;
mod route_p;
mod block_p;
mod loop_p;
mod helper;



pub struct ExprParser;

impl ExprParser {

  // Public
  pub fn read_expr(ctx: &mut Ctx) -> Result<ExprId, Message> {
    Self::read_expr_sub(ctx, 0)
  }

  fn read_expr_sub(ctx: &mut Ctx, min_bp: u8) -> Result<ExprId, Message> {
    // Starter / Pre Unary
    let mut lhs = match ctx.lex.peek()?.kind() {
      // Literal
      WK::SelfB => literal_p::pre_self_big(ctx)?,
      WK::SelfS => literal_p::pre_self_small(ctx)?,
      
      WK::Word => literal_p::pre_nick(ctx)?,
      
      WK::True | WK::False => literal_p::pre_bool(ctx)?,
      WK::Number => literal_p::pre_number(ctx)?,
      WK::String => literal_p::pre_string(ctx)?,

      WK::ParenL   => literal_p::pre_tuple(ctx)?,
      WK::BracketL => literal_p::pre_propagate(ctx)?,

      // Block
      WK::BraceL | WK::Backtick => Self::pre_block(ctx)?,
      
      WK::Unsafe  => block_p::pre_unsafe(ctx)?,
      WK::Relaxed => block_p::pre_relaxed(ctx)?,
      
      WK::Let | WK::Var => block_p::pre_let(ctx)?,
      
      // Loop
      WK::Loop  => loop_p::pre_loop(ctx)?,
      WK::While => loop_p::pre_while(ctx)?,
      WK::For   => loop_p::pre_for(ctx)?,
      
      // Route
      WK::Ret      => route_p::pre_ret(ctx)?,
      WK::Break    => route_p::pre_break(ctx)?,
      WK::Continue => route_p::pre_continue(ctx)?,
      
      // Condition
      WK::If    => condition_p::pre_if(ctx)?,
      WK::Match => condition_p::pre_match(ctx)?,
      
      // Operator
      WK::Sub | WK::Add | WK::Bang => operator_p::pre_unary(ctx)?,
      
      _ => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(ctx.lex.get()?))),
    };

    // Post Unary
    loop {
      let start = ctx.cre.get(lhs).pos;

      lhs = match ctx.lex.peek()?.kind() {
        // Access
        WK::Dot => postfix_p::post_member(ctx, start, lhs)?,

        WK::Colon2 => postfix_p::post_scope_spec(ctx, start, lhs)?,

        // Operator
        WK::Bang2 | WK::Question | WK::Amp | WK::Caret => operator_p::post_unary(ctx, start, lhs)?,

        // Call & Index
        WK::ParenL   => operator_p::post_call(ctx, start, lhs)?,
        WK::BracketL => operator_p::post_index(ctx, start, lhs)?,

        // Cast
        WK::As => postfix_p::post_cast(ctx, start, lhs)?,

        _ => break
      }
    }

    // Binary
    loop {
      let start = ctx.cre.get(lhs).pos;

      let op_tok = ctx.lex.peek()?;
      
      let (_, r_bp) = match helper::get_infix_bp(op_tok.kind()) {
        Some((l, r)) if l >= min_bp => { ctx.lex.bump()?; (l, r) },
        _ => break,
      };

      let rhs = Self::read_expr_sub(ctx, r_bp)?;
      
      let kind = match helper::parse_binary_op(op_tok.kind()) {
        BinOp::Bin(op)      => ExprKind::Binary{op, lhs, rhs},
        BinOp::AssignOp(op) => ExprKind::AssignOp{op, lhs, rhs, op_span: op_tok.into()},

        BinOp::Assign   => ExprKind::Assign{lhs, rhs, op_span: op_tok.into()},
        BinOp::Exchange => ExprKind::Exchange{lhs, rhs},
      };

      let this = Expr{
        pos: ctx.lex.pos_extend(start),
        kind 
      };

      lhs = ctx.cre.push(this);
    }

    Ok(lhs)
  }

}
