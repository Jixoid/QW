/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/



#include "Front.hh"
#include "Basis.hh"
#include "Decls.hh"
#include "Exprs.hh"
#include "ScopeManager.hh"
#include "Stmts.hh"
#include "Typs.hh"
#include "Diagnostic.hh"
#include "Msgs.hh"
#include "Typs.hh"
#include <charconv>
#include <expected>
#include <fcntl.h>
#include <iostream>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if


using namespace std;



#define Require(X) \
  if (!X.has_value()) return fatals::FileEndedButContextNotFinished();

#define Require_Word(X,C) { \
  Require(X); \
  if (!isWord(X->view()[0])) {\
    auto E = errors::ExpectedAWord(*X, X->str()); \
    sum.add(E.error().get()); \
    cerr << E.error(); \
    \
    C; \
  } \
}


#define Expected(LEX,T) { \
  auto X = LEX; \
  Require(X) \
  ef (X->view() != T) \
    return errors::ExpectedIdentifierBut(*X, X->str(), T); \
}


#define if_error(X) { \
  auto E = X; \
  \
  if (!E.has_value()) \
  { \
    if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal) \
      return unexpected(std::move(E.error())); \
    else { \
      sum.add(E.error().get()); \
      cerr << E.error(); \
    } \
  } \
}


#define if_except(X) { \
  auto E = X; \
  \
  if (!E.has_value()) \
    return unexpected(std::move(E.error())); \
}


#define print(E) { \
  if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal) \
    return unexpected(std::move(E.error())); \
  else { \
    sum.add(E.error().get()); \
    cerr << E.error(); \
  } \
}


#define val_error(X) { \
  if (!X.has_value()) \
    return unexpected(std::move(X.error())); \
}




namespace qw
{

  fun frontend::read_File(decls::qNameSpaceDecl *self) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Read
    while (true) {
      auto ID = Lex();
      
      if (!ID.has_value()) return {};

      ef (ID->view() == "module") if_error(read_Module(self))
      ef (ID->view() == "namespace") if_error(read_NameSpace(self))

      ef (ID->view() == "alias") if_error(read_AliasDecl(self))
      ef (ID->view() == "type") if_error(read_TypeDecl(self))
      ef (ID->view() == "fun") if_error(read_FuncDecl(self))
      ef (ID->view() == "record") if_error(read_RecordDecl(self))

      else print(errors::UnknownKeyword(*ID, ID->str()));
    }

    return {};
  };




  fun frontend::read_Module(decls::qNameSpaceDecl *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    auto self = decls::qDecl::make_NameSpace(ctx, parent, Name->str(), *Name);
    ctx->gst().add_ident(scopeManager::mangling_abi_qw(self), self);
    
    Expected(Lex(), "{");
    

    // Read
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      ef (ID->view() == "module") if_error(read_Module(self))
      ef (ID->view() == "namespace") if_error(read_NameSpace(self))

      ef (ID->view() == "alias") if_error(read_AliasDecl(self))
      ef (ID->view() == "type") if_error(read_TypeDecl(self))
      ef (ID->view() == "fun") if_error(read_FuncDecl(self))
      ef (ID->view() == "record") if_error(read_RecordDecl(self))

      else print(errors::UnknownKeyword(*ID, ID->str()));
    }

    return {};
  };

  fun frontend::read_NameSpace(decls::qNameSpaceDecl *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    auto self = decls::qDecl::make_NameSpace(ctx, parent, Name->str(), *Name);
    ctx->gst().add_ident(scopeManager::mangling_abi_qw(self), self);

    Expected(Lex(), "{");
    

    // Read
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      ef (ID->view() == "module") if_error(read_Module(self))
      ef (ID->view() == "namespace") if_error(read_NameSpace(self))

      ef (ID->view() == "alias") if_error(read_AliasDecl(self))
      ef (ID->view() == "type") if_error(read_TypeDecl(self))
      ef (ID->view() == "fun") if_error(read_FuncDecl(self))
      ef (ID->view() == "record") if_error(read_RecordDecl(self))

      else print(errors::UnknownKeyword(*ID, ID->str()));
    }

    return {};
  };




  fun frontend::read_TypeDecl(decls::qNameSpaceDecl *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);
    
    auto self = decls::qDecl::make_Type(ctx, parent, Name->str(), *Name);

    // Assign
    Expected(Lex(), "=");

    // Type
    auto Type = read_Type(self);
    val_error(Type);

    
    // End
    Expected(Lex(), ";");

    return {};
  }

  fun frontend::read_FuncDecl(decls::qNameSpaceDecl *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    // FuncDecl
    auto self = decls::qDecl::make_Func(ctx, parent, Name->str(), *Name, Nil);
    ctx->gst().add_ident(scopeManager::mangling_abi_qw(self), self);

    // FuncType
    auto FType = types::qType::make_Func(ctx, self, *Name, {}, ctx->void_t());
    Expected(Lex(), "(");


    // Params
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == ")") break;
      else LexStore(ID);


      // Type
      auto Type = read_Type(FType, true);
      val_error(Type);

      // Name
      vector<string> Names;
      l_re:
      auto Name = Lex();
      Require(Name);
      Names.push_back(Name->str());

      // End
      auto End = Lex();
      Require(End);
      
      if (End->view() == ",") goto l_re;
      ef (End->view() == ";");
      ef (End->view() == ")") LexStore(End);
      else
        return errors::UnExpectedIdentifier(*End, End->str());


      // Add
      for (auto &X: Names) {
        auto obj = types::qType::make_MemberField(ctx, FType, *Name, X, *Type);
        ctx->gst().add_ident(scopeManager::mangling_abi_qw(obj), obj);
        FType->pars().push_back(obj);
      }
    }

    // Return
    auto Ret = Lex();
    Require(Ret)

    if (Ret->view() == "->") {
      auto Type = read_Type(FType, true);
      val_error(Type);
      FType->ret() = *Type;
      
      Ret = Lex();
      Require(Ret);
    }


    // Code & Decl
    if (Ret->view() == ";")
      return {};

    ef (Ret->view() == "{") {
      if_except(read_CodeBlock(self));

      return {};
    }
    else
      return errors::ExpectedIdentifierBut2(*Ret, Ret->str(), ";","{");
  }

  fun frontend::read_AliasDecl(decls::qNameSpaceDecl *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    // Assign
    Expected(Lex(), "=");

    // Decl
    auto Target = Lex();
    Require(Target);
    auto Decl = types::qType::make_Nick(ctx, parent, *Target, Target->str());

    // Create
    auto self = decls::qDecl::make_Alias(ctx, parent, Name->str(), Decl, *Name);
    ctx->gst().add_ident(scopeManager::mangling_abi_qw(self), self);

    // End
    Expected(Lex(), ";");

    return {};
  }

  fun frontend::read_RecordDecl(decls::qNameSpaceDecl *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);
    
    auto self = decls::qDecl::make_Type(ctx, parent, Name->str(), *Name);

    // Type
    auto Type = read_RecordType(self, false);
    val_error(Type);

    return {};
  }




  fun frontend::read_CodeBlock(decls::qFuncDecl *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    auto self = stmts::qStmt::make_CodeBlock(ctx, parent, *LexLast());

    // Read
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      ef (ID->view() == "ret") if_error(read_ReturnStmt(self))
      ef (ID->view() == "var") if_error(read_VarStmt(self))

      else print(errors::UnknownKeyword(*ID, ID->str()));
    }

    return {};
  }

  fun frontend::read_VarStmt(qIdentifier *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    auto Type = read_Type(parent, true);
    val_error(Type);


    // Name
    l_re:
    auto Name = Lex();
    Require(Name);

    auto End = Lex();
    Require(End);

    
    if (End->view() == ",") {
      stmts::qStmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name);

      goto l_re;
    }
    ef (End->view() == "=") {
      auto Expr = read_Expr(parent);
      val_error(Expr);

      stmts::qStmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name, *Expr, End);

      auto NE = Lex();
      Require(NE);

      if (NE->view() == ",") goto l_re;
      else Expected(NE, ";");
    }
    else {
      stmts::qStmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name);

      Expected(End, ";");
    }
    
    return {};
  }
  
  fun frontend::read_ReturnStmt(qIdentifier *parent) -> expected<void, uptr<diagnostic::qMessage>>
  {
    auto Pos = LexLast();

    auto Expr = read_Expr(parent);
    val_error(Expr);
    
    auto self = stmts::qStmt::make_Return(ctx, parent, *Pos, *Expr);


    // End
    Expected(Lex(), ";");

    return {};
  }




  fun frontend::read_Expr(qIdentifier *parent, bool first) -> expected<exprs::qExpr*, uptr<diagnostic::qMessage>>
  {
    exprs::qExpr *ret{};
    auto ID = Lex();


    if (ID->view() == "true" || ID->view() == "false") {
      
      ret = exprs::qExpr::make_BoolLiteral(ctx, parent, ID->view() == "true", *ID);
    }
    ef (ID->view() == "null") {
      
      ret = exprs::qExpr::make_PtrLiteral(ctx, parent, 0, *ID);
    }
    ef (isNumber(ID->view()[0])) {
      u128 val;
      
      auto eret = from_chars(ID->view().begin(), ID->view().end(), val);
      if (eret.ec != std::errc())
        return errors::CantConvertInteger(*ID, ID->str());

      ret = exprs::qExpr::make_IntegerLiteral(ctx, parent, val, *ID);
    }
    ef (ID->view()[0] == '\'') {
      
      if (ID->view().size() == 2)
        return errors::EmptyCharacterConstant(*ID);

      ef (ID->view().size() > 3)
        return errors::CharacterConstantTooLong(*ID, (string)ID->view().substr(1, ID->view().size()-2));

      ret = exprs::qExpr::make_CharLiteral(ctx, parent, ID->view()[1], *ID);
    }
    ef (isWord(ID->view()[0])) {
      ret = exprs::qExpr::make_Nick(ctx, parent, ID->str(), *ID);
    }
    else
      diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());

    if (!first) return ret;

    

    l_re:
    auto Op = Lex();
    Require(Op);

    if (Op->view() == "+") {
      auto r2 = read_Expr(parent, false);
      val_error(r2);

      ret = exprs::qExpr::make_BinaryOp(ctx, parent, exprs::boe_Add, ret, *r2, *Op);
      goto l_re;
    }
    else LexStore(Op);

    return ret;
  }
  



  fun frontend::read_Type(qIdentifier *parent, bool indecl) -> expected<types::qType*, uptr<diagnostic::qMessage>>
  {
    // Select
    auto ID = Lex();
    Require(ID);


    if (ID->view() == "record") return read_RecordType(parent, indecl);
    if (ID->view() == "fun") return read_FuncType(parent, indecl);

    else {
      if (indecl) {
        auto Nick = types::qType::make_Nick(ctx, parent, *ID, ID->str());

        l_re:
        auto S = Lex();
        Require(S);

        if (S->view() == "::") {
          auto N = Lex();
          Require_Word(N, {return Nick;});
          
          Nick->unResolvedName().push_back(N->str());
          goto l_re;
        }
        else
          LexStore(S);

        return Nick;
      }
      else
        return errors::UnknownKeyword(*ID, ID->str());
    }
  }

  fun frontend::read_RecordType(qIdentifier *parent, bool indecl) -> expected<types::qRecordType*, uptr<diagnostic::qMessage>>
  {
    auto Bracket = Lex();
    Expected(Bracket, "{");
    

    // Record
    auto self = types::qType::make_Record(ctx, parent, *Bracket, {},{},{});
    ctx->gst().add_ident(scopeManager::mangling_abi_qw(self), self);
    

    // Items
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else LexStore(ID);


      // Type
      auto Type = read_Type(self, true);
      val_error(Type);

      // Name
      vector<string> Names;
      l_re:
      auto Name = Lex();
      Require_Word(Name, continue);
      Names.push_back(Name->str());

      // End
      auto End = Lex();
      Require(End);
      
      if (End->view() == ",") goto l_re;
      else Expected(End, ";");


      // Add
      for (auto &X: Names) {
        auto obj = types::qType::make_MemberField(ctx, self, *Name, X, *Type);
        ctx->gst().add_ident(scopeManager::mangling_abi_qw(obj), obj);
        self->vars().push_back(obj);
      }
    }


    return self;
  }

  fun frontend::read_FuncType(qIdentifier *parent, bool indecl) -> expected<types::qFuncType*, uptr<diagnostic::qMessage>>
  {
    auto Bracket = Lex();
    Expected(Bracket, "(");


    // FuncDecl
    auto self = types::qType::make_Func(ctx, parent, *Bracket, {}, ctx->void_t());
    ctx->gst().add_ident(scopeManager::mangling_abi_qw(self), self);


    // Params
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == ")") break;
      else LexStore(ID);


      // Type
      auto Type = read_Type(self, true);
      val_error(Type);

      // Name
      vector<string> Names;
      l_re:
      auto Name = Lex();
      Require(Name);
      Names.push_back(Name->str());

      // End
      auto End = Lex();
      Require(End);
      
      if (End->view() == ",") goto l_re;
      ef (End->view() == ";");
      ef (End->view() == ")") LexStore(End);
      else
        return errors::UnExpectedIdentifier(*End, End->str());


      // Add
      for (auto &X: Names) {
        auto obj = types::qType::make_MemberField(ctx, self, *Name, X, *Type);
        ctx->gst().add_ident(scopeManager::mangling_abi_qw(obj), obj);
        self->pars().push_back(obj);
      }
    }


    // Return
    auto Ret = Lex();
    Require(Ret)

    if (Ret->view() == "->") {
      auto Type = read_Type(self, true);
      val_error(Type);
      self->ret() = *Type;
      
      Ret = Lex();
      Require(Ret);
    }
    else
      LexStore(Ret);


    return self;
  }

}
