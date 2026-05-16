/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qw/basis.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/decls.hh"
#include "qw/pretype.hh"
#include <string>

#define ef else if



namespace qw::stmts
{

  Stmt::Stmt(StmtEnum type, identy *parent, word pos)
    : identy(IdentyEnum::Stmt, parent, pos)
    , m_subType(type)
  {
    if (auto C = static_cast<CodeVar*>(this); type == StmtEnum::CodeVar) {
      if (parent && parent->type() == IdentyEnum::Stmt && static_cast<Stmt*>(parent)->subType() == StmtEnum::CodeBlock)
        static_cast<CodeBlock*>(parent)->vars().push_back(C);
    }
    else {
      if (parent && parent->type() == IdentyEnum::Stmt && static_cast<Stmt*>(parent)->subType() == StmtEnum::CodeBlock)
        static_cast<CodeBlock*>(parent)->m_codes.push_back(this);
    }
  }

  

  fun Stmt::make_CodeBlock(qw::context *ctx, identy *parent, word pos) -> CodeBlock*
  {
    auto obj = new CodeBlock(parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Stmt::make_CodeVar(qw::context *ctx, identy *parent, std::string name, types::Type *type, word pos, exprs::Expr *initialy, std::optional<word> initialy_pos) -> CodeVar*
  {
    auto obj = new CodeVar(parent, name, type, pos, initialy, initialy_pos);
    ctx->push(obj);
    return obj;
  }

  fun Stmt::make_Return(qw::context *ctx, identy *parent, word pos, exprs::Expr *expr) -> Return*
  {
    auto obj = new Return(parent, pos, expr);
    ctx->push(obj);
    return obj;
  }

  fun Stmt::make_ExprStmt(qw::context *ctx, identy *parent, exprs::Expr *expr, word pos) -> ExprStmt*
  {
    auto obj = new ExprStmt(parent, expr, pos);
    ctx->push(obj);
    return obj;
  }



  fun Stmt::dis() -> void { switch (subType())
  {
    case StmtEnum::CodeBlock: delete (CodeBlock*)this; break;
    case StmtEnum::CodeVar: delete (CodeVar*)this; break;
    
    case StmtEnum::Return: delete (Return*)this; break;

    case StmtEnum::ExprStmt: delete (ExprStmt*)this; break;
  }}




  CodeBlock::CodeBlock(identy *parent, word pos): Stmt(StmtEnum::CodeBlock, parent, pos)
  {
    if (parent && parent->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(parent)->subType() == decls::DeclEnum::Func)
      static_cast<decls::Func*>(parent)->body() = this;
  }

}
