/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "Types.hh"
#include <libintl.h>
#include <string>

using namespace std;


inline fun bulk_replace(string_view text, const vector<pair<string, string>> &replacements) -> string
{
  string result;
  result.reserve(text.size() * 1.2); 

  for (u0 i{}; i < text.size();)
  {
    bool matched{};

    for (const auto &r: replacements)
      if (text.substr(i).starts_with(r.first))
      {
        result.append(r.second);
        i += r.first.size();
        matched = true;
        goto l_next;
      }

    if (!matched) {
      result.push_back(text[i]);
      i++;
    }

    l_next:
  }
  return result;
}


inline fun _(string str) -> string
{
  string out = gettext(str.data());

  return bulk_replace(out, {
    {"<reset>", qw::color::RESET},

    {"<gray>",   qw::color::GRAY},
    {"<red>",    qw::color::RED},
    {"<green>",  qw::color::GREEN},
    {"<yellow>", qw::color::YELLOW},
    {"<blue>",   qw::color::BLUE},
    {"<magenta>", qw::color::MAGENTA},
    {"<cyan>",   qw::color::CYAN},
    {"<white>",  qw::color::WHITE},
  });
}
