/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/tree/exprs.hh"
#include "qw/basis.hh"
#include "qw/control/context.hh"
#include "qw/tree/types.hh"
#include <cassert>
#include <llvm/IR/Constants.h>
#include <string>
#include <utility>

#define ef else if



namespace qw::exprs
{

  fun Expr::make_IntegerLiteral(qw::context *ctx, identy *parent, u128 val, word pos) -> Expr*
  {
    auto obj = new Expr(IntegerLiteral{ (u128)val }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_IntegerLiteral(qw::context *ctx, identy *parent, i128 val, word pos) -> Expr*
  {
    auto obj = new Expr(IntegerLiteral{ (i128)val }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_FloatingLiteral(qw::context *ctx, identy *parent, f128 val, word pos) -> Expr*
  {
    auto obj = new Expr(FloatingLiteral{ val }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_CharLiteral(qw::context *ctx, identy *parent, u8 val, word pos) -> Expr*
  {
    auto obj = new Expr(CharLiteral{ val }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_BoolLiteral(qw::context *ctx, identy *parent, bool val, word pos) -> Expr*
  {
    auto obj = new Expr(BoolLiteral{ val }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_PtrLiteral(qw::context *ctx, identy *parent, u64 val, word pos) -> Expr*
  {
    auto obj = new Expr(PtrLiteral{ val }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_StringLiteral(qw::context *ctx, identy *parent, std::string val, word pos) -> Expr*
  {
    auto obj = new Expr(StringLiteral{ std::move(val) }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_UnaryOp(qw::context *ctx, identy *parent, UnaryOpEnum kind, exprs::Expr *o1, word pos) -> Expr*
  {
    auto obj = new Expr(UnaryOp{ o1, kind }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_BinaryOp(qw::context *ctx, identy *parent, BinaryOpEnum kind, exprs::Expr *o1, exprs::Expr *o2, word pos) -> Expr*
  {
    auto obj = new Expr(BinaryOp{ o1, o2, kind }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_PostfixOp(qw::context *ctx, identy *parent, PostfixOpEnum kind, exprs::Expr *_obj, std::vector<exprs::Expr*> operands, word pos) -> Expr*
  {
    auto obj = new Expr(PostfixOp{ _obj, std::move(operands), kind }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_MemberOp(qw::context *ctx, identy *parent, MemberOpEnum kind, Expr *obj, Expr *mem, word pos) -> Expr*
  {
    auto expr = new Expr(MemberOp{obj, mem, kind}, parent, pos);
    ctx->push(expr);
    return expr;
  }

  fun Expr::make_GenericOp(qw::context *ctx, identy *parent, Expr *obj, std::vector<types::Type*> args, word pos) -> Expr*
  {
    auto expr = new Expr(GenericOp{obj, std::move(args)}, parent, pos);
    ctx->push(expr);
    return expr;
  }

  fun Expr::make_VarExpr(qw::context *ctx, identy *parent, stmts::Stmt *var, word pos) -> Expr*
  {
    auto obj = new Expr(VarExpr{ var }, parent, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_ValExpr(qw::context *ctx, identy *parent, types::Type *type, llvm::Value *value, word pos) -> Expr*
  {
    auto obj          = new Expr(ValExpr{}, parent, pos);
    obj->targetType() = type;
    obj->llvm()       = value;
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_Nick(qw::context *ctx, identy *parent, std::vector<std::string> name, word pos) -> Expr*
  {
    auto obj = new Expr(NickExpr{ { name } }, parent, pos);
    ctx->push(obj);
    return obj;
  }

}
