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
    BitOr  = 32, // |
    BitXor = 34, // ^
    BitAnd = 36, // &
    Eq     = 40, // ==, !=
    Rel    = 50, // <, >, <=, >=
    Shift  = 55, // <<, >>
    Add    = 60, // +, -
    Mul    = 70, // *, /, %
    Unary  = 80, // !, ~, @
  };

  inline auto operator+(Precedence p, int i) -> Precedence { return static_cast<Precedence>(static_cast<int>(p) + i); }


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
        /* 3 chars */ "<<=", ">>=", "<<|", "|>>",
        /* 2 chars */ "//", "->", "<-", "::", "==", "<=", ">=", "!=", "&&", "||", "^^", "<<", ">>", "[[", "]]",
        "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^="
      };
      std::vector<char> IgnSyms = {
        ' ', '\n', '\r', '\0', '\e', '\t'
      };
      std::vector<std::string> Operators = {
        "+", "-", "*", "/", "%", "!", "~", "&", "|", "^", "<<", ">>",
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
      fun read_EnumDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_SetDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_RecordFuncDecl(decls::Decl*, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_RecordConstructorDecl(decls::Decl*, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_CodeBlock(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_IfStmt(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_WhileStmt(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_VarStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_LetStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_ReturnStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_Expr(identy*, Precedence prec) -> std::expected<exprs::Expr*, uptr<diagnostic::message>>;

      fun read_Type(identy*, bool indecl = false) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun read_RecordType(identy*, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun read_FuncType(identy*, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>>;
  };

}
