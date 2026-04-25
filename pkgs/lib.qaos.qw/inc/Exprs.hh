/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "Types.hh"
#include <llvm/IR/Constants.h>
#include <llvm/IR/Value.h>
#include <variant>

#define ef else if

using namespace std;



namespace qw::exprs
{

  enum qBinaryOpEnum: u16 {
    boe_Unknown = 0,

    boe_Add = 1,
    boe_Sub = 2,
    boe_Mul = 3,
    boe_Div = 4,
    boe_Rem = 5,
  };



  struct qExpr: qIdentifier
  {
    public:
      enum qEnum: u16 {
        qee_Unknown = 0,

        qee_IntegerLiteral  = 1'01,
        qee_FloatingLiteral = 1'02,
        qee_CharLiteral     = 1'03,
        qee_BoolLiteral     = 1'04,
        qee_PtrLiteral      = 1'05,

        qee_ValExpr = 2'01,

        qee_Nick = 3'01,

        qee_BinaryOp  = 4'01,
        qee_PrimaryOp = 4'02,
      };

    protected:
      inline qExpr(qEnum type, qIdentifier *parent, word pos)
        : qIdentifier(qIdentifier::qEnum::qie_Expr, parent, pos)
        , m_subType(type)
      {}
      

    public:
      static fun make_IntegerLiteral(qw::context *ctx, qIdentifier *parent, u128 val, word pos) -> qIntegerLiteral*;
      static fun make_IntegerLiteral(qw::context *ctx, qIdentifier *parent, i128 val, word pos) -> qIntegerLiteral*;
      static fun make_FloatingLiteral(qw::context *ctx, qIdentifier *parent, f128 val, word pos) -> qFloatingLiteral*;
      static fun make_CharLiteral(qw::context *ctx, qIdentifier *parent, u8 val, word pos) -> qCharLiteral*;
      static fun make_BoolLiteral(qw::context *ctx, qIdentifier *parent, bool val, word pos) -> qBoolLiteral*;
      static fun make_PtrLiteral(qw::context *ctx, qIdentifier *parent, u64 val, word pos) -> qPtrLiteral*;

      static fun make_ValExpr(qw::context *ctx, qIdentifier *parent, types::qType *type, llvm::Value *value, word pos) -> qValExpr*;

      static fun make_Nick(qw::context *ctx, qIdentifier *parent, string name, word pos) -> qNickExpr*;

      static fun make_PrimaryOp(qw::context *ctx, qIdentifier *parent, exprs::qExpr *obj, vector<exprs::qExpr*> operands, word pos) -> qPrimaryOp*;
      static fun make_BinaryOp(qw::context *ctx, qIdentifier *parent, qBinaryOpEnum kind, exprs::qExpr *o1, exprs::qExpr *o2, word pos) -> qBinaryOp*;

      fun dis() -> void;


    private:
      qEnum m_subType{qee_Unknown};
      types::qType *m_targetType;
      llvm::Value *m_llvm{};

    public:
      inline fun  subType() const { return m_subType; }
      inline fun& targetType() { return m_targetType; }
      inline fun& llvm() { return m_llvm; }
  };
  



  #pragma region Literal

  struct qLiteral: qExpr
  {
    protected:
      inline qLiteral(qEnum type, qIdentifier *parent, word pos): qExpr(type, parent, pos) {}
  };



  struct qIntegerLiteral: qLiteral
  {
    friend class qExpr;

    protected:
      qIntegerLiteral(qw::context *ctx, qIdentifier *parent, u128 val, word pos);
      qIntegerLiteral(qw::context *ctx, qIdentifier *parent, i128 val, word pos);

    private:
      variant<u128, i128> m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct qFloatingLiteral: qLiteral
  {
    friend class qExpr;
    
    protected:
      qFloatingLiteral(qw::context *ctx, qIdentifier *parent, f128 val, word pos);

    private:
      f128 m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct qCharLiteral: qLiteral
  {
    friend class qExpr;

    protected:
      qCharLiteral(qw::context *ctx, qIdentifier *parent, u8 val, word pos);

    private:
      u8 m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct qBoolLiteral: qLiteral
  {
    friend class qExpr;

    protected:
      qBoolLiteral(qw::context *ctx, qIdentifier *parent, bool val, word pos);

    private:
      bool m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct qPtrLiteral: qLiteral
  {
    friend class qExpr;

    protected:
      qPtrLiteral(qw::context *ctx, qIdentifier *parent, u64 val, word pos);

    private:
      u64 m_val{};

    public:
      inline fun val() { return m_val; }
  };

  /*
  struct qStringLiteral: qLiteral
  {
    public:
      string Value{};
  };

  struct qVoidLiteral: qLiteral
  {};
  */

  #pragma endregion




  struct qValExpr: qExpr
  {
    friend class qExpr;

    protected:
      inline qValExpr(qIdentifier *parent, types::qType *type, llvm::Value *value, word pos)
        : qExpr(qee_ValExpr, parent, pos)
      {
        llvm() = value;
        targetType() = type;
      }

  };

  struct qNickExpr: qExpr
  {
    friend class qExpr;

    protected:
      inline qNickExpr(qIdentifier *parent, string name, word pos): qExpr(qee_Nick, parent, pos), m_unResolvedName({name}) {}
    
    
    private:
      vector<string> m_unResolvedName;

    public:
      inline fun& unResolvedName() { return m_unResolvedName; }
  };



  
  #pragma region Operator

  struct qOperator: qExpr
  {
    protected:
      inline qOperator(qEnum type, qIdentifier *parent, word pos): qExpr(type, parent, pos) {}
  };



  struct qPrimaryOp: qOperator
  {
    friend class qExpr;

    protected:
      inline qPrimaryOp(qIdentifier *parent, exprs::qExpr *obj, vector<exprs::qExpr*> operands, word pos)
        : qOperator(qee_PrimaryOp, parent, pos)
        , m_obj(obj)
        , m_operands(operands)
      {}


    private:
      exprs::qExpr *m_obj{};
      vector<exprs::qExpr*> m_operands;
    
    public:
      inline fun obj() { return m_obj; }
      inline fun& operands() { return m_operands; }
  };

  struct qBinaryOp: qOperator
  {
    friend class qExpr;

    protected:
      inline qBinaryOp(qIdentifier *parent, qBinaryOpEnum kind, exprs::qExpr *o1, exprs::qExpr *o2, word pos)
        : qOperator(qee_BinaryOp, parent, pos)
        , m_o1(o1)
        , m_o2(o2)
        , m_kind(kind)
      {}


    private:
      exprs::qExpr *m_o1{}, *m_o2{};
      qBinaryOpEnum m_kind{boe_Unknown};

    public:
      inline fun& o1() { return m_o1; }
      inline fun& o2() { return m_o2; }
      inline fun  kind() { return m_kind; }
  };



  struct qMemberOp: qOperator
  {
    private:
      sptr<exprs::qExpr> obj,mem;
  };

  struct qUnaryOp: qOperator
  {
    sptr<exprs::qExpr> o1;
  };
  
  struct qTernaryOp: qOperator
  {
    sptr<exprs::qExpr> o1,o2,o3;
  };

  #pragma endregion


  
  /*
  #pragma region PrimaryOp

  struct qCallOp: qPrimaryOp // X(...)
  {
    public:
      sptr<qExpr> Callee;
  };

  struct qIndexOp: qPrimaryOp // X[...]
  {
    public:
      sptr<qExpr> Callee;
  };

  #pragma endregion




  #pragma region MemberOp
  
  struct qLocalMemberOp: qMemberOp{}; // X.Y

  struct qReferenceMemberOp: qMemberOp{}; // X->Y

  struct qStaticMemberOp: qMemberOp{}; // X::Y

  #pragma endregion




  #pragma region UnaryOp
  
  struct qPozitiveOp: qUnaryOp{}; // +X
  struct qNegativeOp: qUnaryOp{}; // -X

  struct qIncrementOp: qUnaryOp{}; // ++X
  struct qDecrementOp: qUnaryOp{}; // --X

  struct qLogicNotOp: qUnaryOp{}; // !X
  struct qBitwiseNotOp: qUnaryOp{}; // ~X
  
  struct qAddressOp: qUnaryOp{}; // &X

  #pragma endregion
  */

}
