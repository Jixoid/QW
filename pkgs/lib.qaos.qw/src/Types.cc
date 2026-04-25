/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "Types.hh"
#include "Context.hh"
#include <string_view>

using namespace std;



namespace qw
{

  fun word::file() -> string_view { return m_mod->mmap().view(); }
  
  fun word::fpath() -> string_view { return m_mod->fpath(); };
  
  fun word::view() -> string_view { return m_mod->mmap().view().substr(m_off,m_size); };
  fun word::str() -> string { return (string)m_mod->mmap().view().substr(m_off,m_size); };


  fun word::interval() -> pair<humanPos,humanPos>
  {
    const auto calc = [](string_view text, u0 offset) -> humanPos
    {
      u0 line = 1, last_newline_pos{};

      for (u0 i{}; i < offset; i++) if (text[i] == '\n') {
        line++;
        last_newline_pos = i+1;
      }

      u0 column = 1;
      for (u0 i = last_newline_pos; i < offset; i++) {
        u8 c = text[i];
        
        if ((c & 0xC0) != 0x80) column++;
      }

      return {line, column};
    };


    auto file = m_mod->mmap().view();
    
    return {calc(file, off()), calc(file, off()+size())};
  }

}
