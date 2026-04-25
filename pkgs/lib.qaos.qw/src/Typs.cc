/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include <llvm/IR/Type.h>
#include <llvm/IR/DerivedTypes.h>
#include "Typs.hh"
#include "Decls.hh"
#include "Types.hh"

#define ef else if

using namespace std;



namespace qw::types
{

  qType::qType(qEnum type, qIdentifier *parent, word pos)
    : qIdentifier(qIdentifier::qEnum::qie_Type, parent, pos)
    , m_subType(type)
  {
    if (parent && parent->type() == qie_Decl)
    {
      if (static_cast<decls::qDecl*>(parent)->subType() == decls::qDecl::qde_TypeDecl)
        static_cast<decls::qTypeDecl*>(parent)->targetType() = this;
      
      ef (static_cast<decls::qDecl*>(parent)->subType() == decls::qDecl::qde_FuncDecl && type == qte_Func)
        static_cast<decls::qFuncDecl*>(parent)->funcType() = (qFuncType*)this;
    }
  }



  fun qType::make_Pointer(qw::context *ctx, qIdentifier *parent, word pos, types::qType *sub) -> qSubType*
  {
    auto obj = new qSubType(qte_Pointer, parent, pos, sub);
    ctx->push(obj);
    return obj;
  }

  fun qType::make_Reference(qw::context *ctx, qIdentifier *parent, word pos, types::qType *sub) -> qSubType*
  {
    auto obj = new qSubType(qte_Reference, parent, pos, sub);
    ctx->push(obj);
    return obj;
  }

  fun qType::make_CArray(qw::context *ctx, qIdentifier *parent, word pos, types::qType *sub) -> qSubType*
  {
    auto obj = new qSubType(qte_CArray, parent, pos, sub);
    ctx->push(obj);
    return obj;
  }

  fun qType::make_Nick(qw::context *ctx, qIdentifier *parent, word pos, string Name) -> qNickType*
  {
    auto obj = new qNickType(parent, pos, Name);
    ctx->push(obj);
    return obj;
  }

  fun qType::make_Func(qw::context *ctx, qIdentifier *parent, word pos, vector<qMemberFieldType*> pars, types::qType *ret) -> qFuncType*
  {
    auto obj = new qFuncType(parent, pos, pars, ret);
    ctx->push(obj);
    return obj;
  }

  fun qType::make_Record(qw::context *ctx, qIdentifier *parent, word pos, vector<qMemberFieldType*> vars, vector<decls::qTypeDecl*> types, vector<decls::qFuncDecl*> funcs) -> qRecordType*
  {
    auto obj = new qRecordType(parent, pos, vars, types, funcs);
    ctx->push(obj);
    return obj;
  }



  fun qType::make_MemberField(qw::context *ctx, qType *parent, word pos, string name, qType *type) -> qMemberFieldType*
  {
    auto obj = new qMemberFieldType(parent, pos, name, type);
    ctx->push(obj);
    return obj;
  }



  fun qType::dis() -> void { switch (subType()) {
    case types::qType::qte_Unknown: break;

    case types::qType::qte_Pointer:
    case types::qType::qte_Reference:
    case types::qType::qte_CArray: delete (types::qSubType*)this; break;

    case types::qType::qte_Nick: delete (types::qNickType*)this; break;
    case types::qType::qte_Func: delete (types::qFuncType*)this; break;
    case types::qType::qte_Record: delete (types::qRecordType*)this; break;

    case types::qType::qte_MemberField: delete (types::qMemberFieldType*)this; break;


    case types::qType::qte_IntU8:
    case types::qType::qte_IntU16:
    case types::qType::qte_IntU32:
    case types::qType::qte_IntU64:
    case types::qType::qte_IntU128:
    
    case types::qType::qte_IntS8:
    case types::qType::qte_IntS16:
    case types::qType::qte_IntS32:
    case types::qType::qte_IntS64:
    case types::qType::qte_IntS128:

    case types::qType::qte_Flo16:
    case types::qType::qte_Flo32:
    case types::qType::qte_Flo64:
    case types::qType::qte_Flo128:

    case types::qType::qte_Char:
    
    case types::qType::qte_Bool:

    case types::qType::qte_Void:
    
    case types::qType::qte_Ptr: delete (types::qType*)this; break;
  }}

  

  


  #pragma region SystemTypes
  fun qType::make_intU8(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntU8, parent, word{});
    obj->m_llvm = llvm::Type::getInt8Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intU16(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntU16, parent, word{});
    obj->m_llvm = llvm::Type::getInt16Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intU32(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntU32, parent, word{});
    obj->m_llvm = llvm::Type::getInt32Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intU64(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntU64, parent, word{});
    obj->m_llvm = llvm::Type::getInt64Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intU128(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntU128, parent, word{});
    obj->m_llvm = llvm::Type::getInt128Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun qType::make_intS8(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntS8, parent, word{});
    obj->m_llvm = llvm::Type::getInt8Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intS16(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntS16, parent, word{});
    obj->m_llvm = llvm::Type::getInt16Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intS32(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntS32, parent, word{});
    obj->m_llvm = llvm::Type::getInt32Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intS64(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntS64, parent, word{});
    obj->m_llvm = llvm::Type::getInt64Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_intS128(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_IntS128, parent, word{});
    obj->m_llvm = llvm::Type::getInt128Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun qType::make_float16(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Flo16, parent, word{});
    obj->m_llvm = llvm::Type::getHalfTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_float32(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Flo32, parent, word{});
    obj->m_llvm = llvm::Type::getFloatTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_float64(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Flo64, parent, word{});
    obj->m_llvm = llvm::Type::getDoubleTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }

  fun qType::make_float128(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Flo128, parent, word{});
    obj->m_llvm = llvm::Type::getFP128Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun qType::make_char(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Char, parent, word{});
    obj->m_llvm = llvm::Type::getInt8Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun qType::make_bool(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Bool, parent, word{});
    obj->m_llvm = llvm::Type::getInt1Ty(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun qType::make_void(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Void, parent, word{});
    obj->m_llvm = llvm::Type::getVoidTy(*ctx->llvm());
    ctx->push(obj);
    return obj;
  }



  fun qType::make_ptr(qw::context *ctx, qIdentifier *parent) -> qType* {
    auto obj = new qType(qte_Ptr, parent, word{});
    obj->m_llvm = llvm::PointerType::get(*ctx->llvm(), 0);
    ctx->push(obj);
    return obj;
  }
  #pragma endregion

}
