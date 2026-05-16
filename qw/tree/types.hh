/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include <cassert>
#include <llvm/IR/Type.h>
#include <string>
#include <string_view>
#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/control/context.hh"

#define ef else if



namespace qw::types
{

  enum struct TypeEnum: u16
  {
    Pointer   = 0x0'01,
    Reference = 0x0'02,
    CArray    = 0x0'03,

    Nick   = 0x1'01,
    Func   = 0x1'02,
    Record = 0x1'03,

    MemberField = 0x2'01,


    // Primitive
    IntU8   = 0x3'0'01,
    IntU16  = 0x3'0'02,
    IntU32  = 0x3'0'04,
    IntU64  = 0x3'0'08,
    IntU128 = 0x3'0'10,

    IntS8   = 0x3'1'01,
    IntS16  = 0x3'1'02,
    IntS32  = 0x3'1'04,
    IntS64  = 0x3'1'08,
    IntS128 = 0x3'1'10,

    Flo16  = 0x4'02,
    Flo32  = 0x4'04,
    Flo64  = 0x4'08,
    Flo128 = 0x4'10,

    Char = 0x5'000,

    Bool = 0x6'000,

    Ptr = 0x7'000,

    Void = 0x8'000,
  };


  // base
  struct Type: identy
  {
    protected:
      Type(TypeEnum type, identy *parent, word pos);


    public:
      static fun make_Pointer(qw::context *ctx, identy *parent, word pos, types::Type *sub) -> Sub*;
      static fun make_Reference(qw::context *ctx, identy *parent, word pos, types::Type *sub) -> Sub*;
      static fun make_CArray(qw::context *ctx, identy *parent, word pos, types::Type *sub) -> Sub*;
      static fun make_Nick(qw::context *ctx, identy *parent, word pos, std::string_view Name) -> Nick*;
      static fun make_Func(qw::context *ctx, identy *parent, word pos, std::vector<MemberField*> pars, types::Type *ret) -> Func*;
      static fun make_Record(qw::context *ctx, identy *parent, word pos, std::vector<MemberField*> vars, std::vector<decls::Type*> types, std::vector<decls::Func*> funcs) -> Record*;

      static fun make_MemberField(qw::context *ctx, Type *parent, word pos, std::string_view name, Type *targetType) -> MemberField*;

      fun dis() -> void;
      

    public:
      #pragma region SystemTypes
      static fun make_intU8(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intU16(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intU32(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intU64(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intU128(qw::context *ctx, identy *parent) -> Type*;

      static fun make_intS8(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intS16(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intS32(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intS64(qw::context *ctx, identy *parent) -> Type*;
      static fun make_intS128(qw::context *ctx, identy *parent) -> Type*;
      
      static fun make_float16(qw::context *ctx, identy *parent) -> Type*;
      static fun make_float32(qw::context *ctx, identy *parent) -> Type*;
      static fun make_float64(qw::context *ctx, identy *parent) -> Type*;
      static fun make_float128(qw::context *ctx, identy *parent) -> Type*;
      
      static fun make_char(qw::context *ctx, identy *parent) -> Type*;
      
      static fun make_bool(qw::context *ctx, identy *parent) -> Type*;
      
      static fun make_void(qw::context *ctx, identy *parent) -> Type*;

      static fun make_ptr(qw::context *ctx, identy *parent) -> Type*;
      #pragma endregion


    public:
      inline fun isChar() { return m_subType == TypeEnum::Char; }
      inline fun isBool() { return m_subType == TypeEnum::Bool; }

      inline fun isInteger() { return ((u16)m_subType & 0xF000) == 0x3'0'00; }
      inline fun isSigned() { assert(isInteger()); return ((u16)m_subType & 0xF00) == 0x1'00; }
      inline fun intBit() { assert(isInteger()); return ((u16)m_subType & 0xFF); }
      
      inline fun isReference() { return m_subType == TypeEnum::Reference; }

    private:
      TypeEnum m_subType;
      llvm::Type *m_llvm{};
      

    public:
      inline fun& llvm() { return m_llvm; }
      inline fun subType() { return m_subType; }
  };



  // types
  struct Sub: Type
  {
    friend struct Type;

    protected:
      inline Sub(TypeEnum type, identy *parent, word pos, Type *sub): Type(type, parent, pos), m_sub(sub) {}


    private:
      Type *m_sub{};

    public:
      inline fun& sub() { return m_sub; }
  };

  struct Nick: Type
  {
    friend struct Type;

    protected:
      inline Nick(identy *parent, word pos, std::string_view name): Type(TypeEnum::Nick, parent, pos), m_unResolvedName({(std::string)name}) {}


    private:
      std::vector<std::string> m_unResolvedName;

    public:
      inline fun& unResolvedName() { return m_unResolvedName; }
  };

  struct Func: Type
  {
    friend struct Type;

    protected:
      inline Func(identy *parent, word pos, std::vector<MemberField*> pars, types::Type *ret): Type(TypeEnum::Func, parent, pos), m_pars(pars), m_ret(ret) {}


    private:
      std::vector<MemberField*> m_pars{};
      types::Type *m_ret{};

    public:
      inline fun& pars() { return m_pars; }
      inline fun& ret() { return m_ret; }
  };

  struct Record: Type
  {
    friend struct Type;

    protected:
      inline Record(identy *parent, word pos, std::vector<MemberField*> vars, std::vector<decls::Type*> types, std::vector<decls::Func*> funcs)
        : Type(TypeEnum::Record, parent, pos)
        , m_vars(vars)
        //, m_types(types)
        //, m_funcs(funcs)
      {}

    private:
      std::vector<MemberField*>  m_vars;
      //vector<decls::TypeDecl*> m_types;
      //vector<decls::qFuncDecl*> m_funcs;

    public:
      inline fun& vars() { return m_vars; }
      //inline fun& types() { return m_types; }
      //inline fun& funcs() { return m_funcs; }

    public:
      //optional<bool> attr_packed{nullopt};
      //optional<bool> attr_sorted{nullopt};
  };



  // members
  struct Member: Type
  {
    protected:
      inline Member(TypeEnum type, identy *parent, word pos, std::string_view name): Type(type, parent, pos), m_name(name) {}


    private:
      std::string m_name;

    public:
      inline fun name() { return m_name; }
  };

  struct MemberField: Member
  {
    friend struct Type;

    protected:
      inline MemberField(identy *parent, word pos, std::string_view name, Type *targetType)
        : Member(TypeEnum::MemberField, parent, pos, name)
        , m_targetType(targetType)
      {}


    private:
      Type *m_targetType{};

    public:
      inline fun& targetType() { return m_targetType; }
  };

}
