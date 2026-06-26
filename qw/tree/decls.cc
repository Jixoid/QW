/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/tree/decls.hh"
#include "qw/basis.hh"
#include "qw/tree/types.hh"
#include <cassert>
#include <string>
#include <string_view>



namespace qw::decls
{

  Decl::Decl(DeclVari vari, Decl *parent, std::string_view name, word pos, Visibility vis)
    : identy(IdentyEnum::Decl, parent, pos)
    , m_vari(std::move(vari))
    , m_name(name)
    , m_vis(vis)
  {
    if (parent && parent->is<NameSpaceDecl>()) {
      parent->as<NameSpaceDecl>()->decls.push_back(this);
    }
    ef (this->is<FuncDecl>() && parent && parent->is<RecordDecl>()) {
      parent->as<RecordDecl>()->func.push_back(this);
    }
    ef (this->is<ConstructorDecl>() && parent && parent->is<RecordDecl>()) {
      parent->as<RecordDecl>()->constructors.push_back(this);
    }
  }

  fun Decl::make_NameSpace(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis) -> Decl*
  {
    auto obj = new Decl(NameSpaceDecl{}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Var(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type, Visibility vis, exprs::Expr *init) -> Decl*
  {
    auto obj = new Decl(VarDecl{type, init}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Type(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type, Visibility vis) -> Decl*
  {
    auto obj = new Decl(TypeDecl{type}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Func(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type, Visibility vis) -> Decl*
  {
    auto obj = new Decl(FuncDecl{nullptr, type, nullptr}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Constructor(qw::context *ctx, Decl *parent, word pos, types::Type *type, Visibility vis) -> Decl*
  {
    auto obj = new Decl(ConstructorDecl{nullptr, type, {}, nullptr}, parent, "init", pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Alias(qw::context *ctx, Decl *parent, std::string_view name, identy *decl, word pos, Visibility vis) -> Decl*
  {
    auto obj = new Decl(AliasDecl{decl}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Record(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis) -> Decl*
  {
    auto obj = new Decl(RecordDecl{}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Enum(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis) -> Decl*
  {
    auto obj = new Decl(EnumDecl{}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Set(qw::context *ctx, Decl *parent, std::string_view name, word pos, Visibility vis) -> Decl*
  {
    auto obj = new Decl(SetDecl{}, parent, name, pos, vis);
    ctx->push(obj);
    return obj;
  }

}
