/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/pretype.hh"
#include "qw/basis.hh"
#include "qw/control/context.hh"
#include <string>
#include <string_view>



namespace qw
{

  auto word::file() -> std::string_view { return m_mod->mmap()->view(); }

  auto word::fpath() -> std::string_view { return m_mod->fpath(); };

  auto word::view() -> std::string_view { return m_mod->mmap()->view().substr(m_off, m_size); };
  auto word::str() -> std::string { return (std::string)m_mod->mmap()->view().substr(m_off, m_size); };

  auto word::interval() -> std::pair<humanPos, humanPos>
  {
    const auto calc = [](std::string_view text, u0 offset) -> humanPos {
      u0 line = 1, last_newline_pos{};

      for (u0 i{}; i < offset; i++)
        if (text[i] == '\n') {
          line++;
          last_newline_pos = i + 1;
        }

      u0 column = 1;
      for (u0 i = last_newline_pos; i < offset; i++) {
        u8 c = text[i];

        if ((c & 0xC0) != 0x80)
          column++;
      }

      return { line, column };
    };

    auto file = m_mod->mmap()->view();

    return { calc(file, off()), calc(file, off() + size()) };
  }

}
