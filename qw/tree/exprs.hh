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
#include <variant>

#define ef else if



namespace qw::exprs
{

  enum struct UnaryOpEnum: u8
  {
    Plus /* "+" */,
    Minus /* "-" */,
    LNot /* "!" */,
    BitNot /* "~" */,
    AddrOf /* "@" */,
  };

  enum struct BinaryOpEnum: u8
  {
    Assign /* "=" */,
    Eq /* "==" */,
    NEq /* "!=" */,
    Lt /* "<" */,
    Gt /* ">" */,
    LEq /* "<=" */,
    GEq /* ">=" */,
    BitAnd /* "&" */,
    BitOr /* "|" */,
    BitXor /* "^" */,
    Shl /* "<<" */,
    LShr /* unsigned ">>" */,
    AShr /* signed ">>" */,
    LRot /* "<<|" */,
    RRot /* "|>>" */,
    LogAnd /* "&&" */,
    LogOr /* "||" */,
    LogXor /* "^^" */,
    Add /* "+" */,
    Sub /* "-" */,
    Mul /* "*" */,
    Div /* "/" */,
    Rem /* "%" */,
    RangeEq /* ".." */,
    AddAssign /* "+=" */,
    SubAssign /* "-=" */,
    MulAssign /* "*=" */,
    DivAssign /* "/=" */,
    RemAssign /* "%=" */,
    BitAndAssign /* "&=" */,
    BitOrAssign /* "|=" */,
    BitXorAssign /* "^=" */,
    ShlAssign /* "<<=" */,
    ShrAssign /* ">>=" */,
  };

  enum struct PostfixOpEnum: u8
  {
    Call /* "("  */,
    Array /* "[" */,
    Deref /* "?" */,
  };

  enum struct MemberOpEnum: u8
  {
    Member /* "." */,
    NameS /* "::" */,
  };

  struct IntegerLiteral  { std::variant<u128, i128> val{}; };
  struct FloatingLiteral { f128 val{}; };
  struct CharLiteral     { u8 val{}; };
  struct BoolLiteral     { bool val{}; };
  struct PtrLiteral      { u64 val{}; };
  struct StringLiteral   { std::string val{}; };

  struct UnaryOp   { Expr *o1{}; UnaryOpEnum kind; };
  struct BinaryOp  { Expr *o1{}, *o2{}; BinaryOpEnum kind; types::Type *computationType{}; };
  struct PostfixOp { Expr *obj{}; std::vector<Expr *> operands; PostfixOpEnum kind; };
  struct MemberOp  { Expr *obj{}, *mem{}; MemberOpEnum kind; };

  struct VarExpr { stmts::Stmt *var{}; };
  struct ValExpr {};

  struct NickExpr { std::vector<std::string> unresolved; };

  using ExprVari = std::variant<
    IntegerLiteral, FloatingLiteral, CharLiteral, BoolLiteral, PtrLiteral, StringLiteral,

    UnaryOp, BinaryOp, PostfixOp, MemberOp,

    ValExpr, VarExpr, NickExpr
  >;

  struct Expr: identy
  {
    protected:
      inline Expr(ExprVari vari, identy *parent, word pos)
        : identy(IdentyEnum::Expr, parent, pos)
        , m_vari(vari)
      {}

    public:
      static fun make_IntegerLiteral(qw::context *ctx, identy *parent, u128 val, word pos) -> Expr*;
      static fun make_IntegerLiteral(qw::context *ctx, identy *parent, i128 val, word pos) -> Expr*;
      static fun make_FloatingLiteral(qw::context *ctx, identy *parent, f128 val, word pos) -> Expr*;
      static fun make_CharLiteral(qw::context *ctx, identy *parent, u8 val, word pos) -> Expr*;
      static fun make_BoolLiteral(qw::context *ctx, identy *parent, bool val, word pos) -> Expr*;
      static fun make_PtrLiteral(qw::context *ctx, identy *parent, u64 val, word pos) -> Expr*;
      static fun make_StringLiteral(qw::context *ctx, identy *parent, std::string val, word pos) -> Expr*;

      static fun make_UnaryOp(qw::context *ctx, identy *parent, UnaryOpEnum kind, Expr *o1, word pos) -> Expr*;
      static fun make_BinaryOp(qw::context *ctx, identy *parent, BinaryOpEnum kind, Expr *o1, Expr *o2, word pos) -> Expr*;
      static fun make_PostfixOp(qw::context *ctx, identy *parent, PostfixOpEnum kind, Expr *obj, std::vector<Expr *> operands, word pos) -> Expr*;
      static fun make_MemberOp(qw::context *ctx, identy *parent, MemberOpEnum kind, Expr *obj, Expr *mem, word pos) -> Expr*;

      static fun make_VarExpr(qw::context *ctx, identy *parent, stmts::Stmt *var, word pos) -> Expr*;
      static fun make_ValExpr(qw::context *ctx, identy *parent, types::Type *type, llvm::Value *value, word pos) -> Expr*;
      static fun make_Nick(qw::context *ctx, identy *parent, std::vector<std::string> name, word pos) -> Expr*;

    private:
      types::Type *m_targetType{};
      llvm::Value *m_llvm{};
      ExprVari m_vari;

    public:
      inline fun& targetType() { return m_targetType; }
      inline fun& llvm() { return m_llvm; }
      inline fun  vari() { return m_vari; }

    public:
      template<typename T>
      inline fun is() { return std::holds_alternative<T>(m_vari); }

      template<typename T>
      inline fun as() { return &std::get<T>(m_vari); }
  };

}
