/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include <llvm/IR/Type.h>
#include <llvm/IR/DerivedTypes.h>
#include <string_view>
#include "qw/tree/types.hh"
#include "qw/tree/decls.hh"
#include "qw/pretype.hh"

#define ef else if



namespace qw::types
{

  Type::Type(TypeEnum type, identy *parent, word pos)
    : identy(IdentyEnum::Type, parent, pos)
    , m_subType(type)
  {
    if (parent && parent->type() == IdentyEnum::Decl)
    {
      if (static_cast<decls::Decl*>(parent)->subType() == decls::DeclEnum::Type)
        static_cast<decls::Type*>(parent)->targetType() = this;
      
      ef (static_cast<decls::Decl*>(parent)->subType() == decls::DeclEnum::Func && type == TypeEnum::Func)
        static_cast<decls::Func*>(parent)->funcType() = (Func*)this;
    }
  }



  fun Type::make_Pointer(qw::context *ctx, identy *parent, word pos, types::Type *sub) -> Sub*
  {
    auto obj = new Sub(TypeEnum::Pointer, parent, pos, sub);
    ctx->push(obj);
    return obj;
  }

  fun Type::make_Reference(qw::context *ctx, identy *parent, word pos, types::Type *sub) -> Sub*
  {
    auto obj = new Sub(TypeEnum::Reference, parent, pos, sub);
    ctx->push(obj);
    return obj;
  }

  fun Type::make_CArray(qw::context *ctx, identy *parent, word pos, types::Type *sub) -> Sub*
  {
    auto obj = new Sub(TypeEnum::CArray, parent, pos, sub);
    ctx->push(obj);
    return obj;
  }

  fun Type::make_Nick(qw::context *ctx, identy *parent, word pos, std::string_view Name) -> Nick*
  {
    auto obj = new Nick(parent, pos, Name);
    ctx->push(obj);
    return obj;
  }

  fun Type::make_Func(qw::context *ctx, identy *parent, word pos, std::vector<MemberField*> pars, types::Type *ret) -> Func*
  {
    auto obj = new Func(parent, pos, pars, ret);
    ctx->push(obj);
    return obj;
  }

  fun Type::make_Record(qw::context *ctx, identy *parent, word pos, std::vector<MemberField*> vars, std::vector<decls::Type*> types, std::vector<decls::Func*> funcs) -> Record*
  {
    auto obj = new Record(parent, pos, vars, types, funcs);
    ctx->push(obj);
    return obj;
  }



  fun Type::make_MemberField(qw::context *ctx, Type *parent, word pos, std::string_view name, Type *type) -> MemberField*
  {
    auto obj = new MemberField(parent, pos, name, type);
    ctx->push(obj);
    return obj;
  }



  fun Type::dis() -> void { switch (subType())
  {
    case TypeEnum::Pointer:
    case TypeEnum::Reference:
    case TypeEnum::CArray: delete (types::Sub*)this; break;

    case TypeEnum::Nick: delete (types::Nick*)this; break;
    case TypeEnum::Func: delete (types::Func*)this; break;
    case TypeEnum::Record: delete (types::Record*)this; break;

    case TypeEnum::MemberField: delete (types::MemberField*)this; break;


    case TypeEnum::IntU8:
    case TypeEnum::IntU16:
    case TypeEnum::IntU32:
    case TypeEnum::IntU64:
    case TypeEnum::IntU128:
    
    case TypeEnum::IntS8:
    case TypeEnum::IntS16:
    case TypeEnum::IntS32:
    case TypeEnum::IntS64:
    case TypeEnum::IntS128:

    case TypeEnum::Flo16:
    case TypeEnum::Flo32:
    case TypeEnum::Flo64:
    case TypeEnum::Flo128:

    case TypeEnum::Char:
    
    case TypeEnum::Bool:

    case TypeEnum::Void:
    
    case TypeEnum::Ptr: delete (types::Type*)this; break;
  }}

  

  


  #pragma region SystemTypes
  fun Type::make_intU8(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntU8, parent, word{});
    obj->m_llvm = llvm::Type::getInt8Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intU16(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntU16, parent, word{});
    obj->m_llvm = llvm::Type::getInt16Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intU32(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntU32, parent, word{});
    obj->m_llvm = llvm::Type::getInt32Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intU64(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntU64, parent, word{});
    obj->m_llvm = llvm::Type::getInt64Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intU128(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntU128, parent, word{});
    obj->m_llvm = llvm::Type::getInt128Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun Type::make_intS8(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntS8, parent, word{});
    obj->m_llvm = llvm::Type::getInt8Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intS16(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntS16, parent, word{});
    obj->m_llvm = llvm::Type::getInt16Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intS32(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntS32, parent, word{});
    obj->m_llvm = llvm::Type::getInt32Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intS64(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntS64, parent, word{});
    obj->m_llvm = llvm::Type::getInt64Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_intS128(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::IntS128, parent, word{});
    obj->m_llvm = llvm::Type::getInt128Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun Type::make_float16(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Flo16, parent, word{});
    obj->m_llvm = llvm::Type::getHalfTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_float32(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Flo32, parent, word{});
    obj->m_llvm = llvm::Type::getFloatTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_float64(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Flo64, parent, word{});
    obj->m_llvm = llvm::Type::getDoubleTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun Type::make_float128(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Flo128, parent, word{});
    obj->m_llvm = llvm::Type::getFP128Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun Type::make_char(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Char, parent, word{});
    obj->m_llvm = llvm::Type::getInt8Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun Type::make_bool(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Bool, parent, word{});
    obj->m_llvm = llvm::Type::getInt1Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun Type::make_void(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Void, parent, word{});
    obj->m_llvm = llvm::Type::getVoidTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun Type::make_ptr(qw::context *ctx, identy *parent) -> Type* {
    auto obj = new Type(TypeEnum::Ptr, parent, word{});
    obj->m_llvm = llvm::PointerType::get(*ctx->llvm(), 0);
    ctx->push(obj);
    return obj;
  }
  #pragma endregion

}
