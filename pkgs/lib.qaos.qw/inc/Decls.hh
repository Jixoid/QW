/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "Context.hh"
#include "Types.hh"
#include <cassert>
#include <llvm/IR/Function.h>
#include <string>

#define ef else if

using namespace std;



namespace qw::decls
{

  // base
  struct qDecl: qIdentifier
  {
    public:
      enum qEnum: u16 {
        qde_Unknown,

        qde_ModuleDecl,
        qde_NameSpaceDecl,
        qde_FuncDecl,
        qde_VarDecl,
        qde_TypeDecl,
        qde_AliasDecl,
      };

    protected:
      qDecl(qEnum type, qDecl *parent, string name, word pos);


    public:
      static fun make_NameSpace(qw::context *ctx, qDecl *parent, string name, word pos) -> qNameSpaceDecl*;
      static fun make_Module(qw::context *ctx, qDecl *parent, string name, word pos) -> qModuleDecl*;
      static fun make_Var(qw::context *ctx, qDecl *parent, string name, types::qType *type, word pos) -> qVarDecl*;
      static fun make_Type(qw::context *ctx, qDecl *parent, string name, word pos, types::qType *type = Nil) -> qTypeDecl*;
      static fun make_Func(qw::context *ctx, qDecl *parent, string name, word pos, types::qFuncType *type = Nil) -> qFuncDecl*;
      static fun make_Alias(qw::context *ctx, qDecl *parent, string name, qIdentifier *decl, word pos) -> qAliasDecl*;

      fun dis() -> void;
      

    private:
      qEnum m_subType{qde_Unknown};
      string m_name;

    public:
      inline fun name() const { return m_name; }
      inline fun subType() const { return m_subType; }
  };



  // decls
  struct qNameSpaceDecl: qDecl
  {
    friend class qDecl;
    friend class qModuleDecl;

    protected:
      inline qNameSpaceDecl(qDecl *parent, string name, word pos): qDecl(qde_NameSpaceDecl, parent, name, pos) {}
      inline qNameSpaceDecl(qEnum type, qDecl *parent, string name, word pos): qDecl(type, parent, name, pos) {}


    private:
      vector<decls::qDecl*> m_decls;

    public:
      inline fun& decls() const { return m_decls; }
  };

  struct qModuleDecl: qNameSpaceDecl
  {
    friend class qDecl;

    protected:
      inline qModuleDecl(qDecl *parent, string name, word pos): qNameSpaceDecl(qde_ModuleDecl, parent, name, pos) {}

  };

  struct qVarDecl: qDecl
  {
    friend class qDecl;

    protected:
      qVarDecl(qDecl *parent, string name, types::qType *type, word pos);


    private:
      types::qType *m_varType{};
      exprs::qExpr *m_initVal{};  // optional

    public:
      inline fun varType() -> auto&  { return m_varType; }
      inline fun initVal() const  { return m_initVal; }
  };

  struct qTypeDecl: qDecl
  {
    friend class qDecl;

    protected:
      qTypeDecl(qDecl *paren, string name, word pos, types::qType *type = Nil);


    private:
      types::qType *m_targetType{};

    public:
      inline fun& targetType() { return m_targetType; }
  };
  
  struct qFuncDecl: qDecl
  {
    friend class qDecl;

    protected:
      qFuncDecl(qDecl *parent, string name, word pos, types::qFuncType *type = Nil);


    private:
      llvm::Function *m_llvm{};

    public:
      inline fun& llvm() { return m_llvm; }

    private:
      types::qFuncType *m_funcType{};
      stmts::qCodeBlock *m_body{};  // optional

    public:
      inline fun& funcType() { return m_funcType; }
      inline fun& body() { return m_body; }
  };

  struct qAliasDecl: qDecl
  {
    friend class qDecl;
    
    protected:
      inline qAliasDecl(qDecl *parent, string name, qIdentifier *decl, word pos): qDecl(qde_TypeDecl, parent, name, pos), m_targetDecl(decl) {}


    private:
      qIdentifier *m_targetDecl{};

    public:
      inline fun& targetDecl() { return m_targetDecl; }
  };

}
