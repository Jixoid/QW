/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/sema/sema.hh"
#include "qw/basis.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include <cassert>
#include <expected>
#include <string>
#include <vector>

#define if_error(X)                                                                                                                                  \
  {                                                                                                                                                  \
    auto E = X;                                                                                                                                      \
    if (!E.has_value()) {                                                                                                                            \
      if (E.error()->type() == qw::diagnostic::MsgType::Fatal)                                                                                       \
        return std::unexpected(std::move(E.error()));                                                                                                \
      else {                                                                                                                                         \
        sum.add(E.error().get());                                                                                                                    \
        std::cerr << E.error();                                                                                                                      \
      }                                                                                                                                              \
    }                                                                                                                                                \
  }

#define if_except(X)                                                                                                                                 \
  {                                                                                                                                                  \
    auto E = X;                                                                                                                                      \
    if (!E.has_value())                                                                                                                              \
      return std::unexpected(std::move(E.error()));                                                                                                  \
  }

#define val_error(X)                                                                                                                                 \
  {                                                                                                                                                  \
    if (!X.has_value())                                                                                                                              \
      return std::unexpected(std::move(X.error()));                                                                                                  \
  }



namespace qw
{

  fun StmtSema::sema_Stmt(stmts::Stmt *X, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    if (X->is<stmts::CodeVar>())      return sctx.stmt->sema_VarStmt(X);
    ef (X->is<stmts::ReturnStmt>())   return sctx.stmt->sema_ReturnStmt(X, expected_ret);
    ef (X->is<stmts::ExprStmt>())     return sctx.expr->sema_Expr(X->as<stmts::ExprStmt>()->expr);
    ef (X->is<stmts::IfStmt>())       return sctx.stmt->sema_IfStmt(X, expected_ret);
    ef (X->is<stmts::WhileStmt>())    return sctx.stmt->sema_WhileStmt(X, expected_ret);
    ef (X->is<stmts::CodeBlock>())    return sctx.stmt->sema_CodeBlock(X, expected_ret);
    ef (X->is<stmts::BreakStmt>())    return sctx.stmt->sema_BreakStmt(X, expected_ret);
    ef (X->is<stmts::ContinueStmt>()) return sctx.stmt->sema_ContinueStmt(X, expected_ret);
    ef (X->is<stmts::UnsafeStmt>())   return sctx.stmt->sema_UnsafeStmt(X, expected_ret);
    else return std::unexpected(fatals::Internal_UnknownStmt().error());
  }

  fun StmtSema::sema_CodeBlock(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto block = now->as<stmts::CodeBlock>();

    for (auto X: block->vars)
      if_error(sctx.stmt->sema_VarStmt(X));

    for (auto &X : block->codes) {
      if_error(sctx.stmt->sema_Stmt(X, expected_ret));
    }

    return {};
  }

  fun StmtSema::sema_VarStmt(stmts::Stmt *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto cvar = now->as<stmts::CodeVar>();
    if (cvar->targetType) if_except(sctx.type->sema_Type(cvar->targetType, now->pos()));

    return {};
  }

  fun StmtSema::sema_ReturnStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto Ret = now->as<stmts::ReturnStmt>();
    if_except(sctx.expr->sema_Expr(Ret->expr));
    if_except(sctx.expr->sema_Convert(expected_ret, Ret->expr, now->pos()));

    return {};
  }

  fun StmtSema::sema_IfStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto if_stmt = now->as<stmts::IfStmt>();
    if_except(sctx.expr->sema_Expr(if_stmt->condition));

    auto cond_type = if_stmt->condition->targetType();
    while (cond_type->isReference()) {
      cond_type = cond_type->as<types::ReferenceType>()->sub;
    }

    if (!cond_type->isBool()) return errors::ConditionMustBeBoolean(if_stmt->condition->pos(), "if", std::string(if_stmt->condition->targetType()->typname()));

    if (if_stmt->then_block)
      if_except(sctx.stmt->sema_CodeBlock(if_stmt->then_block, expected_ret));
    
    if (if_stmt->else_block) {
      if (if_stmt->else_block->is<stmts::IfStmt>())
        if_except(sctx.stmt->sema_IfStmt(if_stmt->else_block, expected_ret))
      else
        if_except(sctx.stmt->sema_CodeBlock(if_stmt->else_block, expected_ret))
    }

    return {};
  }

  fun StmtSema::sema_WhileStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto while_stmt = now->as<stmts::WhileStmt>();
    if_except(sctx.expr->sema_Expr(while_stmt->condition));

    auto cond_type = while_stmt->condition->targetType();
    while (cond_type->isReference()) {
      cond_type = cond_type->as<types::ReferenceType>()->sub;
    }

    if (!cond_type->isBool()) return errors::ConditionMustBeBoolean(while_stmt->condition->pos(), "while", std::string(while_stmt->condition->targetType()->typname()));

    if (while_stmt->body) {
      sctx.meta->loop_stack.push_back(now);
      auto res = sctx.stmt->sema_CodeBlock(while_stmt->body, expected_ret);
      sctx.meta->loop_stack.pop_back();
      if_except(std::move(res));
    }

    return {};
  }

  fun StmtSema::sema_BreakStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    if (sctx.meta->loop_stack.empty()) return errors::StatementNotWithinLoop(now->pos(), "break");
  
    now->as<stmts::BreakStmt>()->loop = sctx.meta->loop_stack.back();
    return {};
  }

  fun StmtSema::sema_ContinueStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    if (sctx.meta->loop_stack.empty()) return errors::StatementNotWithinLoop(now->pos(), "continue");
    
    auto cnt = now->as<stmts::ContinueStmt>();
    cnt->loop = sctx.meta->loop_stack.back();
    return {};
  }

  fun StmtSema::sema_UnsafeStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto u_stmt = now->as<stmts::UnsafeStmt>();
    sctx.meta->unsafe_level++;

    auto X = u_stmt->stmt;
    auto ret = sctx.stmt->sema_Stmt(X, expected_ret);

    sctx.meta->unsafe_level--;
    return ret;
  }

}
