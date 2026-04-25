/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "Typs.hh"
#include <cassert>
#include <llvm/IR/Constants.h>
#include <utility>
#include "Context.hh"
#include "Exprs.hh"

#define ef else if

using namespace std;



namespace qw::exprs
{
 
  fun qExpr::make_IntegerLiteral(qw::context *ctx, qIdentifier *parent, u128 val, word pos) -> qIntegerLiteral*
  {
    auto obj = new qIntegerLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun qExpr::make_IntegerLiteral(qw::context *ctx, qIdentifier *parent, i128 val, word pos) -> qIntegerLiteral*
  {
    auto obj = new qIntegerLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun qExpr::make_FloatingLiteral(qw::context *ctx, qIdentifier *parent, f128 val, word pos) -> qFloatingLiteral*
  {
    auto obj = new qFloatingLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun qExpr::make_CharLiteral(qw::context *ctx, qIdentifier *parent, u8 val, word pos) -> qCharLiteral*
  {
    auto obj = new qCharLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun qExpr::make_BoolLiteral(qw::context *ctx, qIdentifier *parent, bool val, word pos) -> qBoolLiteral*
  {
    auto obj = new qBoolLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }

  fun qExpr::make_PtrLiteral(qw::context *ctx, qIdentifier *parent, u64 val, word pos) -> qPtrLiteral*
  {
    auto obj = new qPtrLiteral(ctx, parent, val, pos);
    ctx->push(obj);
    return obj;
  }


  fun qExpr::make_ValExpr(qw::context *ctx, qIdentifier *parent, types::qType *type, llvm::Value *value, word pos) -> qValExpr*
  {
    auto obj = new qValExpr(parent, type, value, pos);
    ctx->push(obj);
    return obj;
  }


  fun qExpr::make_Nick(qw::context *ctx, qIdentifier *parent, string name, word pos) -> qNickExpr*
  {
    auto obj = new qNickExpr(parent, name, pos);
    ctx->push(obj);
    return obj;
  }


  fun qExpr::make_PrimaryOp(qw::context *ctx, qIdentifier *parent, exprs::qExpr *_obj, vector<exprs::qExpr*> operands, word pos) -> qPrimaryOp*
  {
    auto obj = new qPrimaryOp(parent, _obj, operands, pos);
    ctx->push(obj);
    return obj;
  }

  fun qExpr::make_BinaryOp(qw::context *ctx, qIdentifier *parent, qBinaryOpEnum kind, exprs::qExpr *o1, exprs::qExpr *o2, word pos) -> qBinaryOp*
  {
    auto obj = new qBinaryOp(parent, kind, o1, o2, pos);
    ctx->push(obj);
    return obj;
  }

  

  fun qExpr::dis() -> void { switch (subType())
  {
    case qEnum::qee_Unknown: assert(false && "Unknown Type");
  
    case qEnum::qee_IntegerLiteral:  delete (qIntegerLiteral*)this; break;
    case qEnum::qee_FloatingLiteral: delete (qFloatingLiteral*)this; break;
    case qEnum::qee_CharLiteral:     delete (qCharLiteral*)this; break;
    case qEnum::qee_BoolLiteral:     delete (qBoolLiteral*)this; break;
    case qEnum::qee_PtrLiteral:      delete (qPtrLiteral*)this; break;

    case qEnum::qee_ValExpr: delete (qValExpr*)this; break;

    case qEnum::qee_Nick: delete (qNickExpr*)this; break;

    case qEnum::qee_PrimaryOp: delete (qPrimaryOp*)this; break;
    case qEnum::qee_BinaryOp: delete (qBinaryOp*)this; break;
  }}




  qIntegerLiteral::qIntegerLiteral(qw::context *ctx, qIdentifier *parent, u128 val, word pos)
    : qLiteral(qee_IntegerLiteral, parent, pos)
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

  qIntegerLiteral::qIntegerLiteral(qw::context *ctx, qIdentifier *parent, i128 val, word pos)
    : qLiteral(qee_IntegerLiteral, parent, pos)
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

  qFloatingLiteral::qFloatingLiteral(qw::context *ctx, qIdentifier *parent, f128 val, word pos)
    : qLiteral(qee_FloatingLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->flo128_t();
    llvm() = llvm::ConstantFP::get(targetType()->llvm(), val);
  }

  qCharLiteral::qCharLiteral(qw::context *ctx, qIdentifier *parent, u8 val, word pos)
    : qLiteral(qee_CharLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->char_t();
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), val, false);
  }

  qBoolLiteral::qBoolLiteral(qw::context *ctx, qIdentifier *parent, bool val, word pos)
    : qLiteral(qee_BoolLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->bool_t();
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), val, false);
  }

  qPtrLiteral::qPtrLiteral(qw::context *ctx, qIdentifier *parent, u64 val, word pos)
    : qLiteral(qee_PtrLiteral, parent, pos)
    , m_val(val)
  {
    targetType() = ctx->ptr_t();
    llvm() = llvm::ConstantInt::get(targetType()->llvm(), 0);
  }
  
}
