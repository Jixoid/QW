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
#include <cassert>
#include <llvm/IR/Function.h>
#include <string>
#include <string_view>

#define ef else if



namespace qw::decls
{

  enum struct DeclEnum: u8 { Module, NameSpace, Func, Var, Type, Alias };
  
  struct Decl: qw::identy
  {
    protected:
      explicit Decl(DeclEnum type, Decl *parent, std::string_view name, word pos);


    public:
      static fun make_NameSpace(qw::context *ctx, Decl *parent, std::string_view name, word pos) -> NameSpace*;
      static fun make_Module(qw::context *ctx, Decl *parent, std::string_view name, word pos) -> Module*;
      static fun make_Var(qw::context *ctx, Decl *parent, std::string_view name, types::Type *type, word pos) -> Var*;
      static fun make_Type(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Type *type = nil) -> Type*;
      static fun make_Func(qw::context *ctx, Decl *parent, std::string_view name, word pos, types::Func *type = nil) -> Func*;
      static fun make_Alias(qw::context *ctx, Decl *parent, std::string_view name, identy *decl, word pos) -> Alias*;

      fun dis() -> void;
      

    private:
      DeclEnum m_subType;
      std::string m_name;

    public:
      inline fun name() const { return (std::string_view)m_name; }
      inline fun subType() const { return m_subType; }
  };



  // decls
  struct NameSpace: Decl
  {
    friend struct Decl;
    friend struct Module;

    protected:
      explicit inline NameSpace(Decl *parent, std::string_view name, word pos): Decl(DeclEnum::NameSpace, parent, name, pos) {}
      explicit inline NameSpace(DeclEnum type, Decl *parent, std::string_view name, word pos): Decl(type, parent, name, pos) {}


    private:
      std::vector<decls::Decl*> m_decls;

    public:
      inline fun& decls() const { return m_decls; }
  };

  struct Module: NameSpace
  {
    friend struct Decl;

    protected:
      inline Module(Decl *parent, std::string_view name, word pos): NameSpace(DeclEnum::Module, parent, name, pos) {}
  };

  struct Var: Decl
  {
    friend struct Decl;

    protected:
      Var(Decl *parent, std::string_view name, types::Type *type, word pos);


    private:
      types::Type *m_varType{};
      exprs::Expr *m_initVal{};  // std::optional

    public:
      inline fun& varType() { return m_varType; }
      inline fun& initVal() const { return m_initVal; }
  };

  struct Type: Decl
  {
    friend struct Decl;

    protected:
      Type(Decl *paren, std::string_view name, word pos, types::Type *type = nil);


    private:
      types::Type *m_targetType{};

    public:
      inline fun& targetType() { return m_targetType; }
  };
  
  struct Func: Decl
  {
    friend struct Decl;

    protected:
      Func(Decl *parent, std::string_view name, word pos, types::Func *type = nil);


    private:
      llvm::Function *m_llvm{};

    public:
      inline fun& llvm() { return m_llvm; }

    private:
      types::Func *m_funcType{};
      stmts::CodeBlock *m_body{};  // std::optional

    public:
      inline fun& funcType() { return m_funcType; }
      inline fun& body() { return m_body; }
  };

  struct Alias: Decl
  {
    friend struct Decl;
    
    protected:
      inline Alias(Decl *parent, std::string_view name, identy *decl, word pos): Decl(DeclEnum::Type, parent, name, pos), m_targetDecl(decl) {}


    private:
      identy *m_targetDecl{};

    public:
      inline fun& targetDecl() { return m_targetDecl; }
  };

}
