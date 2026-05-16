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

  enum struct StmtEnum: u16 {
    CodeBlock,
    CodeVar,
    
    Return,
    
    ExprStmt,
  };


  // base
  struct Stmt: identy
  {
    protected:
      Stmt(StmtEnum type, identy *parent, word pos);
      

    public:
      static fun make_CodeBlock(qw::context *ctx, identy *parent, word pos) -> CodeBlock*;
      static fun make_CodeVar(qw::context *ctx, identy *parent, std::string name, types::Type *type, word pos, exprs::Expr *initialy = nil, std::optional<word> initialy_pos = std::nullopt) -> CodeVar*;

      static fun make_Return(qw::context *ctx, identy *parent, word pos, exprs::Expr* expr = nil) -> Return*;

      static fun make_ExprStmt(qw::context *ctx, identy *parent, exprs::Expr* expr, word pos) -> ExprStmt*;

      fun dis() -> void;


    private:
      StmtEnum m_subType;

    public:
      inline fun subType() const { return m_subType; }
  };



  // stmts
  struct CodeBlock: Stmt
  {
    friend struct Stmt;

    protected:
      CodeBlock(identy *parent, word pos);
      

    private:
      std::vector<Stmt*> m_codes;
      std::vector<CodeVar*> m_vars;

    public:
      inline fun& codes() { return m_codes; }
      inline fun& vars() { return m_vars; }
  };
  
  struct CodeVar: Stmt
  {
    friend struct Stmt;

    protected:
      inline CodeVar(identy *parent, std::string name, types::Type *type, word pos, exprs::Expr *initialy = nil, std::optional<word> initialy_pos = std::nullopt)
        : Stmt(StmtEnum::CodeVar, parent, pos)
        , m_name(name)
        , m_targetType(type)
        , m_initialy(initialy)
        , m_initialy_pos(initialy_pos.has_value() ? *initialy_pos:word{})
      {}


    private:
      std::string m_name;
      types::Type *m_targetType{};
      exprs::Expr *m_initialy{};
      word m_initialy_pos{};
      llvm::AllocaInst *m_llvm{};

    public:
      inline fun  name() { return m_name; }
      inline fun& targetType() { return m_targetType; }
      inline fun& initialy() { return m_initialy; }
      inline fun  initialy_pos() { return m_initialy_pos; }
      inline fun& llvm() { return m_llvm; }
  };



  // control
  struct Return: Stmt
  {
    friend struct Stmt;

    protected:
      inline Return(identy *parent, word pos, exprs::Expr *expr = nil): Stmt(StmtEnum::Return, parent, pos), m_expr(expr) {}
      

    private:
      exprs::Expr* m_expr;

    public:
      inline fun& expr() { return m_expr; }
  };



  // expr
  struct ExprStmt: Stmt
  {
    friend struct Stmt;

    protected:
      inline ExprStmt(identy *parent, exprs::Expr* expr, word pos): Stmt(StmtEnum::ExprStmt, parent, pos), m_expr(expr) {}


    private:
      exprs::Expr* m_expr{};

    public:
      inline fun expr() { return m_expr; }
  };

}
