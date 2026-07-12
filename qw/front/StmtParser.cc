/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/front/front.hh"
#include "qw/basis.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/lexer/lexer.hh"
#include "qw/pretype.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include <expected>
#include <fcntl.h>
#include <optional>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if

#define Require(X) \
  if (!X) \
    return fatals::FileEndedButContextNotFinished();

#define Require_Word(X, C) \
  { \
    Require(X); \
    if (lexer.kind(X.view()[0]) != CharKind::Word) { \
      auto E = errors::ExpectedAWord(X, X.str()); \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
      C; \
    } \
  }

#define expected(LEX, T) \
  { \
    auto X = LEX; \
    Require(X) ef(X.view() != T) return errors::ExpectedIdentifierBut(X, X.str(), T); \
  }

#define expected2(LEX, T, T2) \
  { \
    auto X = LEX; \
    Require(X) ef(X.view() != T || X.view() != T2) return errors::ExpectedIdentifierBut2(X, X.str(), T, T2); \
  }

#define if_error(X) \
  { \
    auto E = X; \
    if (!E.has_value()) { \
      if (E.error()->type() == qw::diagnostic::MsgType::Fatal) \
        return std::unexpected(std::move(E.error())); \
      else { \
        sum.add(E.error().get()); \
        std::cerr << E.error(); \
      } \
    } \
  }

#define if_except(X) \
  { \
    auto E = X; \
    if (!E.has_value()) \
      return std::unexpected(std::move(E.error())); \
  }

#define if_except_ref(X) \
  { \
    auto &E = X; \
    if (!E.has_value()) \
      return std::unexpected(std::move(E.error())); \
  }

#define print(E) \
  { \
    if (E.error()->type() == qw::diagnostic::MsgType::Fatal) \
      return std::unexpected(std::move(E.error())); \
    else { \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
    } \
  }

#define val_error(X) \
  { \
    if (!X.has_value()) \
      return std::unexpected(std::move(X.error())); \
  }



namespace qw
{

  fun StmtParser::read_SingleStmt(identy *self, std::optional<word> predefined_id) -> std::expected<void, uptr<diagnostic::message>> {
    auto ID = predefined_id ? *predefined_id : lex();
    if (!predefined_id && !ID) return std::unexpected(errors::ExpectedIdentifierBut(lex.last(), lex.last().str(), "a statement"));

    if (ID.view() == "ret")      if_error(read_ReturnStmt(self))
    ef (ID.view() == "var")      if_error(read_VarStmt(self))
    ef (ID.view() == "let")      if_error(read_LetStmt(self))
    ef (ID.view() == "if")       if_error(read_IfStmt(self))
    ef (ID.view() == "while")    if_error(read_WhileStmt(self))
    ef (ID.view() == "break") {
      stmts::Stmt::make_Break(ctx, self, ID);
      expected(lex(), ";");
    }
    ef (ID.view() == "continue") {
      stmts::Stmt::make_Continue(ctx, self, ID);
      expected(lex(), ";");
    }
    ef (ID.view() == "unsafe") {
      if_error(read_UnsafeStmt(self, ID));
    }
    else {
      lex.store(ID);
      auto expr = pctx.expr->read_Expr(self, Precedence::Lowest);
      val_error(expr);
      stmts::Stmt::make_ExprStmt(ctx, self, *expr, ID);
      expected(lex(), ";");
    }

    return {};
  }

  fun StmtParser::read_BlockOrStmt(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
    auto token = lex();
    Require(token);
    
    if (token.is(WordKind::CurlyBracketBeg)) {
      return read_CodeBlock(parent);
    }
    else {
      lex.store(token);
      auto self = stmts::Stmt::make_CodeBlock(ctx, parent, token);
      if_error(read_SingleStmt(self, std::nullopt));
      return self;
    }
  }

  fun StmtParser::read_CodeBlock(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
    auto self = stmts::Stmt::make_CodeBlock(ctx, parent, lex.last());

    while (true) {
      auto ID = lex();
      Require(ID);

      if (ID.is(WordKind::CurlyBracketEnd)) break;
      
      if_error(read_SingleStmt(self, ID));
    }

    return self;
  }

  fun StmtParser::read_IfStmt(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
    word pos = lex.last();
    expected(lex(), "(");
    auto cond = pctx.expr->read_Expr(parent, Precedence::Lowest);
    val_error(cond);
    expected(lex(), ")");

    auto if_stmt = stmts::Stmt::make_IfStmt(ctx, parent, pos, *cond, nullptr, nullptr);

    auto then_ret = read_BlockOrStmt(if_stmt);
    if (!then_ret) return std::unexpected(std::move(then_ret.error()));
    if_stmt->as<stmts::IfStmt>()->then_block = *then_ret;

    auto nxt = lex();
    if (!nxt) return {};

    if (nxt.view() == "else") {
      auto nxt2 = lex();
      Require(nxt2);

      if (nxt2.view() == "if") {
        auto elif_ret = read_IfStmt(if_stmt);
        if (!elif_ret) return std::unexpected(std::move(elif_ret.error()));
        if_stmt->as<stmts::IfStmt>()->else_block = *elif_ret;
      }
      else {
        lex.store(nxt2);
        auto else_ret = read_BlockOrStmt(if_stmt);
        if (!else_ret) return std::unexpected(std::move(else_ret.error()));
        if_stmt->as<stmts::IfStmt>()->else_block = *else_ret;
      }
    }
    ef (nxt.view() == "ef") {
      auto elif_ret = read_IfStmt(if_stmt);
      if (!elif_ret) return std::unexpected(std::move(elif_ret.error()));
      if_stmt->as<stmts::IfStmt>()->else_block = *elif_ret;
    }
    else {
      lex.store(nxt);
    }

    return if_stmt;
  }

  fun StmtParser::read_WhileStmt(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
    word pos = lex.last();
    expected(lex(), "(");
    auto cond = pctx.expr->read_Expr(parent, Precedence::Lowest);
    val_error(cond);
    expected(lex(), ")");

    auto while_stmt = stmts::Stmt::make_WhileStmt(ctx, parent, pos, *cond, nullptr);

    auto body_ret = read_BlockOrStmt(while_stmt);
    if (!body_ret) return std::unexpected(std::move(body_ret.error()));
    while_stmt->as<stmts::WhileStmt>()->body = *body_ret;

    return while_stmt;
  }

  fun StmtParser::read_VarStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<word> Vars;

    // Name
    re:
    auto Name = lex();
    Require(Name);
    Vars.push_back(Name);

    auto Colon = lex();
    Require(Colon);

    types::Type *Type = nullptr;

    if (Colon.is(WordKind::Assign) || Colon.is(WordKind::Semicolon)) {
      lex.store(Colon);
    }
    else {
      if (Colon.is(WordKind::Comma)) goto re;
      ef (Colon.is(WordKind::Colon));
      else
        return errors::ExpectedIdentifierBut2(Colon, Colon.str(), ",", ":");

      // Type
      auto TypeRes = pctx.type->read_Type(parent, true);
      val_error(TypeRes);
      Type = *TypeRes;
    }

    // Value
    auto Assi = lex();
    Require(Assi);

    if (Assi.is(WordKind::Assign)) {
      if (Vars.size() != 1)
        return std::unexpected(errors::OnlyOneVariableCanBeInitialized(Assi, Assi.str()));

      auto Expr = pctx.expr->read_Expr(parent, Precedence::Lowest);
      val_error(Expr);

      stmts::Stmt::make_CodeVar(ctx, parent, Name.str(), Type, Name, *Expr, Assi);
    }
    else {
      if (!Type)
        return std::unexpected(errors::TypeRequiredWithoutAssignment(Assi, Name.str()));
      lex.store(Assi);

      for (auto& var : Vars)
        stmts::Stmt::make_CodeVar(ctx, parent, var.str(), Type, var, nullptr, std::nullopt);
    }

    expected(lex(), ";");

    return {};
  }

  fun StmtParser::read_LetStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<word> Vars;

    // Name
    re:
    auto Name = lex();
    Require(Name);
    Vars.push_back(Name);

    auto Colon = lex();
    Require(Colon);

    types::Type *Type = nullptr;

    if (Colon.is(WordKind::Assign) || Colon.is(WordKind::Semicolon)) {
      lex.store(Colon);
    }
    else {
      if (Colon.is(WordKind::Comma)) goto re;
      ef (Colon.is(WordKind::Colon));
      else
        return std::unexpected(errors::ExpectedIdentifierBut2(Colon, Colon.str(), ",", ":"));

      // Type
      auto TypeRes = pctx.type->read_Type(parent, true);
      val_error(TypeRes);
      Type = *TypeRes;
    }

    // Value
    auto Assi = lex();
    Require(Assi);

    if (Assi.is(WordKind::Assign)) {
      if (Vars.size() != 1)
        return std::unexpected(errors::OnlyOneVariableCanBeInitialized(Assi, Assi.str()));

      auto Expr = pctx.expr->read_Expr(parent, Precedence::Lowest);
      val_error(Expr);

      stmts::Stmt::make_CodeVar(ctx, parent, Name.str(), Type, Name, *Expr, Assi);
    }
    else {
      if (!Type)
        return std::unexpected(errors::TypeRequiredWithoutAssignment(Assi, Name.str()));
      lex.store(Assi);

      for (auto &v : Vars)
        stmts::Stmt::make_CodeVar(ctx, parent, v.str(), Type, v, nullptr, std::nullopt);
    }

    expected(lex(), ";");

    return {};
  }

  fun StmtParser::read_ReturnStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Pos = lex.last();

    auto Expr = pctx.expr->read_Expr(parent, Precedence::Lowest);
    val_error(Expr);
    expected(lex(), ";");

    auto self = stmts::Stmt::make_Return(ctx, parent, Pos, *Expr);

    return {};
  }

  fun StmtParser::read_UnsafeStmt(identy *parent, word pos) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
    auto next = lex();
    Require(next);

    auto obj = stmts::Stmt::make_Unsafe(ctx, parent, pos, nullptr);

    if (next.is(WordKind::CurlyBracketBeg)) {
      auto blk = read_CodeBlock(obj);
      if (!blk) return std::unexpected(std::move(blk.error()));
      obj->as<stmts::UnsafeStmt>()->stmt = *blk;
    }
    else {
      lex.store(next);
      auto expr = pctx.expr->read_Expr(obj, Precedence::Lowest);
      if (!expr) return std::unexpected(std::move(expr.error()));
      obj->as<stmts::UnsafeStmt>()->stmt = stmts::Stmt::make_ExprStmt(ctx, obj, *expr, pos);
      expected(lex(), ";");
    }

    return obj;
  }

}
