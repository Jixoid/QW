/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qw/basis.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/types.hh"
#include <cassert>
#include <string>
#include <string_view>



namespace qw::decls
{
  Decl::Decl(DeclEnum type, Decl *parent, std::string_view name, word pos)
    : identy(IdentyEnum::Decl, parent, pos)
    , m_subType(type)
    , m_name(name)
  {
    if (parent && parent->subType() == DeclEnum::NameSpace)
      static_cast<NameSpace*>(parent)->m_decls.push_back(this);
  }


  fun Decl::make_NameSpace(qw::context *ctx, Decl *parent, std::string_view name, word pos) -> NameSpace*
  {
    auto obj = new NameSpace(parent, name, pos);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Module(qw::context *ctx, Decl *parent, std::string_view name, word pos) -> Module*
  {
    auto obj = new Module(parent, name, pos);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Var(qw::context *ctx, Decl *parent, std::string_view name, types::Type *type, word pos) -> Var*
  {
    auto obj = new Var(parent, name, type, pos);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Type(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type) -> Type*
  {
    auto obj = new Type(parent, name, pos, type);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Func(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Func *type) -> Func*
  {
    auto obj = new Func(parent, name, pos, type);
    ctx->push(obj);
    return obj;
  }

  fun Decl::make_Alias(qw::context *ctx, Decl *parent, std::string_view name, identy *decl, word pos) -> Alias*
  {
    auto obj = new Alias(parent, name, decl, pos);
    ctx->push(obj);
    return obj;
  }



  fun Decl::dis() -> void { switch (subType())
  {
    case DeclEnum::Module: delete (decls::Module*)this; break;
    case DeclEnum::NameSpace: delete (decls::NameSpace*)this; break;
    case DeclEnum::Alias: delete (decls::Alias*)this; break;
    case DeclEnum::Func: delete (decls::Func*)this; break;
    case DeclEnum::Var: delete (decls::Var*)this; break;
    case DeclEnum::Type: delete (decls::Type*)this; break;
  }}
  


  Var::Var(Decl *parent, std::string_view name, types::Type *type, word pos): Decl(DeclEnum::Var, parent, name, pos), m_varType(type)
  {
    assert(!name.empty() && "Name is empty");
    assert(type && "Type is null");

    type->parent() = this;
  }


  Type::Type(Decl *parent, std::string_view name, word pos, types::Type *type): Decl(DeclEnum::Type, parent, name, pos), m_targetType(type)
  {
    assert(!name.empty() && "Name is empty");

    if (type) type->parent() = this;
  }

  
  Func::Func(Decl *parent, std::string_view name, word pos, types::Func *type): Decl(DeclEnum::Func, parent, name, pos), m_funcType(type)
  {
    assert(!name.empty() && "Name is empty");
    
    if (type) type->parent() = this;
  }
  
}
