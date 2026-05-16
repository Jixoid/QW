/*
  This file is part of qw.
 
  This  file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with qw. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025-2026 by Kadir Aydın.
*/



#include <iostream>
#include <optional>
#include <string_view>

#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/front/front.hh"

#define ef else if



namespace qw
{
  
  fun frontend::isIgn(char C) -> bool {
    return find(IgnSyms.begin(), IgnSyms.end(), C) != IgnSyms.end();
  }

  fun frontend::isSeperator(char C) -> bool {
    return find(Seperater.begin(), Seperater.end(), C) != Seperater.end();
  }

  fun frontend::isNumber(char C) -> bool {
    return isdigit(C);
  }

  fun frontend::isString(char C) -> bool {
    return find(strings.begin(), strings.end(), C) != strings.end();
  }

  fun frontend::isWord(char C) -> bool {
    return (
      !isdigit(C) && !isIgn(C) &&
      find(strings.begin(), strings.end(), C) == strings.end() &&
      find(Seperater.begin(), Seperater.end(), C) == Seperater.end()
    );
  }



  fun frontend::LexStore(std::optional<word> W) -> void
  {
    m_lexStore.emplace(*W);
  }


  [[gnu::hot]] fun frontend::__Lex() -> std::optional<word>
  {
    entry:
    // EOF
    if (Off >= is.size()) return std::nullopt;


    // std::string
    if (char StrSym = is[Off]; isString(is[Off])) {
      auto LegOff = Off++;

      while (Off < is.size() && is[Off] != StrSym) Off++;

      if (Off < is.size()) Off++;
      else
        std::cerr << "fatal: std::string is unterminated" << std::endl;
      
      return word{mod, LegOff, Off-LegOff};
    }


    // Comment
    if (is[Off] == '#') {
      while (Off < is.size() && is[Off++] != '\n');
      
      goto entry;
    }
    if (Off+1 < is.size() && std::string_view(&is[Off], 2) == "//") {
      while (Off < is.size() && is[Off++] != '\n');

      goto entry;
    }


    // Big Symbols
    if (Off+1 < is.size() && find(BigSyms.begin(), BigSyms.end(), std::string_view(&is[Off], 2)) != BigSyms.end()) {
      Off += 2;

      return word{mod, Off-2, 2};
    }


    // Symbols
    if (isSeperator(is[Off])) {
      Off++;

      return word{mod, Off-1, 1};
    }


    // Whitespace
    if (isIgn(is[Off])) {
      while (Off < is.size() && isIgn(is[Off])) Off++;

      goto entry;
    }


    // Identifier
    auto LegOff = Off;

    while (Off < is.size() && (isWord(is[Off]) || isNumber(is[Off]))) Off++;
    
    if (LegOff == Off) {
      Off++;

      goto entry; 
    }

    return word{mod, LegOff, Off-LegOff};
  };

}
