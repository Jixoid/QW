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
#include "qw/tree/decls.hh"
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

  fun MetaParser::read_Attributes() -> std::expected<void, uptr<diagnostic::message>> {
    auto ID = lex();
    Require(ID);
    
    if (!ID.is(WordKind::DoubleSquareBracketBeg)) {
      lex.store(ID);
      return {};
    }

    while (true) {
      auto attr_name = lex();
      Require(attr_name);

      if (attr_name.is(WordKind::DoubleSquareBracketEnd)) break;

      decls::Attribute attr;
      attr.name = attr_name.str();
      attr.pos = attr_name;

      auto next = lex();
      Require(next);

      if (next.is(WordKind::Colon)) {
        auto attr_val = lex();
        Require(attr_val);
        attr.value = attr_val.str();

        next = lex();
        Require(next);
      }

      m_attrs.push_back(attr);

      if (next.is(WordKind::DoubleSquareBracketEnd)) break;
      ef (next.is(WordKind::Comma)) continue;
      else
        return errors::ExpectedIdentifierBut2(next, next.str(), "]]", ",");
    }
    return {};
  }

  fun MetaParser::read_Visibility(Visibility *scope) -> std::expected<Visibility, uptr<diagnostic::message>> {
    auto ID = lex();
    Require(ID);

    if (ID.view() == "pub")   return Visibility::Public;
    ef (ID.view() == "priv")  return Visibility::Private;
    ef (ID.view() == "prot")  return Visibility::Protected;
    ef (ID.view() == "crate") return Visibility::Crate;
    ef (ID.view() == "group") return Visibility::Group;
    else {
      lex.store(ID);
      return scope ? *scope : Visibility::Private;
    }
  }

  fun MetaParser::read_FuncParams(identy *parent) -> std::expected<std::vector<types::FieldType>, uptr<diagnostic::message>> {
    std::vector<types::FieldType> pars;

    while (true) {
      auto ID = lex();
      Require(ID);

      if (ID.view() == ")") break;
      else
        lex.store(ID);

      // Name
      std::vector<std::string> Names;
      re:
      auto NameArg = lex();
      Require(NameArg);
      Names.push_back(NameArg.str());

      // Colon
      auto Colon = lex();
      Require(Colon);

      if (Colon.view() == ",") goto re;
      ef (Colon.view() == ":");
      else
        return errors::ExpectedIdentifierBut(Colon, Colon.str(), ":");

      // Type
      auto Type = pctx.type->read_Type(parent, true);
      val_error(Type);

      // End
      auto End = lex();
      Require(End);

      if (End.view() == ";");
      ef (End.view() == ")") lex.store(End);
      else
        return errors::ExpectedIdentifierBut2(End, End.str(), ";", ")");

      // Push
      for (auto &X: Names) pars.push_back({X, *Type});
    }

    return pars;
  }

  fun MetaParser::read_GenericParams(decls::Decl *parent) -> std::expected<decls::GenericContext*, uptr<diagnostic::message>> {
    auto OptAngle = lex();
    if (!OptAngle || OptAngle.view() != "<") {
      if (OptAngle) lex.store(OptAngle);
      return nullptr;
    }
    
    auto ctx_obj = new decls::GenericContext{};
    
    while (true) {
      auto PName = lex();
      Require(PName);
      
      auto param_decl = decls::Decl::make_TypeParam(ctx, parent, PName.str(), PName, Visibility::Public);
      
      ctx_obj->params.push_back(param_decl);
      
      auto CommaOrEnd = lex();
      Require(CommaOrEnd);
      if (CommaOrEnd.view() == ">") break;
      ef (CommaOrEnd.view() == ",") continue;
      else
        return errors::ExpectedIdentifierBut(CommaOrEnd, CommaOrEnd.str(), "> veya ,");
    }
    
    return ctx_obj;
  }

}
