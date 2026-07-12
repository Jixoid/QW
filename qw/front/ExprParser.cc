/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/front/front.hh"
#include "qw/basis.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/lexer/lexer.hh"
#include "qw/pretype.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/types.hh"
#include <expected>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if

#define Require(X) \
  if (!X) \
    return fatals::FileEndedButContextNotFinished();

#define Require_Word(X, C) \
  { \
    Require(X); \
    if (lexer.kind(X.view()[0]) != CharKind::Word) { \
      auto E = errors::ExpectedAWord(X, X.str()); \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
      C; \
    } \
  }

#define expected(LEX, T) \
  { \
    auto X = LEX; \
    Require(X) ef(X.view() != T) return errors::ExpectedIdentifierBut(X, X.str(), T); \
  }

#define expected2(LEX, T, T2) \
  { \
    auto X = LEX; \
    Require(X) ef(X.view() != T || X.view() != T2) return errors::ExpectedIdentifierBut2(X, X.str(), T, T2); \
  }

#define if_error(X) \
  { \
    auto E = X; \
    if (!E.has_value()) { \
      if (E.error()->type() == qw::diagnostic::MsgType::Fatal) \
        return std::unexpected(std::move(E.error())); \
      else { \
        sum.add(E.error().get()); \
        std::cerr << E.error(); \
      } \
    } \
  }

#define if_except(X) \
  { \
    auto E = X; \
    if (!E.has_value()) \
      return std::unexpected(std::move(E.error())); \
  }

#define if_except_ref(X) \
  { \
    auto &E = X; \
    if (!E.has_value()) \
      return std::unexpected(std::move(E.error())); \
  }

#define print(E) \
  { \
    if (E.error()->type() == qw::diagnostic::MsgType::Fatal) \
      return std::unexpected(std::move(E.error())); \
    else { \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
    } \
  }

#define val_error(X) \
  { \
    if (!X.has_value()) \
      return std::unexpected(std::move(X.error())); \
  }



namespace qw
{

  fun ExprParser::read_Expr(identy *parent, Precedence min_prec) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
    exprs::Expr *ret{};
    auto ID = lex();
    Require(ID);

    if (ID.is(WordKind::At) || ID.is(WordKind::Add) || ID.is(WordKind::Sub) || ID.is(WordKind::Bang) || ID.is(WordKind::Tilde)) {
      exprs::UnaryOpEnum kind;
      if (ID.is(WordKind::At))    kind = exprs::UnaryOpEnum::AddrOf;
      ef (ID.is(WordKind::Add))   kind = exprs::UnaryOpEnum::Plus;
      ef (ID.is(WordKind::Sub))   kind = exprs::UnaryOpEnum::Minus;
      ef (ID.is(WordKind::Bang))  kind = exprs::UnaryOpEnum::LNot;
      ef (ID.is(WordKind::Tilde)) kind = exprs::UnaryOpEnum::BitNot;

      auto sub = read_Expr(parent, Precedence::Unary);
      val_error(sub);
      ret = exprs::Expr::make_UnaryOp(ctx, parent, kind, *sub, ID);
    }
    ef (ID.view() == "true" || ID.view() == "false") {
      ret = exprs::Expr::make_BoolLiteral(ctx, parent, ID.view() == "true", ID);
    }
    ef (ID.view() == "null") {
      ret = exprs::Expr::make_PtrLiteral(ctx, parent, 0, ID);
    }
    ef (ID.is(WordKind::Word) && lex.kind(ID.view()[0]) == CharKind::Numeral) {
      auto val = lex.asInteger();
      if_except_ref(val);

      ret = exprs::Expr::make_IntegerLiteral(ctx, parent, *val, ID);
    }
    ef (ID.is(WordKind::String) && ID.view()[0] == '\'') {
      std::string unescaped = lex.asString();

      if (unescaped.size() == 0) return errors::EmptyCharacterConstant(ID);
      if (unescaped.size() > 1)  return errors::CharacterConstantTooLong(ID, (std::string)ID.view().substr(1, ID.view().size() - 2));

      ret = exprs::Expr::make_CharLiteral(ctx, parent, unescaped[0], ID);
    }
    ef (ID.is(WordKind::String) && ID.view()[0] == '\"') {
      ret = exprs::Expr::make_StringLiteral(
        ctx, parent, 
        lex.asString(),
        ID
      );
    }
    ef (ID.is(WordKind::Word)) {
      ret = exprs::Expr::make_Nick(ctx, parent, {ID.str()}, ID);
    }
    ef (ID.is(WordKind::ParenBeg)) {
      auto expr = read_Expr(parent, Precedence::Lowest);
      val_error(expr);
      ret = *expr;
      expected(lex(), ")");
    }
    else
      diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());

    
    auto p_ret = read_Expr_Postfix(parent, ret);
    val_error(p_ret);
    ret = *p_ret;

    auto i_ret = read_Expr_Infix(parent, min_prec, ret);
    val_error(i_ret);
    return *i_ret;
  }

  
  fun ExprParser::read_Expr_Postfix(identy *parent, exprs::Expr *ret) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
    
    while (true) {
      auto Op = lex();
      if (!Op)
        break;

      if (Op.is(WordKind::Dot) || Op.is(WordKind::Scope)) {
        auto MemName = lex();
        Require(MemName);

        auto kind = Op.is(WordKind::Dot) ? exprs::MemberOpEnum::Member : exprs::MemberOpEnum::NameS;
        
        if (kind == exprs::MemberOpEnum::NameS && ret->is<exprs::NickExpr>()) {
          ret->as<exprs::NickExpr>()->unresolved.push_back(MemName.str());
          continue;
        }

        auto MemExpr = exprs::Expr::make_Nick(ctx, parent, { MemName.str() }, MemName);
        ret = exprs::Expr::make_MemberOp(ctx, parent, kind, ret, MemExpr, Op);
      }
      ef (Op.is(WordKind::ParenBeg) || Op.is(WordKind::SquareBracketBeg)) {
        auto kind = Op.is(WordKind::ParenBeg) ? exprs::PostfixOpEnum::Call : exprs::PostfixOpEnum::Array;
        auto clsk = Op.is(WordKind::ParenBeg) ? WordKind::ParenEnd : WordKind::SquareBracketEnd;

        std::vector<exprs::Expr *> ops;
        auto Next = lex();
        Require(Next);

        while (!Next.is(clsk)) {
          lex.store(Next);
          auto ex = read_Expr(parent, Precedence::Lowest);
          if_except_ref(ex);
          ops.push_back(*ex);

          Next = lex();
          Require(Next);
          if (Next.is(WordKind::Comma)) {
            Next = lex();
            Require(Next);
          }
        }
        ret = exprs::Expr::make_PostfixOp(ctx, parent, kind, ret, ops, Next);
      }
      ef (Op.is(WordKind::Question)) {
        ret = exprs::Expr::make_PostfixOp(ctx, parent, exprs::PostfixOpEnum::Deref, ret, {}, Op);
      }
      ef (Op.is(WordKind::AngleBeg)) {
        u0 old_off = lex.offset();
        
        std::vector<types::Type *> generic_args;
        bool success = true;
        
        while (true) {
          auto type = pctx.type->read_Type(parent, true);
          if (!type) {
            success = false;
            break;
          }
          generic_args.push_back(*type);
          
          auto Next = lex();
          if (!Next) {
            success = false;
            break;
          }
          
          if (Next.is(WordKind::AngleEnd)) break;
          ef (Next.is(WordKind::Comma)) continue;
          else {
            success = false;
            break;
          }
        }
        
        if (success)
          ret = exprs::Expr::make_GenericOp(ctx, parent, ret, std::move(generic_args), Op);
        else {
          lex.offset() = old_off;
          lex.store(Op);
          break;
        }
      }
      else {
        lex.store(Op);
        break;
      }
    }
    return ret;
  }

  fun ExprParser::read_Expr_Infix(identy *parent, Precedence min_prec, exprs::Expr *ret) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
    
    while (true) {
      auto Op = lex();
      if (!Op)
        break;

      Precedence op_prec = Precedence::Lowest;
      exprs::BinaryOpEnum kind;

      if (Op.is(WordKind::Assign)) {
        op_prec = Precedence::Assign;
        kind    = exprs::BinaryOpEnum::Assign;
      }
      ef (Op.is(WordKind::AssignmentAdd) || Op.is(WordKind::AssignmentSub) || Op.is(WordKind::AssignmentMul) || Op.is(WordKind::AssignmentDiv) || Op.is(WordKind::AssignmentRem) || Op.is(WordKind::AssignmentBitwiseAnd) || Op.is(WordKind::AssignmentBitwiseOr) || Op.is(WordKind::AssignmentBitwiseXor) || Op.is(WordKind::AssignmentLeftShift) || Op.is(WordKind::AssignmentRighShift)) {
        op_prec = Precedence::Assign;
        if (Op.is(WordKind::AssignmentAdd))         kind = exprs::BinaryOpEnum::AddAssign;
        ef (Op.is(WordKind::AssignmentSub))         kind = exprs::BinaryOpEnum::SubAssign;
        ef (Op.is(WordKind::AssignmentMul))         kind = exprs::BinaryOpEnum::MulAssign;
        ef (Op.is(WordKind::AssignmentDiv))         kind = exprs::BinaryOpEnum::DivAssign;
        ef (Op.is(WordKind::AssignmentRem))         kind = exprs::BinaryOpEnum::RemAssign;
        ef (Op.is(WordKind::AssignmentBitwiseAnd))  kind = exprs::BinaryOpEnum::BitAndAssign;
        ef (Op.is(WordKind::AssignmentBitwiseOr))   kind = exprs::BinaryOpEnum::BitOrAssign;
        ef (Op.is(WordKind::AssignmentBitwiseXor))  kind = exprs::BinaryOpEnum::BitXorAssign;
        ef (Op.is(WordKind::AssignmentLeftShift))    kind = exprs::BinaryOpEnum::ShlAssign;
        ef (Op.is(WordKind::AssignmentRighShift))    kind = exprs::BinaryOpEnum::ShrAssign;
      }
      ef (Op.is(WordKind::Equal) || Op.is(WordKind::NotEqual)) {
        op_prec = Precedence::Eq;
        kind    = Op.is(WordKind::Equal) ? exprs::BinaryOpEnum::Eq : exprs::BinaryOpEnum::NEq;
      }
      ef (Op.is(WordKind::AngleBeg) || Op.is(WordKind::AngleEnd) || Op.is(WordKind::SmallerEqual) || Op.is(WordKind::BiggerEqual)) {
        op_prec = Precedence::Rel;
        if (Op.is(WordKind::AngleBeg))      kind = exprs::BinaryOpEnum::Lt;
        ef (Op.is(WordKind::AngleEnd))      kind = exprs::BinaryOpEnum::Gt;
        ef (Op.is(WordKind::SmallerEqual))  kind = exprs::BinaryOpEnum::LEq;
        ef (Op.is(WordKind::BiggerEqual))   kind = exprs::BinaryOpEnum::GEq;
      }
      ef (Op.is(WordKind::Add) || Op.is(WordKind::Sub)) {
        op_prec = Precedence::Add;
        kind    = Op.is(WordKind::Add) ? exprs::BinaryOpEnum::Add : exprs::BinaryOpEnum::Sub;
      }
      ef (Op.is(WordKind::Mul) || Op.is(WordKind::Div) || Op.is(WordKind::Rem)) {
        op_prec = Precedence::Mul;
        kind    = Op.is(WordKind::Mul) ? exprs::BinaryOpEnum::Mul : Op.is(WordKind::Div) ? exprs::BinaryOpEnum::Div : exprs::BinaryOpEnum::Rem;
      }
      ef (Op.is(WordKind::BitwiseAnd)) {
        op_prec = Precedence::BitAnd;
        kind    = exprs::BinaryOpEnum::BitAnd;
      }
      ef (Op.is(WordKind::BitwiseOr)) {
        op_prec = Precedence::BitOr;
        kind    = exprs::BinaryOpEnum::BitOr;
      }
      ef (Op.is(WordKind::BitwiseXor)) {
        op_prec = Precedence::BitXor;
        kind    = exprs::BinaryOpEnum::BitXor;
      }
      ef (Op.is(WordKind::ShiftLeft) || Op.is(WordKind::ShiftRigh)) {
        op_prec = Precedence::Shift;
        kind    = Op.is(WordKind::ShiftLeft) ? exprs::BinaryOpEnum::Shl : exprs::BinaryOpEnum::LShr;
      }
      ef (Op.is(WordKind::LogicalAnd)) {
        op_prec = Precedence::LogAnd;
        kind    = exprs::BinaryOpEnum::LogAnd;
      }
      ef (Op.is(WordKind::LogicalOr)) {
        op_prec = Precedence::LogOr;
        kind    = exprs::BinaryOpEnum::LogOr;
      }
      else {
        lex.store(Op);
        break;
      }

      if (op_prec < min_prec) {
        lex.store(Op);
        break;
      }

      Precedence next_prec = (kind == exprs::BinaryOpEnum::Assign) ? op_prec : op_prec + 1;
      auto r2              = read_Expr(parent, next_prec);
      val_error(r2);

      ret = exprs::Expr::make_BinaryOp(ctx, parent, kind, ret, *r2, Op);
    }

    return ret;
  }

}
