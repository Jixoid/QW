/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "Context.hh"
#include "Exprs.hh"
#include "Types.hh"
#include <llvm/IR/Instructions.h>
#include <optional>
#include <vector>
#define ef else if

using namespace std;



namespace qw::stmts
{

  // base
  struct qStmt: qIdentifier
  {
    public:
      enum qEnum: u16 {
        qse_Unknown,

        qse_CodeBlock,
        qse_CodeVar,
        
        qse_Return,
        
        qse_ExprStmt,
      };

    protected:
      qStmt(qEnum type, qIdentifier *parent, word pos);
      

    public:
      static fun make_CodeBlock(qw::context *ctx, qIdentifier *parent, word pos) -> qCodeBlock*;
      static fun make_CodeVar(qw::context *ctx, qIdentifier *parent, string name, types::qType *type, word pos, exprs::qExpr *initialy = Nil, optional<word> initialy_pos = nullopt) -> qCodeVar*;

      static fun make_Return(qw::context *ctx, qIdentifier *parent, word pos, exprs::qExpr* expr = Nil) -> qReturn*;

      static fun make_ExprStmt(qw::context *ctx, qIdentifier *parent, exprs::qExpr* expr, word pos) -> qExprStmt*;

      fun dis() -> void;


    private:
      qEnum m_subType{qse_Unknown};

    public:
      inline fun subType() const { return m_subType; }
  };



  // stmts
  struct qCodeBlock: qStmt
  {
    friend class qStmt;

    protected:
      qCodeBlock(qIdentifier *parent, word pos);
      

    private:
      vector<qStmt*> m_codes;
      vector<qCodeVar*> m_vars;

    public:
      inline fun& codes() { return m_codes; }
      inline fun& vars() { return m_vars; }
  };
  
  struct qCodeVar: qStmt
  {
    friend class qStmt;

    protected:
      inline qCodeVar(qIdentifier *parent, string name, types::qType *type, word pos, exprs::qExpr *initialy = Nil, optional<word> initialy_pos = nullopt)
        : qStmt(qse_CodeVar, parent, pos)
        , m_name(name)
        , m_targetType(type)
        , m_initialy(initialy)
        , m_initialy_pos(initialy_pos.has_value() ? *initialy_pos:word{})
      {}


    private:
      string m_name;
      types::qType *m_targetType{};
      exprs::qExpr *m_initialy{};
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
  struct qReturn: qStmt
  {
    friend class qStmt;

    protected:
      inline qReturn(qIdentifier *parent, word pos, exprs::qExpr *expr = Nil): qStmt(qse_Return, parent, pos), m_expr(expr) {}
      

    private:
      exprs::qExpr* m_expr;

    public:
      inline fun& expr() { return m_expr; }
  };



  // expr
  struct qExprStmt: qStmt
  {
    friend class qStmt;

    protected:
      inline qExprStmt(qIdentifier *parent, exprs::qExpr* expr, word pos): qStmt(qse_ExprStmt, parent, pos), m_expr(expr) {}


    private:
      exprs::qExpr* m_expr{};

    public:
      inline fun expr() { return m_expr; }
  };

}
