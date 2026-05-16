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
#include <llvm/IR/Constants.h>
#include <llvm/IR/Value.h>
#include <string>
#include <string_view>
#include <variant>

#define ef else if



namespace qw::exprs
{

  enum struct BinaryOpEnum: u8 { Add, Sub, Mul, Div, Rem };


  enum struct ExprEnum: u16 {
    IntegerLiteral  = 1'01,
    FloatingLiteral = 1'02,
    CharLiteral     = 1'03,
    BoolLiteral     = 1'04,
    PtrLiteral      = 1'05,

    ValExpr = 2'01,

    Nick = 3'01,

    BinaryOp  = 4'01,
    PrimaryOp = 4'02,
  };

  struct Expr: identy
  {
    protected:
      inline Expr(ExprEnum type, identy *parent, word pos)
        : identy(IdentyEnum::Expr, parent, pos)
        , m_subType(type)
      {}
      

    public:
      static fun make_IntegerLiteral(qw::context *ctx, identy *parent, u128 val, word pos) -> IntegerLiteral*;
      static fun make_IntegerLiteral(qw::context *ctx, identy *parent, i128 val, word pos) -> IntegerLiteral*;
      static fun make_FloatingLiteral(qw::context *ctx, identy *parent, f128 val, word pos) -> FloatingLiteral*;
      static fun make_CharLiteral(qw::context *ctx, identy *parent, u8 val, word pos) -> CharLiteral*;
      static fun make_BoolLiteral(qw::context *ctx, identy *parent, bool val, word pos) -> BoolLiteral*;
      static fun make_PtrLiteral(qw::context *ctx, identy *parent, u64 val, word pos) -> PtrLiteral*;

      static fun make_ValExpr(qw::context *ctx, identy *parent, types::Type *type, llvm::Value *value, word pos) -> ValExpr*;

      static fun make_Nick(qw::context *ctx, identy *parent, std::string_view name, word pos) -> NickExpr*;

      static fun make_PrimaryOp(qw::context *ctx, identy *parent, exprs::Expr *obj, std::vector<exprs::Expr*> operands, word pos) -> PrimaryOp*;
      static fun make_BinaryOp(qw::context *ctx, identy *parent, BinaryOpEnum kind, exprs::Expr *o1, exprs::Expr *o2, word pos) -> BinaryOp*;

      fun dis() -> void;


    private:
      ExprEnum m_subType;
      types::Type *m_targetType;
      llvm::Value *m_llvm{};

    public:
      inline fun  subType() const { return m_subType; }
      inline fun& targetType() { return m_targetType; }
      inline fun& llvm() { return m_llvm; }
  };
  



  #pragma region Literal

  struct Literal: Expr
  {
    protected:
      inline Literal(ExprEnum type, identy *parent, word pos): Expr(type, parent, pos) {}
  };



  struct IntegerLiteral: Literal
  {
    friend struct Expr;

    protected:
      IntegerLiteral(qw::context *ctx, identy *parent, u128 val, word pos);
      IntegerLiteral(qw::context *ctx, identy *parent, i128 val, word pos);

    private:
      std::variant<u128, i128> m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct FloatingLiteral: Literal
  {
    friend struct Expr;
    
    protected:
      FloatingLiteral(qw::context *ctx, identy *parent, f128 val, word pos);

    private:
      f128 m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct CharLiteral: Literal
  {
    friend struct Expr;

    protected:
      CharLiteral(qw::context *ctx, identy *parent, u8 val, word pos);

    private:
      u8 m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct BoolLiteral: Literal
  {
    friend struct Expr;

    protected:
      BoolLiteral(qw::context *ctx, identy *parent, bool val, word pos);

    private:
      bool m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct PtrLiteral: Literal
  {
    friend struct Expr;

    protected:
      PtrLiteral(qw::context *ctx, identy *parent, u64 val, word pos);

    private:
      u64 m_val{};

    public:
      inline fun val() { return m_val; }
  };

  struct stringLiteral: Literal
  {
    friend struct Expr;

    protected:
      stringLiteral(qw::context *ctx, identy *parent, std::string_view val, word pos);

    public:
      std::string_view m_val{};

    public:
      inline fun val() { return m_val; }
  };

  /*
  struct qVoidLiteral: Literal
  {};
  */

  #pragma endregion




  struct ValExpr: Expr
  {
    friend struct Expr;

    protected:
      inline ValExpr(identy *parent, types::Type *type, llvm::Value *value, word pos)
        : Expr(ExprEnum::ValExpr, parent, pos)
      {
        llvm() = value;
        targetType() = type;
      }

  };

  struct NickExpr: Expr
  {
    friend struct Expr;

    protected:
      inline NickExpr(identy *parent, std::string_view name, word pos): Expr(ExprEnum::Nick, parent, pos), m_unResolvedName({(std::string)name}) {}
    
    
    private:
      std::vector<std::string> m_unResolvedName;

    public:
      inline fun& unResolvedName() { return m_unResolvedName; }
  };



  
  #pragma region Operator

  struct Operator: Expr
  {
    protected:
      inline Operator(ExprEnum type, identy *parent, word pos): Expr(type, parent, pos) {}
  };



  struct PrimaryOp: Operator
  {
    friend struct Expr;

    protected:
      inline PrimaryOp(identy *parent, exprs::Expr *obj, std::vector<exprs::Expr*> operands, word pos)
        : Operator(ExprEnum::PrimaryOp, parent, pos)
        , m_obj(obj)
        , m_operands(operands)
      {}


    private:
      exprs::Expr *m_obj{};
      std::vector<exprs::Expr*> m_operands;
    
    public:
      inline fun obj() { return m_obj; }
      inline fun& operands() { return m_operands; }
  };

  struct BinaryOp: Operator
  {
    friend struct Expr;

    protected:
      inline BinaryOp(identy *parent, BinaryOpEnum kind, exprs::Expr *o1, exprs::Expr *o2, word pos)
        : Operator(ExprEnum::BinaryOp, parent, pos)
        , m_o1(o1)
        , m_o2(o2)
        , m_kind(kind)
      {}


    private:
      exprs::Expr *m_o1{}, *m_o2{};
      BinaryOpEnum m_kind;

    public:
      inline fun& o1() { return m_o1; }
      inline fun& o2() { return m_o2; }
      inline fun  kind() { return m_kind; }
  };



  struct MemberOp: Operator
  {
    private:
      sptr<exprs::Expr> obj,mem;
  };

  struct UnaryOp: Operator
  {
    sptr<exprs::Expr> o1;
  };
  
  struct TernaryOp: Operator
  {
    sptr<exprs::Expr> o1,o2,o3;
  };

  #pragma endregion


  
  /*
  #pragma region PrimaryOp

  struct qCallOp: qPrimaryOp // X(...)
  {
    public:
      sptr<Expr> Callee;
  };

  struct qIndexOp: qPrimaryOp // X[...]
  {
    public:
      sptr<Expr> Callee;
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
