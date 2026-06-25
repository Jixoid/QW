/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/tree/stmts.hh"
#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include <string>

#define ef else if



namespace qw::stmts
{

  Stmt::Stmt(StmtVari vari, identy *parent, word pos)
    : identy(IdentyEnum::Stmt, parent, pos)
    , m_vari(std::move(vari))
  {
    if (parent && parent->type() == IdentyEnum::Stmt) {
      auto parentStmt = static_cast<Stmt *>(parent);

      if (parentStmt->is<CodeBlock>()) {
        auto cb = parentStmt->as<CodeBlock>();

        if (this->is<CodeVar>()) {
          cb->vars.push_back(this);
        } else {
          cb->codes.push_back(this);
        }
      }
    }
    ef (this->is<CodeBlock>() && parent && parent->type() == IdentyEnum::Decl)
    {
      auto parentDecl = static_cast<decls::Decl *>(parent);

      if (parentDecl->is<decls::FuncDecl>())
        parentDecl->as<decls::FuncDecl>()->body = this;
      ef (parentDecl->is<decls::ConstructorDecl>())
        parentDecl->as<decls::ConstructorDecl>()->body = this;
    }
  }

  fun Stmt::make_CodeBlock(qw::context *ctx, identy *parent, word pos) -> Stmt*
  {
    auto obj = new Stmt(CodeBlock{}, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Stmt::make_CodeVar(qw::context *ctx, identy *parent, std::string name, types::Type *type, word pos, exprs::Expr *initialy, std::optional<word> initialy_pos) -> Stmt*
  {
    auto obj = new Stmt(CodeVar{ std::move(name), type, nullptr }, parent, pos);
    ctx->push(obj);

    if (initialy) {
      auto ass = exprs::Expr::make_BinaryOp(
        ctx, parent, exprs::BinaryOpEnum::Assign, exprs::Expr::make_VarExpr(ctx, parent, obj, pos), initialy, initialy_pos.value()
      );
      stmts::Stmt::make_ExprStmt(ctx, parent, ass, pos);
    }

    return obj;
  }

  fun Stmt::make_ExprStmt(qw::context *ctx, identy *parent, exprs::Expr *expr, word pos) -> Stmt*
  {
    auto obj = new Stmt(ExprStmt{ expr }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Stmt::make_Return(qw::context *ctx, identy *parent, word pos, exprs::Expr *expr) -> Stmt* 
  {
    auto obj = new Stmt(ReturnStmt{ expr }, parent, pos);
    ctx->push(obj);
    return obj;
  }

}
