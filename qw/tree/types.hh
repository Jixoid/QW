/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "qw/basis.hh"
#include "qw/control/context.hh"
#include "qw/pretype.hh"
#include <cassert>
#include <llvm/IR/Type.h>
#include <string>
#include <string_view>
#include <type_traits>
#include <variant>

#define ef else if



namespace qw::types
{

  struct FieldType { std::string name{}; Type *type{}; Visibility vis = Visibility::Public; };
  struct FieldCons { std::string cons{}; std::variant<u64, i64> val{}; };

  enum struct PrimitiveEnum: u8
  {
    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F16, F32, F64, F128,
    Bool,
    Char,
    Void,
    Ptr,
    Null,
  };

  struct PrimitiveType { PrimitiveEnum kind; };

  struct PointerType   { Type *sub{}; };
  struct ReferenceType { Type *sub{}; };
  struct ZArrayType    { Type *sub{}; }; // Null-Terminated
  struct PArrayType    { Type *sub{}; u64 size{}; }; // Sized

  struct GenericType { Type *sub{}; std::vector<Type *> fields{}; };

  struct FuncType      { std::vector<FieldType> pars{}; Type *ret{}; };
  struct StructType    { std::vector<Type*> baseTypes{}; std::vector<FieldType> vars{}; std::vector<FieldType> typs{}; decls::Decl *decl{}; std::vector<word> baseTypePos{}; };
  struct EnumType      { std::vector<FieldCons> vals{}; std::vector<FieldType> typs{}; decls::Decl *decl{}; Type *baseType{}; word baseTypePos{}; };
  struct SetType       { std::vector<FieldCons> vals{}; std::vector<FieldType> typs{}; decls::Decl *decl{}; Type *baseType{}; word baseTypePos{}; };
  struct IFaceType     { std::vector<Type*> baseTypes{}; std::vector<FieldType> typs{}; decls::Decl *decl{}; std::vector<word> baseTypePos{}; };
  struct TypeParamType { decls::Decl *decl{}; };
  
  struct NickType   { std::vector<std::string> unresolved; };

  using TypeVari = std::variant<
    PrimitiveType, PointerType, ReferenceType, ZArrayType, PArrayType,

    GenericType,

    FuncType,

    StructType, EnumType, SetType, IFaceType,

    TypeParamType, NickType
  >;

  constexpr auto a = std::is_same_v<int, int>;

  struct Type
  {
    public:
      Type(TypeVari vari, std::string cached_name);

    public:
      static fun make_Primitive(qw::context *ctx, PrimitiveEnum kind) -> Type*;

      static fun make_Pointer(qw::context *ctx, Type *sub) -> Type*;
      static fun make_Reference(qw::context *ctx, Type *sub) -> Type*;
      static fun make_ZArray(qw::context *ctx, Type *sub) -> Type*;
      static fun make_PArray(qw::context *ctx, Type *sub, u32 size) -> Type*;

      static fun make_Func(qw::context *ctx, std::vector<FieldType> pars, Type *ret) -> Type*;
      static fun make_Struct(qw::context *ctx, std::vector<FieldType> vars, std::vector<FieldType> typs, decls::Decl *decl, std::vector<Type*> baseTypes = {}, std::vector<word> baseTypePos = {}, std::string tname = "struct") -> Type*;
      static fun make_Enum(qw::context *ctx, std::vector<FieldCons> vals, std::vector<FieldType> typs, decls::Decl *decl, Type *baseType, word baseTypePos = {}, std::string tname = "enum") -> Type*;
      static fun make_Set(qw::context *ctx, std::vector<FieldCons> vals, std::vector<FieldType> typs, decls::Decl *decl, Type *baseType, word baseTypePos = {}, std::string tname = "set") -> Type*;
      static fun make_IFace(qw::context *ctx, std::vector<FieldType> typs, decls::Decl *decl, std::vector<Type*> baseTypes = {}, std::vector<word> baseTypePos = {}, std::string tname = "iface") -> Type*;

      static fun make_Generic(qw::context *ctx, Type *sub, std::vector<Type*> fields) -> Type*;

      static fun make_Nick(qw::context *ctx, std::vector<std::string> unresolved) -> Type*;
      static fun make_TypeParam(qw::context *ctx, decls::Decl *decl, std::string tname) -> Type*;

    private:
      TypeVari m_vari;
      llvm::Type *m_llvm{};
      std::string m_cached_typname;
      StageStatus m_sema = StageStatus::NotChecked;
      StageStatus m_cgen = StageStatus::NotChecked;

    public:
      identy *owner_ident{};

    public:
      inline fun& llvm() { return m_llvm; }
      inline fun  vari() { return m_vari; }
      fun typname() -> std::string_view;
      inline fun& sema() { return m_sema; }
      inline fun& cgen() { return m_cgen; }

    public:
      template<typename T>
      inline fun is() { return std::holds_alternative<T>(m_vari); }

      template<typename T>
      inline fun as() { return &std::get<T>(m_vari); }

    public:
      fun isInteger() -> bool;
      fun isFloat() -> bool;
      fun isSigned() -> bool;
      fun isUnSigned() -> bool;
      fun isChar() -> bool;
      fun isBool() -> bool;
      fun isReference() -> bool { return is<ReferenceType>(); }
      fun isPointer() -> bool { return is<PointerType>(); }
      fun intBit() -> u8;
  };

}
