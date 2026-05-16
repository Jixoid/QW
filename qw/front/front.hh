/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/



#pragma once

#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/control/context.hh"
#include "qw/tree/decls.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/tree/types.hh"
#include <expected>
#include <fcntl.h>
#include <iostream>
#include <string>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <string_view>
#include <unordered_map>

#define ef else if



namespace qw
{

  enum token: u32
  {
    // Equality
    tEquality,     // ==
    tInEquality,   // !=

    tLessEqual,    // <=
    tGreaterEqual, // >=

    tLessThan,     // <
    tGreaterThan,  // >


    // Brackets
    tParenthesis_beg, // (
    tParenthesis_end, // )

    tCurlyBracket_beg, // {
    tCurlyBracket_end, // }

    tSquareBracket_beg, // [
    tSquareBracket_end, // ]

    tDoubleSquareBracket_beg, // [[
    tDoubleSquareBracket_end, // ]]


    // Arithmetic
    tPlus,     // +
    tMinus,    // -
    tSlash,    // /
    tPercent,  // %
    tAsterisk, // *


    // Bitwise
    tLeftShift,  // <<
    tRightShift, // >>

    tTilde, // ~
    tCaret, // ^

    tAmpersand, // &
    tPipe,      // |


    // Access
    tArrow, // ->
    tScope, // ::

    tPerion,    // .
    tComma,     // ,
    tColon,     // :
    tSemiColon, // ;
    
    tAt, // @


    // Misc
    tEqual, // =

    tQuestion,    // ?
    tExclamation, // !
  };


  inline std::unordered_map<std::string, token> QToken = {
    // Equality
    {"==", tEquality},
    {"!=", tInEquality},

    {"<=", tLessEqual},
    {">=", tGreaterEqual},

    {"<", tLessThan},
    {">", tGreaterThan},


    // Brackets
    {"(", tParenthesis_beg},
    {")", tParenthesis_end},

    {"{", tCurlyBracket_beg},
    {"}", tCurlyBracket_end},

    {"[", tSquareBracket_beg},
    {"]", tSquareBracket_end},

    {"[[", tDoubleSquareBracket_beg},
    {"]]", tDoubleSquareBracket_end},


    // Arithmetic
    {"+", tPlus},
    {"-", tMinus},
    {"/", tSlash},
    {"%", tPercent},
    {"*", tAsterisk},


    // Bitwise
    {"<<", tLeftShift},
    {">>", tRightShift},

    {"~", tTilde},
    {"^", tCaret},

    {"&", tAmpersand},
    {"|", tPipe},


    // Access
    {"->", tArrow},
    {"::", tScope},

    {".", tPerion},
    {",", tComma},
    {":", tColon},
    {";", tSemiColon},
    
    {"@", tAt},


    // Misc
    {"=", tEqual},

    {"?", tQuestion},
    {"!", tExclamation},
  };

  inline std::unordered_map<token, std::string> QTokenR = {
    // Equality
    {tEquality, "=="},
    {tInEquality, "!="},

    {tLessEqual, "<="},
    {tGreaterEqual, ">="},

    {tLessThan, "<"},
    {tGreaterThan, ">"},


    // Brackets
    {tParenthesis_beg, "("},
    {tParenthesis_end, ")"},

    {tCurlyBracket_beg, "{"},
    {tCurlyBracket_end, "}"},

    {tSquareBracket_beg, "["},
    {tSquareBracket_end, "]"},

    {tDoubleSquareBracket_beg, "[["},
    {tDoubleSquareBracket_end, "]]"},


    // Arithmetic
    {tPlus, "+"},
    {tMinus, "-"},
    {tSlash, "/"},
    {tPercent, "%"},
    {tAsterisk, "*"},


    // Bitwise
    {tLeftShift, "<<"},
    {tRightShift, ">>"},

    {tTilde, "~"},
    {tCaret, "^"},

    {tAmpersand, "&"},
    {tPipe, "|"},


    // Access
    {tArrow, "->"},
    {tScope, "::"},

    {tPerion, "."},
    {tComma, ","},
    {tColon, ":"},
    {tSemiColon, ";"},
    
    {tAt, "@"},


    // Misc
    {tEqual, "="},

    {tQuestion, "?"},
    {tExclamation, "!"},
  };



  class frontend
  {
    public:
      inline frontend(qw::module *mod)
        : mod(mod), ctx(mod->ctx())
        , m_fpath(mod->fpath())
        , is(mod->mmap()->view())
        , m_file({mod, 0, 0})
      {}


      inline fun process() {
        if (auto E = read_File(mod->nameSpace()); !E.has_value()) {
          sum.add(E.error().get());
          std::cerr << E.error();
        }

        return sum;
      }


    protected:
      qw::context *ctx{};
      qw::module *mod{};
      std::string_view m_fpath{};
      word m_file;
      std::string_view is;
      u0 Off{};
      diagnostic::qSummary sum;


    protected:
      // Tokenizer
      std::vector<char>   strings   = {'"', '\''};
      std::vector<char>   Seperater = {/*for comment -->*/'#',   '{','}', '.',':',';',',','=', '(',')', '<','>', '[',']', '-','+','/','%','*','^','~','&','|','@','?','!'};
      std::vector<std::string> BigSyms   = {/*for comment -->*/"//",  "->", "::", "==", "<=", ">=", "!=", "++", "--", "&&", "||", "<<", ">>", "[[", "]]"};
      std::vector<char>   IgnSyms   = {' ', '\n','\r', '\0','\e', '\t'};
      std::vector<std::string> Operators = {
        "+", "-", "*", "/", "%", 
        "!", "~",
        "&", "|", "^",
        "<<", ">>",
      };
      std::vector<std::string> Directs = {

        // General Types
        "deprecated",
        "aligned",
        "volatile",

        // General Symbols
        "weak", "strong",
        "export", "import", "hidden",
        "static", "threadlocal", "inscope",
        "alias",
        "section",


        // symbol_fun
        "noreturn",
        "noexcept",
        "inline:on", "inline:always", "inline:off",
        "naked",
        "pure",
        "leaf",
        "flatten",
        "hot", "cold",

        // symbol_fun (instance)
        "init", "fini",
        "copy", "move",
        "virtual", "override",

        // symbol_var
        "atomic",

        // type_fun
        "cc:dynamic", "cc:cdecl", "cc:stdcall", "cc:regcall", "cc:thiscall", "cc:sysv",

        // type_record
        "packed", "unpacked",
        "sorted", "unsorted",
      };
      std::unordered_map<char,char> Escapes = {
        {'\'', '\''},
        {'\"', '\"'},
        {'\\', '\\'},

        {'0', '\0'},
        {'a', '\a'},
        {'b', '\b'},
        {'e', '\e'},
        {'f', '\f'},
        {'n', '\n'},
        {'r', '\r'},
        {'t', '\t'},
        {'v', '\v'},
      };


    protected:
      fun isIgn(char C) -> bool;
      fun isSeperator(char C) -> bool;
      fun isNumber(char C) -> bool;
      fun isString(char C) -> bool;
      fun isWord(char C) -> bool;


    protected:
      fun LexStore(std::optional<word>) -> void;
      fun LexLast() { return m_lexLast; }

      [[gnu::hot]] fun __Lex() -> std::optional<word>;
      [[gnu::hot]] fun Lex() {
        if (m_lexStore.has_value()) {
          auto Cac= m_lexStore;
          m_lexStore = std::nullopt;
          return Cac;
        }

        m_lexLast = __Lex();
        return m_lexLast;
      }
      

    private:
      std::optional<word> m_lexLast;
      std::optional<word> m_lexStore;


    protected:
      fun read_File(decls::NameSpace*) -> std:: expected<void, uptr<diagnostic::qMessage>>;

      fun read_Module(decls::NameSpace*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      fun read_NameSpace(decls::NameSpace*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      
      fun read_TypeDecl(decls::NameSpace*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      fun read_FuncDecl(decls::NameSpace*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      fun read_AliasDecl(decls::NameSpace*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      fun read_RecordDecl(decls::NameSpace*) -> std:: expected<void, uptr<diagnostic::qMessage>>;

      fun read_CodeBlock(decls::Func*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      fun read_VarStmt(identy*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      fun read_ReturnStmt(identy*) -> std:: expected<void, uptr<diagnostic::qMessage>>;
      
      fun read_Expr(identy*, bool first = true) -> std:: expected<exprs::Expr*, uptr<diagnostic::qMessage>>;

      fun read_Type(identy*, bool indecl = false) -> std:: expected<types::Type*, uptr<diagnostic::qMessage>>;
      fun read_RecordType(identy*, bool indecl) -> std:: expected<types::Record*, uptr<diagnostic::qMessage>>;
      fun read_FuncType(identy*, bool indecl) -> std:: expected<types::Func*, uptr<diagnostic::qMessage>>;
  };

}
