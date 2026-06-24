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
#include "qw/diagnostic/diagnostic.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/types.hh"
#include <expected>
#include <fcntl.h>
#include <iostream>
#include <string>
#include <string_view>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <unordered_map>

#define ef else if

namespace qw
{

  enum struct Precedence : int
  {
    Lowest = 0,
    Assign = 10, // =
    LogOr  = 20, // ||
    LogAnd = 30, // &&
    Eq     = 40, // ==, !=
    Rel    = 50, // <, >, <=, >=
    Add    = 60, // +, -
    Mul    = 70, // *, /, %
    Unary  = 80, // !, ~, @
  };

  inline auto operator+(Precedence p, int i) -> Precedence { return static_cast<Precedence>(static_cast<int>(p) + i); }

  enum token : u32
  {
    // Equality
    tEquality,   // ==
    tInEquality, // !=

    tLessEqual,    // <=
    tGreaterEqual, // >=

    tLessThan,    // <
    tGreaterThan, // >

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

    tPeriod,    // .
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
    { "==", tEquality },
    { "!=", tInEquality },

    { "<=", tLessEqual },
    { ">=", tGreaterEqual },

    { "<", tLessThan },
    { ">", tGreaterThan },

    // Brackets
    { "(", tParenthesis_beg },
    { ")", tParenthesis_end },

    { "{", tCurlyBracket_beg },
    { "}", tCurlyBracket_end },

    { "[", tSquareBracket_beg },
    { "]", tSquareBracket_end },

    { "[[", tDoubleSquareBracket_beg },
    { "]]", tDoubleSquareBracket_end },

    // Arithmetic
    { "+", tPlus },
    { "-", tMinus },
    { "/", tSlash },
    { "%", tPercent },
    { "*", tAsterisk },

    // Bitwise
    { "<<", tLeftShift },
    { ">>", tRightShift },

    { "~", tTilde },
    { "^", tCaret },

    { "&", tAmpersand },
    { "|", tPipe },

    // Access
    { "->", tArrow },
    { "::", tScope },

    { ".", tPeriod },
    { ",", tComma },
    { ":", tColon },
    { ";", tSemiColon },

    { "@", tAt },

    // Misc
    { "=", tEqual },

    { "?", tQuestion },
    { "!", tExclamation },
  };

  inline std::unordered_map<token, std::string> QTokenR = {
    // Equality
    { tEquality, "==" },
    { tInEquality, "!=" },

    { tLessEqual, "<=" },
    { tGreaterEqual, ">=" },

    { tLessThan, "<" },
    { tGreaterThan, ">" },

    // Brackets
    { tParenthesis_beg, "(" },
    { tParenthesis_end, ")" },

    { tCurlyBracket_beg, "{" },
    { tCurlyBracket_end, "}" },

    { tSquareBracket_beg, "[" },
    { tSquareBracket_end, "]" },

    { tDoubleSquareBracket_beg, "[[" },
    { tDoubleSquareBracket_end, "]]" },

    // Arithmetic
    { tPlus, "+" },
    { tMinus, "-" },
    { tSlash, "/" },
    { tPercent, "%" },
    { tAsterisk, "*" },

    // Bitwise
    { tLeftShift, "<<" },
    { tRightShift, ">>" },

    { tTilde, "~" },
    { tCaret, "^" },

    { tAmpersand, "&" },
    { tPipe, "|" },

    // Access
    { tArrow, "->" },
    { tScope, "::" },

    { tPeriod, "." },
    { tComma, "," },
    { tColon, ":" },
    { tSemiColon, ";" },

    { tAt, "@" },

    // Misc
    { tEqual, "=" },

    { tQuestion, "?" },
    { tExclamation, "!" },
  };


  class frontend
  {
    public:
      inline frontend(qw::module *mod)
        : mod(mod)
        , ctx(mod->ctx())
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
      diagnostic::summary sum;

    protected:
      // Tokenizer
      std::vector<char> strings   = {'"', '\''};
      std::vector<char> Seperater = {
        /*for comment -->*/ '#', '{', '}', '.', ':', ';', ',', '=', '(', ')', '<', '>', '[', ']', '-', '+', '/', '%', '*', '^', '~', '&', '|', '@', '?', '!'
      };
      std::vector<std::string> BigSyms = {
        /*for comment -->*/ "//", "->", "<-", "::", "==", "<=", ">=", "!=", "&&", "||", "^^", "<<", ">>", "<<|", "|>>", "[[", "]]"
      };
      std::vector<char> IgnSyms = {
        ' ', '\n', '\r', '\0', '\e', '\t'
      };
      std::vector<std::string> Operators = {
        "+", "-", "*", "/", "%", "!", "~", "&", "|", "^", "<<", ">>",
      };
      std::vector<std::string> Directs = {

        // General Types
        "deprecated",
        "aligned",
        "volatile",

        // General Symbols
        "weak",
        "strong",
        "export",
        "import",
        "hidden",
        "static",
        "threadlocal",
        "inscope",
        "alias",
        "section",

        // symbol_auto
        "noreturn",
        "noexcept",
        "inline:on",
        "inline:always",
        "inline:off",
        "naked",
        "pure",
        "leaf",
        "flatten",
        "hot",
        "cold",

        // symbol_auto (instance)
        "init",
        "fini",
        "copy",
        "move",
        "virtual",
        "override",

        // symbol_var
        "atomic",

        // type_auto
        "cc:dynamic",
        "cc:cdecl",
        "cc:stdcall",
        "cc:regcall",
        "cc:thiscall",
        "cc:sysv",

        // type_record
        "packed",
        "unpacked",
        "sorted",
        "unsorted",
      };
      std::unordered_map<char, char> Escapes = {
        { '\'', '\'' }, { '\"', '\"' }, { '\\', '\\' },

        { '0', '\0' },  { 'a', '\a' },  { 'b', '\b' },  { 'e', '\e' }, { 'f', '\f' }, { 'n', '\n' }, { 'r', '\r' }, { 't', '\t' }, { 'v', '\v' },
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
          auto Cac   = m_lexStore;
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
      fun read_File(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_NameSpace(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_Route(std::string id, word w, decls::Decl *self, VisibilityFlag visflag) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_TypeDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_FuncDecl(decls::Decl*) -> std::expected<decls::Decl*, uptr<diagnostic::message>>;
      fun read_AliasDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_VarDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_RecordDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_RecordFuncDecl(decls::Decl*, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_CodeBlock(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_VarStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_LetStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_ReturnStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_Expr(identy*, Precedence prec) -> std::expected<exprs::Expr*, uptr<diagnostic::message>>;

      fun read_Type(identy*, bool indecl = false) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun read_RecordType(identy*, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun read_FuncType(identy*, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>>;
  };

}
