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

#define if_error_ref(X) \
  { \
    auto &E = X; \
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


namespace qw
{

  fun frontend::read_File(decls::Decl *self) -> std::expected<void, uptr<diagnostic::message>> {
    while (true) {
      auto &ID = lexer();

      if (!ID) return {};
      ef (ID.is(WordKind::CurlyBracketEnd)) break;
      ef (ID.is(WordKind::CompilerDirective)) {
        if_error(read_FileAttributes(self));
      }
      else {
        lexer.store(ID);
        
        auto attr = pctx.meta->read_Attributes();
        if_error_ref(attr);

        auto vis = pctx.meta->read_Visibility();
        if_error_ref(vis);

        auto decl = pctx.decl->read_Decl(self);
        if_error_ref(decl);
      }
    }
    
    return {};
  }


  fun frontend::read_FileAttributes(decls::Decl *self) -> std::expected<void, uptr<diagnostic::message>> {
    while (true) {
      auto attr_name = lexer();
      Require(attr_name);

      if (attr_name.is(WordKind::SquareBracketEnd)) break;

      decls::Attribute attr;
      attr.name = attr_name.str();

      auto next = lexer();
      Require(next);

      while (!next.is(WordKind::SquareBracketEnd) && !next.is(WordKind::Comma)) {
        attr.value += (attr.value.empty() ? "" : " ") + next.str();
        next = lexer();
        Require(next);
      }

      self->attrs().push_back(attr);

      if (next.is(WordKind::SquareBracketEnd)) break;
    }
    return {};
  }
  
}
