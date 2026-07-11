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
#include "qw/lexer/lexer.hh"
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

#define ef else if

namespace qw
{

  enum struct Precedence: i32 {
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
        , lexer(mod)
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
      qw::lexer lexer;
      qw::context *ctx{};
      qw::module *mod{};
      std::string_view m_fpath{};
      word m_file;
      std::string_view is;
      u0 Off{};
      diagnostic::summary sum;
      std::vector<decls::Attribute> m_current_attrs;
      
    public:
      fun read_File(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
    
      
    protected:
      fun read_Route(std::string id, word w, decls::Decl *self, VisibilityFlag visflag) -> std::expected<void, uptr<diagnostic::message>>;
      
      fun read_Attributes() -> std::expected<void, uptr<diagnostic::message>>;
      fun read_FileAttributes(decls::Decl *self) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_TypeDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_FuncParams(identy *parent) -> std::expected<std::vector<types::FieldType>, uptr<diagnostic::message>>;
      fun read_FuncDecl(decls::Decl*) -> std::expected<decls::Decl*, uptr<diagnostic::message>>;
      fun read_AliasDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_VarDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_StructDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_EnumDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_SetDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_IFaceDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_ModDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_StructFuncDecl(decls::Decl*, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_StructConstructorDecl(decls::Decl*, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_StructDestructorDecl(decls::Decl*, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>;

      fun read_CodeBlock(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_SingleStmt(identy*, std::optional<word>) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_BlockOrStmt(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_ExprStmt(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_GenericParams(decls::Decl *parent) -> std::expected<decls::GenericContext*, uptr<diagnostic::message>>;
      fun read_IfStmt(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_WhileStmt(identy*) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;
      fun read_VarStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_LetStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_ReturnStmt(identy*) -> std::expected<void, uptr<diagnostic::message>>;
      fun read_UnsafeStmt(identy*, word pos) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>;

      fun read_Expr(identy*, Precedence prec) -> std::expected<exprs::Expr*, uptr<diagnostic::message>>;
      fun read_Expr_Postfix(identy*, exprs::Expr *ret) -> std::expected<exprs::Expr*, uptr<diagnostic::message>>;
      fun read_Expr_Infix(identy*, Precedence min_prec, exprs::Expr *ret) -> std::expected<exprs::Expr*, uptr<diagnostic::message>>;

      fun read_Type(identy*, bool indecl = false) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun read_StructType(identy *parent, bool indecl, std::vector<types::Type*> baseTypes = {}, std::vector<word> baseTypePos = {}) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun read_IFaceType(identy *parent, bool indecl, std::vector<types::Type*> baseTypes = {}, std::vector<word> baseTypePos = {}) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun read_FuncType(identy*, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>>;
  };

}
