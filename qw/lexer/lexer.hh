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
      u0 off{};

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

    public:
      inline fun operator()() -> const word& { return lex(); }
  };

}
