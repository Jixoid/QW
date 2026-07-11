/*
  This file is part of qw.

  This  file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with qw. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include <array>
#include <iostream>
#include <string_view>

#include "qw/basis.hh"
#include "qw/lexer/lexer.hh"
#include "qw/pretype.hh"

#define ef else if



namespace qw
{

  static constexpr std::array<CharKind, 256> CreateCharLUT() {
    std::array<CharKind, 256> lut{};
    lut.fill(CharKind::Word); // Default Word

    // Whitespace
    for (char c: {' ', '\n', '\r', '\t'}) lut[static_cast<uint8_t>(c)] = CharKind::Ignored;

    // Numeral
    for (char c = '0'; c <= '9'; ++c) lut[static_cast<uint8_t>(c)] = CharKind::Numeral;

    // String
    lut['\''] = CharKind::String;
    lut['"'] = CharKind::String;

    // Symbols
    for (char c: {'#', '{', '}', '.', ':', ';', ',', '=', '(', ')', '<', '>', '[', ']', '-', '+', '/', '%', '*', '^', '~', '&', '|', '@', '?', '!'}) {
      lut[static_cast<uint8_t>(c)] = CharKind::Symbol;
    }

    return lut;
  }

  static constexpr auto CharLUT = CreateCharLUT();

  [[gnu::hot]]
  fun lexer::kind(char C) -> CharKind { return CharLUT[static_cast<uint8_t>(C)]; }


  [[gnu::hot]]
  fun lexer::f_lex() -> word {
    entry:

    // EOF
    if (off >= is.size()) return word{};

    auto knd = kind(is[off]);

    
    // string
    if (knd == CharKind::String) {
      char StrSym = is[off];
      auto legoff = off++;

      while (off < is.size() && is[off] != StrSym)
        if (is[off] == '\\' && off+1 < is.size())
          off += 2;
        else
          off++;

      if (off < is.size()) off++;
      else
        std::cerr << "fatal: std::string is unterminated" << std::endl;

      return word{mod, legoff, off-legoff};
    }

    // Whitespace
    ef (knd == CharKind::Ignored) {
      while (off < is.size() && kind(is[off]) == CharKind::Ignored) off++;
      goto entry;
    }

    // Symbols
    ef (knd == CharKind::Symbol) {
      auto legoff = off;
      
      switch (is[off]) {
        case '<':
          if (off + 1 < is.size()) {
            if (is[off+1] == '<') {
              if (off + 2 < is.size() && is[off+2] == '=') { off += 3; return word{mod, legoff, 3}; } // "<<="
              if (off + 2 < is.size() && is[off+2] == '|') { off += 3; return word{mod, legoff, 3}; } // "<<|"
              off += 2; return word{mod, legoff, 2}; // "<<"
            }
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "<="
            if (is[off+1] == '-') { off += 2; return word{mod, legoff, 2}; } // "<-"
          }
          break;

        case '>':
          if (off + 1 < is.size()) {
            if (is[off+1] == '>') {
              if (off + 2 < is.size() && is[off+2] == '=') { off += 3; return word{mod, legoff, 3}; } // ">>="
              off += 2; return word{mod, legoff, 2}; // ">>"
            }
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // ">="
          }
          break;

        case '|':
          if (off + 1 < is.size()) {
            if (is[off+1] == '>') {
              if (off + 2 < is.size() && is[off+2] == '>') { off += 3; return word{mod, legoff, 3}; } // "|>>"
            }
            if (is[off+1] == '|') { off += 2; return word{mod, legoff, 2}; } // "||"
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "|="
          }
          break;

        case '-':
          if (off + 1 < is.size()) {
            if (is[off+1] == '>') { off += 2; return word{mod, legoff, 2}; } // "->"
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "-="
          }
          break;

        case '+':
          if (off + 1 < is.size() && is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "+="
          break;

        case '*':
          if (off + 1 < is.size() && is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "*="
          break;

        case '%':
          if (off + 1 < is.size() && is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "%="
          break;

        case '=':
          if (off + 1 < is.size() && is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "=="
          break;

        case ':':
          if (off + 1 < is.size() && is[off+1] == ':') { off += 2; return word{mod, legoff, 2}; } // "::"
          break;

        case '/':
          if (off + 1 < is.size()) {
            if (is[off+1] == '/') { // Comment
              while (off < is.size() && is[off] != '\n') off++;
              goto entry;
            }
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "/="
          }
          break;

        case '!':
          if (off + 1 < is.size()) {
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "!="
            if (is[off+1] == '[') { off += 2; return word{mod, legoff, 2}; } // "!["
          }
          break;

        case '&':
          if (off + 1 < is.size()) {
            if (is[off+1] == '&') { off += 2; return word{mod, legoff, 2}; } // "&&"
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "&="
          }
          break;

        case '^':
          if (off + 1 < is.size()) {
            if (is[off+1] == '^') { off += 2; return word{mod, legoff, 2}; } // "^^"
            if (is[off+1] == '=') { off += 2; return word{mod, legoff, 2}; } // "^="
          }
          break;

        case '[':
          if (off + 1 < is.size() && is[off+1] == '[') { off += 2; return word{mod, legoff, 2}; } // "[["
          break;

          case ']':
          if (off + 1 < is.size() && is[off+1] == ']') { off += 2; return word{mod, legoff, 2}; } // "]]"
          break;
      }

      // Small Symbol
      off++;
      return word{mod, legoff, 1};
    }

    
    // Word
    auto legoff = off;

    while (off < is.size() && (kind(is[off]) & (CharKind::Word | CharKind::Numeral))) off++;

    if (legoff == off) {
      off++;
      goto entry;
    }

    return word{mod, legoff, off-legoff};
  };

}
