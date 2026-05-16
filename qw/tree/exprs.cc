/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qw/basis.hh"
#include "qw/tree/types.hh"
#include "qw/tree/exprs.hh"
#include "qw/control/context.hh"
#include <cassert>
#include <string_view>
#include <utility>
#include <llvm/IR/Constants.h>

#define ef else if



namespace qw::exprs
{
 
  fun Expr::make_IntegerLiteral(qw::context *ctx, identy *parent, u128 val, word pos) -> IntegerLiteral*
  {
    auto obj = new IntegerLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_IntegerLiteral(qw::context *ctx, identy *parent, i128 val, word pos) -> IntegerLiteral*
  {
    auto obj = new IntegerLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_FloatingLiteral(qw::context *ctx, identy *parent, f128 val, word pos) -> FloatingLiteral*
  {
    auto obj = new FloatingLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_CharLiteral(qw::context *ctx, identy *parent, u8 val, word pos) -> CharLiteral*
  {
    auto obj = new CharLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_BoolLiteral(qw::context *ctx, identy *parent, bool val, word pos) -> BoolLiteral*
  {
    auto obj = new BoolLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_PtrLiteral(qw::context *ctx, identy *parent, u64 val, word pos) -> PtrLiteral*
  {
    auto obj = new PtrLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }


  fun Expr::make_ValExpr(qw::context *ctx, identy *parent, types::Type *type, llvm::Value *value, word pos) -> ValExpr*
  {
    auto obj = new ValExpr(parent, type, value, pos);
    ctx->push(obj);
    return obj;
  }


  fun Expr::make_Nick(qw::context *ctx, identy *parent, std::string_view name, word pos) -> NickExpr*
  {
    auto obj = new NickExpr(parent, name, pos);
    ctx->push(obj);
    return obj;
  }


  fun Expr::make_PrimaryOp(qw::context *ctx, identy *parent, exprs::Expr *_obj, std::vector<exprs::Expr*> operands, word pos) -> PrimaryOp*
  {
    auto obj = new PrimaryOp(parent, _obj, operands, pos);
    ctx->push(obj);
    return obj;
  }

  fun Expr::make_BinaryOp(qw::context *ctx, identy *parent, BinaryOpEnum kind, exprs::Expr *o1, exprs::Expr *o2, word pos) -> BinaryOp*
  {
    auto obj = new BinaryOp(parent, kind, o1, o2, pos);
    ctx->push(obj);
    return obj;
  }

  

  fun Expr::dis() -> void { switch (subType())
  {
    case ExprEnum::IntegerLiteral:  delete (IntegerLiteral*)this; break;
    case ExprEnum::FloatingLiteral: delete (FloatingLiteral*)this; break;
    case ExprEnum::CharLiteral:     delete (CharLiteral*)this; break;
    case ExprEnum::BoolLiteral:     delete (BoolLiteral*)this; break;
    case ExprEnum::PtrLiteral:      delete (PtrLiteral*)this; break;

    case ExprEnum::ValExpr: delete (ValExpr*)this; break;

    case ExprEnum::Nick: delete (NickExpr*)this; break;

    case ExprEnum::PrimaryOp: delete (PrimaryOp*)this; break;
    case ExprEnum::BinaryOp: delete (BinaryOp*)this; break;
  }}




  IntegerLiteral::IntegerLiteral(qw::context *ctx, identy *parent, u128 val, word pos)
    : Literal(ExprEnum::IntegerLiteral, parent, pos)
    , m_val(val)
  {
    if (std::in_range<u8>(val))   targetType() = ctx->intU8_t();
    ef (std::in_range<u16>(val))  targetType() = ctx->intU16_t();
    ef (std::in_range<u32>(val))  targetType() = ctx->intU32_t();
    ef (std::in_range<u64>(val))  targetType() = ctx->intU64_t();
    else
      targetType() = ctx->intU128_t();
    
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), val, false);
  }

  IntegerLiteral::IntegerLiteral(qw::context *ctx, identy *parent, i128 val, word pos)
    : Literal(ExprEnum::IntegerLiteral, parent, pos)
    , m_val(val)
  {
    if (std::in_range<i8>(val))   targetType() = ctx->intS8_t();
    ef (std::in_range<i16>(val))  targetType() = ctx->intS16_t();
    ef (std::in_range<i32>(val))  targetType() = ctx->intS32_t();
    ef (std::in_range<i64>(val))  targetType() = ctx->intS64_t();
    else
      targetType() = ctx->intS128_t();
    
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), val, true);
  }

  FloatingLiteral::FloatingLiteral(qw::context *ctx, identy *parent, f128 val, word pos)
    : Literal(ExprEnum::FloatingLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->flo128_t();
    llvm() = llvm::ConstantFP::get(targetType()->llvm(), val);
  }

  CharLiteral::CharLiteral(qw::context *ctx, identy *parent, u8 val, word pos)
    : Literal(ExprEnum::CharLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->char_t();
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), val, false);
  }

  BoolLiteral::BoolLiteral(qw::context *ctx, identy *parent, bool val, word pos)
    : Literal(ExprEnum::BoolLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->bool_t();
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), val, false);
  }

  PtrLiteral::PtrLiteral(qw::context *ctx, identy *parent, u64 val, word pos)
    : Literal(ExprEnum::PtrLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->ptr_t();
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), 0);
  }
  
}
