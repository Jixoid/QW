/*
  This file is part of QLang.
 
  This  file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QLang. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025-2026 by Kadir Aydın.
*/



#include <iostream>
#include <optional>
#include <string_view>

#include "Basis.hh"
#include "Types.hh"
#include "Front.hh"

#define ef else if

using namespace std;



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
    return find(Strings.begin(), Strings.end(), C) != Strings.end();
  }

  fun frontend::isWord(char C) -> bool {
    return (
      !isdigit(C) && !isIgn(C) &&
      find(Strings.begin(), Strings.end(), C) == Strings.end() &&
      find(Seperater.begin(), Seperater.end(), C) == Seperater.end()
    );
  }



  fun frontend::LexStore(optional<word> W) -> void
  {
    m_lexStore.emplace(*W);
  }


  [[gnu::hot]] fun frontend::__Lex() -> optional<word>
  {
    entry:
    // EOF
    if (Off >= is.size()) return nullopt;


    // String
    if (char StrSym = is[Off]; isString(is[Off])) {
      auto LegOff = Off++;

      while (Off < is.size() && is[Off] != StrSym) Off++;

      if (Off < is.size()) Off++;
      else
        cerr << "fatal: string is unterminated" << endl;
      
      return word{mod, LegOff, Off-LegOff};
    }


    // Comment
    if (is[Off] == '#') {
      while (Off < is.size() && is[Off++] != '\n');
      
      goto entry;
    }
    if (Off+1 < is.size() && string_view(&is[Off], 2) == "//") {
      while (Off < is.size() && is[Off++] != '\n');

      goto entry;
    }


    // Big Symbols
    if (Off+1 < is.size() && find(BigSyms.begin(), BigSyms.end(), string_view(&is[Off], 2)) != BigSyms.end()) {
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
