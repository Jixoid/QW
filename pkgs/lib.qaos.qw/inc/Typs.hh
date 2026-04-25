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
#include "Basis.hh"
#include "Context.hh"
#include "Types.hh"

#define ef else if

using namespace std;



namespace qw::types
{

  // base
  struct qType: qIdentifier
  {
    public:
      enum qEnum: u16
      {
        qte_Unknown = 0,

        qte_Pointer   = 0x0'01,
        qte_Reference = 0x0'02,
        qte_CArray    = 0x0'03,

        qte_Nick   = 0x1'01,
        qte_Func   = 0x1'02,
        qte_Record = 0x1'03,

        qte_MemberField = 0x2'01,


        // Primitive
        qte_IntU8   = 0x3'0'01,
        qte_IntU16  = 0x3'0'02,
        qte_IntU32  = 0x3'0'04,
        qte_IntU64  = 0x3'0'08,
        qte_IntU128 = 0x3'0'10,

        qte_IntS8   = 0x3'1'01,
        qte_IntS16  = 0x3'1'02,
        qte_IntS32  = 0x3'1'04,
        qte_IntS64  = 0x3'1'08,
        qte_IntS128 = 0x3'1'10,

        qte_Flo16  = 0x4'02,
        qte_Flo32  = 0x4'04,
        qte_Flo64  = 0x4'08,
        qte_Flo128 = 0x4'10,

        qte_Char = 0x5'000,

        qte_Bool = 0x6'000,

        qte_Ptr = 0x7'000,

        qte_Void = 0x8'000,
      };
      
    protected:
      qType(qEnum type, qIdentifier *parent, word pos);


    public:
      static fun make_Pointer(qw::context *ctx, qIdentifier *parent, word pos, types::qType *sub) -> qSubType*;
      static fun make_Reference(qw::context *ctx, qIdentifier *parent, word pos, types::qType *sub) -> qSubType*;
      static fun make_CArray(qw::context *ctx, qIdentifier *parent, word pos, types::qType *sub) -> qSubType*;
      static fun make_Nick(qw::context *ctx, qIdentifier *parent, word pos, string Name) -> qNickType*;
      static fun make_Func(qw::context *ctx, qIdentifier *parent, word pos, vector<qMemberFieldType*> pars, types::qType *ret) -> qFuncType*;
      static fun make_Record(qw::context *ctx, qIdentifier *parent, word pos, vector<qMemberFieldType*> vars, vector<decls::qTypeDecl*> types, vector<decls::qFuncDecl*> funcs) -> qRecordType*;

      static fun make_MemberField(qw::context *ctx, qType *parent, word pos, string name, qType *targetType) -> qMemberFieldType*;

      fun dis() -> void;
      

    public:
      #pragma region SystemTypes
      static fun make_intU8(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intU16(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intU32(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intU64(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intU128(qw::context *ctx, qIdentifier *parent) -> qType*;

      static fun make_intS8(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intS16(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intS32(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intS64(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_intS128(qw::context *ctx, qIdentifier *parent) -> qType*;
      
      static fun make_float16(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_float32(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_float64(qw::context *ctx, qIdentifier *parent) -> qType*;
      static fun make_float128(qw::context *ctx, qIdentifier *parent) -> qType*;
      
      static fun make_char(qw::context *ctx, qIdentifier *parent) -> qType*;
      
      static fun make_bool(qw::context *ctx, qIdentifier *parent) -> qType*;
      
      static fun make_void(qw::context *ctx, qIdentifier *parent) -> qType*;

      static fun make_ptr(qw::context *ctx, qIdentifier *parent) -> qType*;
      #pragma endregion


    public:
      inline fun isChar() { return m_subType == qte_Char; }
      inline fun isBool() { return m_subType == qte_Bool; }

      inline fun isInteger() { return (m_subType & 0xF000) == 0x3'0'00; }
      inline fun isSigned() { assert(isInteger()); return (m_subType & 0xF00) == 0x1'00; }
      inline fun intBit() { assert(isInteger()); return (m_subType & 0xFF); }
      
      inline fun isReference() { return m_subType == qte_Reference; }

    private:
      qEnum m_subType{qte_Unknown};
      llvm::Type *m_llvm{};
      

    public:
      inline fun& llvm() { return m_llvm; }
      inline fun subType() { return m_subType; }
  };



  // types
  struct qSubType: qType
  {
    friend class qType;

    protected:
      inline qSubType(qEnum type, qIdentifier *parent, word pos, qType *sub): qType(type, parent, pos), m_sub(sub) {}


    private:
      qType *m_sub{};

    public:
      inline fun& sub() { return m_sub; }
  };

  struct qNickType: qType
  {
    friend class qType;

    protected:
      inline qNickType(qIdentifier *parent, word pos, string name): qType(qte_Nick, parent, pos), m_unResolvedName({name}) {}


    private:
      vector<string> m_unResolvedName;

    public:
      inline fun& unResolvedName() { return m_unResolvedName; }
  };

  struct qFuncType: qType
  {
    friend class qType;

    protected:
      inline qFuncType(qIdentifier *parent, word pos, vector<qMemberFieldType*> pars, types::qType *ret): qType(qte_Func, parent, pos), m_pars(pars), m_ret(ret) {}


    private:
      vector<qMemberFieldType*> m_pars{};
      types::qType *m_ret{};

    public:
      inline fun& pars() { return m_pars; }
      inline fun& ret() { return m_ret; }
  };

  struct qRecordType: qType
  {
    friend class qType;

    protected:
      inline qRecordType(qIdentifier *parent, word pos, vector<qMemberFieldType*> vars, vector<decls::qTypeDecl*> types, vector<decls::qFuncDecl*> funcs)
        : qType(qte_Record, parent, pos)
        , m_vars(vars)
        //, m_types(types)
        //, m_funcs(funcs)
      {}

    private:
      vector<qMemberFieldType*>  m_vars;
      //vector<decls::qTypeDecl*> m_types;
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
  struct qMemberType: qType
  {
    protected:
      inline qMemberType(qEnum type, qIdentifier *parent, word pos, string name): qType(type, parent, pos), m_name(name) {}


    private:
      string m_name;

    public:
      inline fun name() { return m_name; }
  };

  struct qMemberFieldType: qMemberType
  {
    friend class qType;

    protected:
      inline qMemberFieldType(qIdentifier *parent, word pos, string name, qType *targetType)
        : qMemberType(qte_MemberField, parent, pos, name)
        , m_targetType(targetType)
      {}


    private:
      qType *m_targetType{};

    public:
      inline fun& targetType() { return m_targetType; }
  };

}
