/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "Stmts.hh"
#include "Exprs.hh"
#include "Decls.hh"
#include "Types.hh"
#include <string>

#define ef else if

using namespace std;



namespace qw::stmts
{

  qStmt::qStmt(qEnum type, qIdentifier *parent, word pos)
    : qIdentifier(qIdentifier::qEnum::qie_Stmt, parent, pos)
    , m_subType(type)
  {
    if (auto C = static_cast<qCodeVar*>(this); type == qse_CodeVar) {
      if (parent && parent->type() == qie_Stmt && static_cast<qStmt*>(parent)->subType() == qse_CodeBlock)
        static_cast<qCodeBlock*>(parent)->vars().push_back(C);
    }
    else {
      if (parent && parent->type() == qie_Stmt && static_cast<qStmt*>(parent)->subType() == qse_CodeBlock)
        static_cast<qCodeBlock*>(parent)->m_codes.push_back(this);
    }
  }

  

  fun qStmt::make_CodeBlock(qw::context *ctx, qIdentifier *parent, word pos) -> qCodeBlock*
  {
    auto obj = new qCodeBlock(parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun qStmt::make_CodeVar(qw::context *ctx, qIdentifier *parent, string name, types::qType *type, word pos, exprs::qExpr *initialy, optional<word> initialy_pos) -> qCodeVar*
  {
    auto obj = new qCodeVar(parent, name, type, pos, initialy, initialy_pos);
    ctx->push(obj);
    return obj;
  }

  fun qStmt::make_Return(qw::context *ctx, qIdentifier *parent, word pos, exprs::qExpr *expr) -> qReturn*
  {
    auto obj = new qReturn(parent, pos, expr);
    ctx->push(obj);
    return obj;
  }

  fun qStmt::make_ExprStmt(qw::context *ctx, qIdentifier *parent, exprs::qExpr *expr, word pos) -> qExprStmt*
  {
    auto obj = new qExprStmt(parent, expr, pos);
    ctx->push(obj);
    return obj;
  }



  fun qStmt::dis() -> void { switch (subType()) {
    case qStmt::qEnum::qse_Unknown: assert(false && "Unknown Type");
    
    case qStmt::qEnum::qse_CodeBlock: delete (qCodeBlock*)this; break;
    case qStmt::qEnum::qse_CodeVar: delete (qCodeVar*)this; break;
    
    case qStmt::qEnum::qse_Return: delete (qReturn*)this; break;

    case qStmt::qEnum::qse_ExprStmt: delete (qExprStmt*)this; break;
  }}




  qCodeBlock::qCodeBlock(qIdentifier *parent, word pos): qStmt(qse_CodeBlock, parent, pos)
  {
    if (parent && parent->type() == qie_Decl && static_cast<decls::qDecl*>(parent)->subType() == decls::qDecl::qde_FuncDecl)
      static_cast<decls::qFuncDecl*>(parent)->body() = this;
  }

}
