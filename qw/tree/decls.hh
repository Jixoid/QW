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
#include "qw/tree/types.hh"
#include <cassert>
#include <llvm/IR/Function.h>
#include <string>
#include <string_view>
#include <variant>

#define ef else if



namespace qw::decls
{

  struct NameSpaceDecl   { std::vector<decls::Decl*> decls; };
  struct VarDecl         { types::Type *type{}; exprs::Expr *initer{}; };
  struct TypeDecl        { types::Type *type{}; };
  struct FuncDecl        { llvm::Function *llvm{}; types::Type *funcType{}; stmts::Stmt *body{}; /* optional */ };
  struct ConstructorDecl { llvm::Function *llvm{}; types::Type *funcType{}; std::vector<std::pair<std::string, exprs::Expr*>> inits{}; stmts::Stmt *body{}; };
  struct AliasDecl       { identy *decl{}; };
  struct StructDecl      { std::vector<decls::Decl*> func{}; std::vector<decls::Decl*> constructors{}; };
  struct EnumDecl        { std::vector<decls::Decl*> func{}; };
  struct SetDecl         { std::vector<decls::Decl*> func{}; };

  using DeclVari = std::variant<NameSpaceDecl, VarDecl, TypeDecl, FuncDecl, ConstructorDecl, AliasDecl, StructDecl, EnumDecl, SetDecl>;


  struct Decl: qw::identy
  {
    protected:
      explicit Decl(DeclVari vari, Decl *parent, std::string_view name, word pos, Visibility vis = Visibility::Private);

    public:
      static fun make_NameSpace(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis = Visibility::Private) -> Decl*;

      static fun make_Var(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type = nil, Visibility vis = Visibility::Private, exprs::Expr *init = nil) -> Decl*;
      static fun make_Type(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type = nil, Visibility vis = Visibility::Private) -> Decl*;
      static fun make_Func(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type = nil, Visibility vis = Visibility::Private) -> Decl*;
      static fun make_Constructor(qw::context *ctx, Decl *parent, word pos, types::Type *type = nil, Visibility vis = Visibility::Private) -> Decl*;
      static fun make_Alias(qw::context *ctx, Decl *parent, std::string_view name, identy *decl, word pos, Visibility vis = Visibility::Private)-> Decl*;

      static fun make_Struct(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis = Visibility::Private) -> Decl*;
      static fun make_Enum(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis = Visibility::Private) -> Decl*;
      static fun make_Set(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis = Visibility::Private) -> Decl*;

    private:
      DeclVari m_vari;
      std::string m_name, m_cached_symbol;
      Visibility m_vis;

    public:
      inline fun name() const { return (std::string_view)m_name; }
      inline fun &vari() const { return m_vari; }
      inline fun vis() const { return m_vis; }
      fun symbol() -> std::string_view;

    public:
      template<typename T>
      inline fun is() { return std::holds_alternative<T>(m_vari); }

      template<typename T>
      inline fun as() { return &std::get<T>(m_vari); }

      template<typename T>
      inline fun set(T val) { m_vari = val; }

  };

}
