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
#include "qw/diagnostic/msgs.hh"
#include <fcntl.h>
#include <stack>
#include <string_view>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if


namespace qw
{

  enum struct CharKind: u8 {
    Ignored = 0x1,
    Symbol  = 0x2,
    Numeral = 0x4,
    String  = 0x8,
    Word    = 0x10,
  };

  inline fun operator |(CharKind a, CharKind b) -> CharKind {
    using utyp = std::underlying_type<CharKind>::type;

    return (CharKind)( (utyp)(a) | (utyp)(b) );
  }

  inline fun operator &(CharKind a, CharKind b) -> std::underlying_type<CharKind>::type {
    using utyp = std::underlying_type<CharKind>::type;

    return (utyp)(a) & (utyp)(b);
  }


  enum struct WordKind: u32 {
    // Bases
    String,
    Word,


    // Shift
    AssignmentLeftShift, // <<=
    AssignmentRighShift, // >>=

    ShiftLeft, // <<
    ShiftRigh, // >>

    // Arithmetic
    AssignmentAdd, // +=
    AssignmentSub, // -=
    AssignmentMul, // *=
    AssignmentDiv, // /=
    AssignmentRem, // %=

    // Logical
    AssignmentLogicalAnd, // &&=
    AssignmentLogicalXor, // ||=
    AssignmentLogicalOr,  // ^^=
    
    LogicalAnd, // &&
    LogicalXor, // ||
    LogicalOr,  // ^^

    // Bitwise
    AssignmentBitwiseAnd, // &=
    AssignmentBitwiseXor, // |=
    AssignmentBitwiseOr,  // ^=
    
    BitwiseAnd, // &
    BitwiseXor, // |
    BitwiseOr,  // ^

    // Brackets
    DoubleSquareBracketBeg, // [[
    DoubleSquareBracketEnd, // ]]

    SquareBracketBeg, // [
    SquareBracketEnd, // ]

    CurlyBracketBeg, // {
    CurlyBracketEnd, // }

    ParenBeg, // (
    ParenEnd, // )

    AngleBeg, // <
    AngleEnd, // >

    // Equalities
    Equal,        // ==
    NotEqual,     // !=
    BiggerEqual,  // >=
    SmallerEqual, // <=

    // Punctuation
    Scope,     // ::
    Colon,     // :
    Semicolon, // ;
    Comma,     // ,
    Dot,       // .
    Hash,      // #
    At,        // @
    Question,  // ?
    Tilde,     // ~

    // Assignment
    Assign, // =

    // Arithmetic (single)
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Rem, // %

    // Directives / Special
    CompilerDirective, // ![
    Bang,              // !

    ArrowLeft, // <-
    ArrowRigh, // ->

    RotateLeft, // <<|
    RotateRigh, // |>>
  };



  struct lexer
  {
    public:
      inline lexer(qw::module *mod)
        : mod(mod)
        , ctx(mod->ctx())
        , is(mod->mmap()->view())
      {}

    private:
      qw::context *ctx{};
      qw::module *mod{};
      std::string_view is;
      usize off{};

      word m_lexLast{};
      std::stack<word> m_lexStore;

    private:
      [[gnu::hot]] fun f_lex() -> word;

    public:
      [[gnu::hot]] fun kind(char) -> CharKind;

      [[gnu::hot]] fun lex() -> const word& {
        if (!m_lexStore.empty())
          return (m_lexLast = m_lexStore.top(), m_lexStore.pop(), m_lexLast);
        else
          return m_lexLast = f_lex();
      }

      fun  store(word W) { m_lexStore.push(W); };
      fun& last() const { return m_lexLast; }
      fun& offset() { return off; }


      fun asString(const word &W) const -> std::string {
        auto s = W.view().substr(1, W.view().size() - 2);
        std::string res;
        res.reserve(s.size());
        for (size_t i = 0; i < s.size(); ++i) {
          if (s[i] == '\\' && i + 1 < s.size()) {
            ++i;
            switch (s[i]) {
              case 'e': res += '\e'; break;
              case 'n': res += '\n'; break;
              case 'r': res += '\r'; break;
              case 't': res += '\t'; break;
              case '0': res += '\0'; break;
              case '\\': res += '\\'; break;
              case '\"': res += '\"'; break;
              case '\'': res += '\''; break;
              default: res += '\\'; res += s[i]; break;
            }
          }
          else {
            res += s[i];
          }
        }
        return res;
      }

      fun asString() const -> std::string {
        return asString(m_lexLast);
      }


      fun asInteger(const word &W) const -> std::expected<u64, uptr<diagnostic::message>> {
        auto view = W.view();

        int base = 10;
        usize val{}, offset{};

        if (view.size() > 1 && view[0] == '0') {
          if (view[1] == 'x') {
            if (view.size() == 2) return errors::InvalidConstantValue(W, W.str(), "hexadecimal prefix '0x' requires digits");
            base = 16;
            offset = 2;
          }
          ef (view[1] == 'b') {
            if (view.size() == 2) return errors::InvalidConstantValue(W, W.str(), "binary prefix '0b' requires digits");
            base = 2;
            offset = 2;
          }
        }
        
        auto eret = std::from_chars(view.begin() + offset, view.end(), val, base);
        if (eret.ec != std::errc() || eret.ptr != view.end()) {
          return errors::CantConvertInteger(W, W.str());
        }
        
        return val;
      }

      fun asInteger() const -> std::expected<u64, uptr<diagnostic::message>> {
        return asInteger(m_lexLast);
      }


    public:
      inline fun operator()() -> const word& { return lex(); }
  };

}
