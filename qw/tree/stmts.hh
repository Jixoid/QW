/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "qw/basis.hh"
#include "qw/control/context.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/types.hh"
#include <llvm/IR/Instructions.h>
#include <optional>
#include <string>
#include <vector>

#define ef else if



namespace qw::stmts
{

  struct CodeBlock { std::vector<Stmt*> codes{}; std::vector<Stmt*> vars{}; };
  struct CodeVar   { std::string name; types::Type *targetType{}; llvm::AllocaInst *llvm{}; };
  struct ExprStmt  { exprs::Expr *expr{}; };

  struct IfStmt    { exprs::Expr *condition{}; Stmt *then_block{}; Stmt *else_block{}; /* optional */ };
  struct WhileStmt { exprs::Expr *condition{}; Stmt *body{}; };

  struct ReturnStmt   { exprs::Expr *expr{}; };
  struct BreakStmt    {};
  struct ContinueStmt {};

  using StmtVari = std::variant<
    CodeBlock, CodeVar, ExprStmt,

    IfStmt, WhileStmt,

    ReturnStmt, BreakStmt, ContinueStmt
  >;


  struct Stmt: identy
  {
    protected:
      Stmt(StmtVari vari, identy *parent, word pos);

    public:
      static fun make_CodeBlock(qw::context *ctx, identy *parent, word pos) -> Stmt*;
      static fun make_CodeVar(qw::context *ctx, identy *parent, std::string name, types::Type *type, word pos, exprs::Expr *initialy = nil, std::optional<word> initialy_pos = std::nullopt) -> Stmt*;

      static fun make_ExprStmt(qw::context *ctx, identy *parent, exprs::Expr *expr, word pos) -> Stmt*;

      static fun make_Return(qw::context *ctx, identy *parent, word pos, exprs::Expr *expr = nil) -> Stmt*;

    private:
      StmtVari m_vari;

    public:
      inline fun& vari() { return m_vari; }

    public:
      template<typename T>
      inline fun is() { return std::holds_alternative<T>(m_vari); }

      template<typename T>
      inline fun as() { return &std::get<T>(m_vari); }
    
  };

}
