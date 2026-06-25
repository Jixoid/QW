/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/tree/types.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include <llvm/IR/DerivedTypes.h>
#include <llvm/IR/Type.h>
#include <string>
#include <string_view>
#include <utility>

#define ef else if



namespace qw::types
{

  Type::Type(TypeVari vari, std::string cached_name)
    : m_vari(std::move(vari))
    , m_cached_typname(std::move(cached_name))
  {}

  fun Type::typname() -> std::string_view { return m_cached_typname; }

  fun Type::isInteger() -> bool
  {
    if (!is<PrimitiveType>()) return false;

    auto kind = as<PrimitiveType>()->kind;
    return (kind >= PrimitiveEnum::I8 && kind <= PrimitiveEnum::I128) || (kind >= PrimitiveEnum::U8 && kind <= PrimitiveEnum::U128);
  }

  fun Type::isFloat() -> bool
  {
    if (!is<PrimitiveType>()) return false;

    auto kind = as<PrimitiveType>()->kind;
    return (kind >= PrimitiveEnum::F16 && kind <= PrimitiveEnum::F32) || (kind >= PrimitiveEnum::F64 && kind <= PrimitiveEnum::F128);
  }

  fun Type::isSigned() -> bool
  {
    if (!is<PrimitiveType>()) return false;

    auto kind = as<PrimitiveType>()->kind;
    return (kind >= PrimitiveEnum::I8 && kind <= PrimitiveEnum::I128);
  }

  fun Type::isChar() -> bool
  {
    if (!is<PrimitiveType>()) return false;

    return as<PrimitiveType>()->kind == PrimitiveEnum::Char;
  }

  fun Type::isBool() -> bool
  {
    if (!is<PrimitiveType>()) return false;

    return as<PrimitiveType>()->kind == PrimitiveEnum::Bool;
  }

  fun Type::intBit() -> u8
  {
    if (!is<PrimitiveType>()) return 0;

    auto kind = as<PrimitiveType>()->kind;
    switch (kind) {
      case PrimitiveEnum::I8:
      case PrimitiveEnum::U8:
      case PrimitiveEnum::Char: return 8;
      case PrimitiveEnum::I16:
      case PrimitiveEnum::U16: return 16;
      case PrimitiveEnum::I32:
      case PrimitiveEnum::U32: return 32;
      case PrimitiveEnum::I64:
      case PrimitiveEnum::U64: return 64;
      case PrimitiveEnum::I128:
      case PrimitiveEnum::U128: return 128;
      case PrimitiveEnum::Bool: return 1;
      default: return 0;
    }
  }

  fun Type::make_Primitive(qw::context *ctx, PrimitiveEnum kind) -> Type*
  {
    llvm::Type *llvm = nil;
    std::string name;

    switch (kind) {
      case PrimitiveEnum::I8:   name = "i8";   llvm = llvm::Type::getInt8Ty(*ctx->llvm());   break;
      case PrimitiveEnum::I16:  name = "i16";  llvm = llvm::Type::getInt16Ty(*ctx->llvm());  break;
      case PrimitiveEnum::I32:  name = "i32";  llvm = llvm::Type::getInt32Ty(*ctx->llvm());  break;
      case PrimitiveEnum::I64:  name = "i64";  llvm = llvm::Type::getInt64Ty(*ctx->llvm());  break;
      case PrimitiveEnum::I128: name = "i128"; llvm = llvm::Type::getInt128Ty(*ctx->llvm()); break;
      case PrimitiveEnum::U8:   name = "u8";   llvm = llvm::Type::getInt8Ty(*ctx->llvm());   break;
      case PrimitiveEnum::U16:  name = "u16";  llvm = llvm::Type::getInt16Ty(*ctx->llvm());  break;
      case PrimitiveEnum::U32:  name = "u32";  llvm = llvm::Type::getInt32Ty(*ctx->llvm());  break;
      case PrimitiveEnum::U64:  name = "u64";  llvm = llvm::Type::getInt64Ty(*ctx->llvm());  break;
      case PrimitiveEnum::U128: name = "u128"; llvm = llvm::Type::getInt128Ty(*ctx->llvm()); break;
      case PrimitiveEnum::F16:  name = "f16";  llvm = llvm::Type::getHalfTy(*ctx->llvm());   break;
      case PrimitiveEnum::F32:  name = "f32";  llvm = llvm::Type::getFloatTy(*ctx->llvm());  break;
      case PrimitiveEnum::F64:  name = "f64";  llvm = llvm::Type::getDoubleTy(*ctx->llvm()); break;
      case PrimitiveEnum::F128: name = "f128"; llvm = llvm::Type::getFP128Ty(*ctx->llvm());  break;
      case PrimitiveEnum::Bool: name = "bool"; llvm = llvm::Type::getInt1Ty(*ctx->llvm());   break;
      case PrimitiveEnum::Char: name = "char"; llvm = llvm::Type::getInt8Ty(*ctx->llvm());   break;
      case PrimitiveEnum::Void: name = "void"; llvm = llvm::Type::getVoidTy(*ctx->llvm());   break;
      case PrimitiveEnum::Ptr:  name = "ptr";  llvm = llvm::PointerType::get(*ctx->llvm(), 0); break;
    }

    auto obj = new Type(PrimitiveType{kind}, name);
    obj->llvm() = llvm;

    ctx->m_types.push_back(obj);
    return obj;
  }

  fun Type::make_Pointer(qw::context *ctx, Type *sub) -> Type*
  {
    if (auto it = ctx->m_ptr_pool.find(sub); it != ctx->m_ptr_pool.end())
      return it->second;

    auto obj = new Type(PointerType{sub}, std::string(sub->typname()) + "^");
    obj->llvm() = llvm::PointerType::getUnqual(*ctx->llvm());
    ctx->m_types.push_back(obj);
    ctx->m_ptr_pool[sub] = obj;
    return obj;
  }

  fun Type::make_Reference(qw::context *ctx, Type *sub) -> Type*
  {
    if (sub->is<ReferenceType>())
      return sub;

    if (auto it = ctx->m_ref_pool.find(sub); it != ctx->m_ref_pool.end())
      return it->second;

    auto obj = new Type(ReferenceType{sub}, std::string(sub->typname()) + "&");
    obj->llvm() = llvm::PointerType::getUnqual(*ctx->llvm());
    ctx->m_types.push_back(obj);
    ctx->m_ref_pool[sub] = obj;
    return obj;
  }

  fun Type::make_ZArray(qw::context *ctx, Type *sub) -> Type*
  {
    if (auto it = ctx->m_zarray_pool.find(sub); it != ctx->m_zarray_pool.end())
      return it->second;

    auto obj = new Type(ZArrayType{sub}, std::string(sub->typname()) + "[]");
    obj->llvm() = llvm::PointerType::getUnqual(*ctx->llvm());
    ctx->m_types.push_back(obj);
    ctx->m_zarray_pool[sub] = obj;
    return obj;
  }

  fun Type::make_PArray(qw::context *ctx, Type *sub, u32 size) -> Type*
  {
    auto key = std::make_pair(sub, size);
    if (auto it = ctx->m_parray_pool.find(key); it != ctx->m_parray_pool.end())
      return it->second;

    auto obj = new Type(PArrayType{sub, size}, std::string(sub->typname()) + "[" + std::to_string(size) + "]");

    ctx->m_types.push_back(obj);
    ctx->m_parray_pool[key] = obj;
    return obj;
  }

  fun Type::make_Func(qw::context *ctx, std::vector<FieldType> pars, Type *ret) -> Type*
  {
    std::vector<Type*> param_types;
    param_types.reserve(pars.size());
    for (const auto &p: pars)
      param_types.push_back(p.type);
    

    auto key = std::make_pair(param_types, ret);
    if (auto it = ctx->m_func_pool.find(key); it != ctx->m_func_pool.end())
      return it->second;

    std::string name = "func(";
    for (size_t i = 0; i < param_types.size(); i++) {
      name += param_types[i]->typname();
      if (i + 1 < param_types.size())
        name += ", ";
    }
    name += ") -> " + std::string(ret->typname());

    auto obj = new Type(FuncType{pars, ret}, name);

    ctx->m_types.push_back(obj);
    ctx->m_func_pool[key] = obj;
    return obj;
  }

  fun Type::make_Record(qw::context *ctx, std::vector<FieldType> vars, std::vector<FieldType> typs, decls::RecordDecl *decl) -> Type*
  {
    auto obj = new Type(RecordType{vars, typs, decl}, "record");

    ctx->m_types.push_back(obj);
    return obj;
  }

  fun Type::make_Enum(qw::context *ctx, std::vector<FieldCons> vals, std::vector<FieldType> typs) -> Type*
  {
    auto obj = new Type(EnumType{vals, typs, nullptr}, "enum");
    obj->llvm() = llvm::Type::getInt32Ty(*ctx->llvm());
    ctx->m_types.push_back(obj);
    return obj;
  }

  fun Type::make_Set(qw::context *ctx, std::vector<FieldCons> vals, std::vector<FieldType> typs) -> Type*
  {
    auto obj = new Type(SetType{vals, typs, nullptr}, "set");
    obj->llvm() = llvm::Type::getInt64Ty(*ctx->llvm());
    ctx->m_types.push_back(obj);
    return obj;
  }

  fun Type::make_Nick(qw::context *ctx, std::vector<std::string> unresolved) -> Type*
  {
    auto obj = new Type(NickType{unresolved}, "nick");
    ctx->m_types.push_back(obj);
    return obj;
  }

}
