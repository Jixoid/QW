/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/



#include "qw/basis.hh"
#include "qw/front/front.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include "qw/control/scopemng.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include <charconv>
#include <expected>
#include <fcntl.h>
#include <iostream>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if



#define Require(X) \
  if (!X.has_value()) return fatals::FileEndedButContextNotFinished();

#define Require_Word(X,C) { \
  Require(X); \
  if (!isWord(X->view()[0])) {\
    auto E = errors::expectedAWord(*X, X->str()); \
    sum.add(E.error().get()); \
    std::cerr << E.error(); \
    \
    C; \
  } \
}


#define expected(LEX,T) { \
  auto X = LEX; \
  Require(X) \
  ef (X->view() != T) \
    return errors::expectedIdentifierBut(*X, X->str(), T); \
}


#define if_error(X) { \
  auto E = X; \
  \
  if (!E.has_value()) \
  { \
    if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal) \
      return std::unexpected(std::move(E.error())); \
    else { \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
    } \
  } \
}


#define if_except(X) { \
  auto E = X; \
  \
  if (!E.has_value()) \
    return std::unexpected(std::move(E.error())); \
}


#define print(E) { \
  if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal) \
    return std::unexpected(std::move(E.error())); \
  else { \
    sum.add(E.error().get()); \
    std::cerr << E.error(); \
  } \
}


#define val_error(X) { \
  if (!X.has_value()) \
    return std::unexpected(std::move(X.error())); \
}




namespace qw
{

  fun frontend::read_File(decls::NameSpace *self) -> std::expected<void, uptr<diagnostic::qMessage>>
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




  fun frontend::read_Module(decls::NameSpace *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    auto self = decls::Decl::make_NameSpace(ctx, parent, Name->str(), *Name);
    ctx->gst().add_ident(scopemng::mangling_abi_qw(self), self);
    
    expected(Lex(), "{");
    

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

  fun frontend::read_NameSpace(decls::NameSpace *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    auto self = decls::Decl::make_NameSpace(ctx, parent, Name->str(), *Name);
    ctx->gst().add_ident(scopemng::mangling_abi_qw(self), self);

    expected(Lex(), "{");
    

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




  fun frontend::read_TypeDecl(decls::NameSpace *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);
    
    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);

    // Assign
    expected(Lex(), "=");

    // Type
    auto Type = read_Type(self);
    val_error(Type);

    
    // End
    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_FuncDecl(decls::NameSpace *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    // FuncDecl
    auto self = decls::Decl::make_Func(ctx, parent, Name->str(), *Name, nil);
    ctx->gst().add_ident(scopemng::mangling_abi_qw(self), self);

    // FuncType
    auto FType = types::Type::make_Func(ctx, self, *Name, {}, ctx->void_t());
    expected(Lex(), "(");


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
      std::vector<std::string> Names;
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
        return errors::UnexpectedIdentifier(*End, End->str());


      // Add
      for (auto &X: Names) {
        auto obj = types::Type::make_MemberField(ctx, FType, *Name, X, *Type);
        ctx->gst().add_ident(scopemng::mangling_abi_qw(obj), obj);
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
      return errors:: expectedIdentifierBut2(*Ret, Ret->str(), ";","{");
  }

  fun frontend::read_AliasDecl(decls::NameSpace *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    // Assign
    expected(Lex(), "=");

    // Decl
    auto Target = Lex();
    Require(Target);
    auto Decl = types::Type::make_Nick(ctx, parent, *Target, Target->str());

    // Create
    auto self = decls::Decl::make_Alias(ctx, parent, Name->str(), Decl, *Name);
    ctx->gst().add_ident(scopemng::mangling_abi_qw(self), self);

    // End
    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_RecordDecl(decls::NameSpace *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);
    
    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);

    // Type
    auto Type = read_RecordType(self, false);
    val_error(Type);

    return {};
  }




  fun frontend::read_CodeBlock(decls::Func *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    auto self = stmts::Stmt::make_CodeBlock(ctx, parent, *LexLast());

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

  fun frontend::read_VarStmt(identy *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
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
      stmts::Stmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name);

      goto l_re;
    }
    ef (End->view() == "=") {
      auto Expr = read_Expr(parent);
      val_error(Expr);

      stmts::Stmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name, *Expr, End);

      auto NE = Lex();
      Require(NE);

      if (NE->view() == ",") goto l_re;
      else expected(NE, ";");
    }
    else {
      stmts::Stmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name);

      expected(End, ";");
    }
    
    return {};
  }
  
  fun frontend::read_ReturnStmt(identy *parent) -> std::expected<void, uptr<diagnostic::qMessage>>
  {
    auto Pos = LexLast();

    auto Expr = read_Expr(parent);
    val_error(Expr);
    
    auto self = stmts::Stmt::make_Return(ctx, parent, *Pos, *Expr);


    // End
    expected(Lex(), ";");

    return {};
  }




  fun frontend::read_Expr(identy *parent, bool first) -> std::expected<exprs::Expr*, uptr<diagnostic::qMessage>>
  {
    exprs::Expr *ret{};
    auto ID = Lex();


    if (ID->view() == "true" || ID->view() == "false") {
      
      ret = exprs::Expr::make_BoolLiteral(ctx, parent, ID->view() == "true", *ID);
    }
    ef (ID->view() == "null") {
      
      ret = exprs::Expr::make_PtrLiteral(ctx, parent, 0, *ID);
    }
    ef (isNumber(ID->view()[0])) {
      u128 val;
      
      auto eret = std::from_chars(ID->view().begin(), ID->view().end(), val);
      if (eret.ec != std::errc())
        return errors::CantConvertInteger(*ID, ID->str());

      ret = exprs::Expr::make_IntegerLiteral(ctx, parent, val, *ID);
    }
    ef (ID->view()[0] == '\'') {
      
      if (ID->view().size() == 2)
        return errors::EmptyCharacterConstant(*ID);

      ef (ID->view().size() > 3)
        return errors::CharacterConstantTooLong(*ID, (std::string)ID->view().substr(1, ID->view().size()-2));

      ret = exprs::Expr::make_CharLiteral(ctx, parent, ID->view()[1], *ID);
    }
    ef (isWord(ID->view()[0])) {
      ret = exprs::Expr::make_Nick(ctx, parent, ID->str(), *ID);
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

      ret = exprs::Expr::make_BinaryOp(ctx, parent, exprs::BinaryOpEnum::Add, ret, *r2, *Op);
      goto l_re;
    }
    else LexStore(Op);

    return ret;
  }
  



  fun frontend::read_Type(identy *parent, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::qMessage>>
  {
    // Select
    auto ID = Lex();
    Require(ID);


    if (ID->view() == "record") return read_RecordType(parent, indecl);
    if (ID->view() == "fun") return read_FuncType(parent, indecl);

    else {
      if (indecl) {
        auto Nick = types::Type::make_Nick(ctx, parent, *ID, ID->str());

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

  fun frontend::read_RecordType(identy *parent, bool indecl) -> std::expected<types::Record*, uptr<diagnostic::qMessage>>
  {
    auto Bracket = Lex();
    expected(Bracket, "{");
    

    // Record
    auto self = types::Type::make_Record(ctx, parent, *Bracket, {},{},{});
    ctx->gst().add_ident(scopemng::mangling_abi_qw(self), self);
    

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
      std::vector<std::string> Names;
      l_re:
      auto Name = Lex();
      Require_Word(Name, continue);
      Names.push_back(Name->str());

      // End
      auto End = Lex();
      Require(End);
      
      if (End->view() == ",") goto l_re;
      else expected(End, ";");


      // Add
      for (auto &X: Names) {
        auto obj = types::Type::make_MemberField(ctx, self, *Name, X, *Type);
        ctx->gst().add_ident(scopemng::mangling_abi_qw(obj), obj);
        self->vars().push_back(obj);
      }
    }


    return self;
  }

  fun frontend::read_FuncType(identy *parent, bool indecl) -> std::expected<types::Func*, uptr<diagnostic::qMessage>>
  {
    auto Bracket = Lex();
    expected(Bracket, "(");


    // FuncDecl
    auto self = types::Type::make_Func(ctx, parent, *Bracket, {}, ctx->void_t());
    ctx->gst().add_ident(scopemng::mangling_abi_qw(self), self);


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
      std::vector<std::string> Names;
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
        return errors::UnexpectedIdentifier(*End, End->str());


      // Add
      for (auto &X: Names) {
        auto obj = types::Type::make_MemberField(ctx, self, *Name, X, *Type);
        ctx->gst().add_ident(scopemng::mangling_abi_qw(obj), obj);
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
