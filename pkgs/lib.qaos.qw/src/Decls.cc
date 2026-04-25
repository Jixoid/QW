/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "Typs.hh"
#include "Decls.hh"
#include <cassert>
#include <string>



namespace qw::decls
{
  qDecl::qDecl(qEnum type, qDecl *parent, string name, word pos)
    : qIdentifier(qIdentifier::qEnum::qie_Decl, parent, pos)
    , m_subType(type)
    , m_name(name)
  {
    if (parent && parent->subType() == qde_NameSpaceDecl)
      static_cast<qNameSpaceDecl*>(parent)->m_decls.push_back(this);
  }


  fun qDecl::make_NameSpace(qw::context *ctx, qDecl *parent, string name, word pos) -> qNameSpaceDecl*
  {
    auto obj = new qNameSpaceDecl(parent, name, pos);
    ctx->push(obj);
    return obj;
  }

  fun qDecl::make_Module(qw::context *ctx, qDecl *parent, string name, word pos) -> qModuleDecl*
  {
    auto obj = new qModuleDecl(parent, name, pos);
    ctx->push(obj);
    return obj;
  }

  fun qDecl::make_Var(qw::context *ctx, qDecl *parent, string name, types::qType *type, word pos) -> qVarDecl*
  {
    auto obj = new qVarDecl(parent, name, type, pos);
    ctx->push(obj);
    return obj;
  }

  fun qDecl::make_Type(qw::context *ctx, qDecl *parent, string name, word pos, types::qType *type) -> qTypeDecl*
  {
    auto obj = new qTypeDecl(parent, name, pos, type);
    ctx->push(obj);
    return obj;
  }

  fun qDecl::make_Func(qw::context *ctx, qDecl *parent, string name, word pos, types::qFuncType *type) -> qFuncDecl*
  {
    auto obj = new qFuncDecl(parent, name, pos, type);
    ctx->push(obj);
    return obj;
  }

  fun qDecl::make_Alias(qw::context *ctx, qDecl *parent, string name, qIdentifier *decl, word pos) -> qAliasDecl*
  {
    auto obj = new qAliasDecl(parent, name, decl, pos);
    ctx->push(obj);
    return obj;
  }



  fun qDecl::dis() -> void { switch (subType()) {
    case decls::qDecl::qde_Unknown: break;

    case decls::qDecl::qde_ModuleDecl: delete (decls::qModuleDecl*)this; break;
    case decls::qDecl::qde_NameSpaceDecl: delete (decls::qNameSpaceDecl*)this; break;
    case decls::qDecl::qde_AliasDecl: delete (decls::qAliasDecl*)this; break;
    case decls::qDecl::qde_FuncDecl: delete (decls::qFuncDecl*)this; break;
    case decls::qDecl::qde_VarDecl: delete (decls::qVarDecl*)this; break;
    case decls::qDecl::qde_TypeDecl: delete (decls::qTypeDecl*)this; break;
  }}
  


  qVarDecl::qVarDecl(qDecl *parent, string name, types::qType *type, word pos): qDecl(qde_VarDecl, parent, name, pos), m_varType(type)
  {
    assert(!name.empty() && "Name is empty");
    assert(type && "Type is null");

    type->parent() = this;
  }


  qTypeDecl::qTypeDecl(qDecl *parent, string name, word pos, types::qType *type): qDecl(qde_TypeDecl, parent, name, pos), m_targetType(type)
  {
    assert(!name.empty() && "Name is empty");

    if (type) type->parent() = this;
  }

  
  qFuncDecl::qFuncDecl(qDecl *parent, string name, word pos, types::qFuncType *type): qDecl(qde_FuncDecl, parent, name, pos), m_funcType(type)
  {
    assert(!name.empty() && "Name is empty");
    
    if (type) type->parent() = this;
  }
  
}
