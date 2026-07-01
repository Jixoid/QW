/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/front/front.hh"
#include "qw/basis.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include <charconv>
#include <expected>
#include <fcntl.h>
#include <filesystem>
#include <iostream>
#include <optional>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if

#define Require(X) \
  if (!X.has_value()) \
    return fatals::FileEndedButContextNotFinished();

#define Require_Word(X, C) \
  { \
    Require(X); \
    if (!isWord(X->view()[0])) { \
      auto E = errors::ExpectedAWord(*X, X->str()); \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
      C; \
    } \
  }

#define expected(LEX, T) \
  { \
    auto X = LEX; \
    Require(X) ef(X->view() != T) return errors::ExpectedIdentifierBut(*X, X->str(), T); \
  }

#define expected2(LEX, T, T2) \
  { \
    auto X = LEX; \
    Require(X) ef(X->view() != T || X->view() != T2) return errors::ExpectedIdentifierBut2(*X, X->str(), T, T2); \
  }

#define if_error(X) \
  { \
    auto E = X; \
    if (!E.has_value()) { \
      if (E.error()->type() == qw::diagnostic::MsgType::Fatal) \
        return std::unexpected(std::move(E.error())); \
      else { \
        sum.add(E.error().get()); \
        std::cerr << E.error(); \
      } \
    } \
  }

#define if_except(X) \
  { \
    auto E = X; \
    if (!E.has_value()) \
      return std::unexpected(std::move(E.error())); \
  }

#define if_except_ref(X) \
  { \
    auto &E = X; \
    if (!E.has_value()) \
      return std::unexpected(std::move(E.error())); \
  }

#define print(E) \
  { \
    if (E.error()->type() == qw::diagnostic::MsgType::Fatal) \
      return std::unexpected(std::move(E.error())); \
    else { \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
    } \
  }

#define val_error(X) \
  { \
    if (!X.has_value()) \
      return std::unexpected(std::move(X.error())); \
  }



namespace qw
{

  fun frontend::read_File(decls::Decl *self) -> std::expected<void, uptr<diagnostic::message>>
  {
    // Read
    while (true) {
      auto ID = Lex();

      if (!ID.has_value()) return {};

      ef(ID->view() == "}") break;
      else if_except(read_Route(ID->str(), ID.value(), self, Visibility::Public | Visibility::Private | Visibility::Crate | Visibility::Group));
    }

    return {};
  };

  fun frontend::read_NameSpace(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    auto self = decls::Decl::make_NameSpace(ctx, parent, Name->str(), *Name);

    expected(Lex(), "{");

    // Read
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}")
        break;
      else
        if_except(read_Route(ID->str(), ID.value(), self, Visibility::Public | Visibility::Private | Visibility::Crate | Visibility::Group));
    }

    return {};
  };

  fun frontend::read_Route(std::string id, word w, decls::Decl *self, VisibilityFlag visflag) -> std::expected<void, uptr<diagnostic::message>>
  {
    Visibility vis = Visibility::Private;

    if (id == "pub" || id == "priv" || id == "prot" || id == "crate" || id == "group") {
      if (id == "pub")   vis = Visibility::Public;
      ef (id == "priv")  vis = Visibility::Private;
      ef (id == "prot")  vis = Visibility::Protected;
      ef (id == "crate") vis = Visibility::Crate;
      ef (id == "group") vis = Visibility::Group;

      if (!(visflag & vis))
        return std::unexpected(errors::VisibilitySettingNApplicableInContext(w, id));

      auto Nx = Lex();
      Require(Nx);

      id = Nx->str();
      w  = Nx.value();
    }

    if (id == "namespace") if_error(read_NameSpace(self))
    ef (id == "alias")     if_error(read_AliasDecl(self))
    ef (id == "var")       if_error(read_VarDecl(self))
    ef (id == "type")      if_error(read_TypeDecl(self))
    ef (id == "fun")       if_error(read_FuncDecl(self))
    ef (id == "struct")    if_error(read_StructDecl(self))
    ef (id == "enum")      if_error(read_EnumDecl(self))
    ef (id == "set")       if_error(read_SetDecl(self))
    ef (id == "mod")       if_error(read_ModDecl(self))
    else
      print(errors::UnknownKeyword(w, id))
        
    return {};
  }

  fun frontend::read_ModDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto Name = Lex();
    Require(Name);
    expected(Lex(), ";");

    std::string mod_name = Name->str();

    std::filesystem::path curr_path(m_fpath);
    std::filesystem::path base_dir = curr_path.parent_path();

    std::string target_file1 = (base_dir / (mod_name + ".qw")).string();
    std::string target_file2 = (base_dir / mod_name / "mod.qw").string();

    std::string target_file = std::filesystem::exists(target_file1) ? target_file1 : target_file2;
    if (!std::filesystem::exists(target_file)) {
      return errors::ModuleNotFound(*Name, mod_name);
    }

    auto sub_mod = ctx->make_module(mod_name, target_file);
    auto sub_ns = decls::Decl::make_NameSpace(ctx, parent, mod_name, *Name);

    frontend sub_front(sub_mod);
    auto err = sub_front.read_File(sub_ns);
    if (!err) return err;

    return {};
  }

  fun frontend::read_TypeDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto Name = Lex();
    Require(Name);

    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);

    expected(Lex(), "=");

    auto Type = read_Type(self, true);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;

    expected(Lex(), ";");
    return {};
  }

  fun frontend::read_FuncDecl(decls::Decl *parent) -> std::expected<decls::Decl*, uptr<diagnostic::message>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    // FuncDecl
    auto self = decls::Decl::make_Func(ctx, parent, Name->str(), *Name, nil);

    expected(Lex(), "(");

    std::vector<types::FieldType> pars;

    // Params
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == ")") break;
      else
        LexStore(ID);

      // Name
      std::vector<std::string> Names;
      l_re:
      auto NameArg = Lex();
      Require(NameArg);
      Names.push_back(NameArg->str());

      // Colon
      auto Colon = Lex();
      Require(Colon);

      if (Colon->view() == ",") goto l_re;
      ef (Colon->view() == ":");
      else
        return errors::UnexpectedIdentifier(*Colon, Colon->str());

      // Type
      auto Type = read_Type(self, true);
      val_error(Type);

      // End
      auto End = Lex();
      Require(End);

      if (End->view() == ";");
      ef (End->view() == ")") LexStore(End);
      else return errors::UnexpectedIdentifier(*End, End->str());

      // Add
      for (auto &X: Names) {
        pars.push_back({ X, *Type });
      }
    }

    // Return
    auto Ret = Lex();
    Require(Ret)

    types::Type *retType = ctx->void_t();
    
    if (Ret->view() == "->") {
      auto Type = read_Type(self, true);
      val_error(Type);
      retType = *Type;

      Ret = Lex();
      Require(Ret);
    }

    auto FType = types::Type::make_Func(ctx, pars, retType);
    self->as<decls::FuncDecl>()->funcType = FType;

    // Code & Decl
    if (Ret->view() == ";") return self;
    ef (Ret->view() == "{") {
      auto block_ret = read_CodeBlock(self);
      if (!block_ret) return std::unexpected(std::move(block_ret.error()));
      return self;
    }
    else
      return errors::ExpectedIdentifierBut2(*Ret, Ret->str(), ";", "{");
  }

  fun frontend::read_AliasDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    // Assign
    expected(Lex(), "=");

    auto Target = Lex();
    Require(Target);
    auto Decl = exprs::Expr::make_Nick(ctx, parent, {Target->str()}, *Target);

    // Create
    auto self = decls::Decl::make_Alias(ctx, parent, Name->str(), Decl, *Name);

    // End
    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_VarDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    l_re:
    // Name
    std::vector<word> Names;
    auto Name = Lex();
    Require(Name);
    Names.push_back(*Name);

    // Colon
    auto Colon = Lex();
    Require(Colon);

    if (Colon->view() == ":");
    ef (Colon->view() == ",") goto l_re;
    else
      return errors ::ExpectedIdentifierBut2(*Colon, Colon->str(), ":", ",");

    auto self = decls::Decl::make_Var(ctx, parent, Names[0].str(), Names[0]);

    // Type
    auto Type = read_Type(self, true);
    val_error(Type);

    // Value
    auto Assi = Lex();
    Require(Assi);

    if (Assi->view() == "=") {
      if (Names.size() != 1)
        return std::unexpected(errors::OnlyOneVariableCanBeInitialized(*Assi, Assi->str()));

      auto Expr = read_Expr(parent, Precedence::Lowest);
      val_error(Expr);

      decls::Decl::make_Var(ctx, parent, Name->str(), *Name, *Type, Visibility::Private, *Expr);
    }
    else {
      LexStore(Assi);

      for (int i = 1; i < Names.size(); i++)
        decls::Decl::make_Var(ctx, parent, Names[0].str(), Names[0], *Type);
    }

    // End
    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_StructDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    // Create Self
    auto Name = Lex();
    Require(Name);

    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);

    auto Type = read_StructType(self, false);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;

    return {};
  }

  fun frontend::read_EnumDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto Name = Lex();
    Require(Name);
    
    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);

    auto ColonOpt = Lex();
    types::Type *baseType = nullptr;
    if (ColonOpt && ColonOpt->view() == ":") {
      auto parsedType = read_Type(parent, true);
      val_error(parsedType);
      baseType = *parsedType;
      auto Bracket = Lex();
      expected(Bracket, "{");
    }
    ef (ColonOpt) {
      if (ColonOpt->view() != "{") {
        return errors::UnexpectedIdentifier(*ColonOpt, "{");
      }
    }
    else {
      return errors::UnexpectedIdentifier(*Name, "Expected { or :");
    }

    auto enumDecl = decls::Decl::make_Enum(ctx, self, "", *Name, Visibility::Public);
    
    std::vector<types::FieldCons> vals;
    std::vector<types::FieldType> typs;
    
    i128 next_val = 1;
    
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else LexStore(ID);

      // Name
      auto EnumConst = Lex();
      Require_Word(EnumConst, continue);
      
      i128 current_val = next_val;

      auto AssignOpt = Lex();
      if (AssignOpt && AssignOpt->view() == "=") {
         auto val_expr = read_Expr(parent, Precedence::Lowest);
         val_error(val_expr);
         if (!(*val_expr)->is<exprs::IntegerLiteral>()) {
            return errors::UnexpectedIdentifier(*AssignOpt, "enum assigned value must be an integer literal");
         }
         auto lit = (*val_expr)->as<exprs::IntegerLiteral>();
         if (std::holds_alternative<u128>(lit->val)) current_val = (i128)std::get<u128>(lit->val);
         else current_val = std::get<i128>(lit->val);
      } else if (AssignOpt) {
         LexStore(AssignOpt);
      }

      vals.push_back({EnumConst->str(), current_val});
      next_val = current_val + 1;

      auto Comma = Lex();
      if (Comma && Comma->view() == ",") {
        continue;
      } else if (Comma && Comma->view() == "}") {
        LexStore(Comma);
      } else if (Comma) {
        return errors::UnexpectedIdentifier(*Comma, Comma->str());
      }
    }

    auto enumType = types::Type::make_Enum(ctx, vals, typs, enumDecl->as<decls::EnumDecl>(), baseType);
    self->as<decls::TypeDecl>()->type = enumType;

    return {};
  }

  static u128 next_pow2(u128 n) {
      if (n == 0) return 1;
      u128 p = 1;
      while (p <= n) {
          p <<= 1;
      }
      return p;
  }

  fun frontend::read_SetDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto Name = Lex();
    Require(Name);
    
    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);

    auto ColonOpt = Lex();
    types::Type *baseType = nullptr;
    if (ColonOpt && ColonOpt->view() == ":") {
      auto parsedType = read_Type(parent, true);
      val_error(parsedType);
      baseType = *parsedType;
      auto Bracket = Lex();
      expected(Bracket, "{");
    }
    ef (ColonOpt) {
      if (ColonOpt->view() != "{") {
        return errors::UnexpectedIdentifier(*ColonOpt, "{");
      }
    }
    else {
      return errors::UnexpectedIdentifier(*Name, "Expected { or :");
    }

    auto setDecl = decls::Decl::make_Set(ctx, self, "", *Name, Visibility::Public);
    
    std::vector<types::FieldCons> vals;
    std::vector<types::FieldType> typs;
    
    i128 next_val = 1;
    
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else LexStore(ID);

      // Name
      auto SetConst = Lex();
      Require_Word(SetConst, continue);
      
      i128 current_val = next_val;

      auto AssignOpt = Lex();
      if (AssignOpt && AssignOpt->view() == "=") {
         auto val_expr = read_Expr(parent, Precedence::Lowest);
         val_error(val_expr);
         if (!(*val_expr)->is<exprs::IntegerLiteral>()) {
            return errors::UnexpectedIdentifier(*AssignOpt, "set assigned value must be an integer literal");
         }
         auto lit = (*val_expr)->as<exprs::IntegerLiteral>();
         if (std::holds_alternative<u128>(lit->val)) current_val = (i128)std::get<u128>(lit->val);
         else current_val = std::get<i128>(lit->val);
      } else if (AssignOpt) {
         LexStore(AssignOpt);
      }

      vals.push_back({SetConst->str(), current_val});
      next_val = (i128)next_pow2((u128)current_val);

      auto Comma = Lex();
      if (Comma && Comma->view() == ",") {
        continue;
      } else if (Comma && Comma->view() == "}") {
        LexStore(Comma);
      } else if (Comma) {
        return errors::UnexpectedIdentifier(*Comma, Comma->str());
      }
    }

    auto setType = types::Type::make_Set(ctx, vals, typs, setDecl->as<decls::SetDecl>(), baseType);
    self->as<decls::TypeDecl>()->type = setType;

    return {};
  }

  fun frontend::read_StructFuncDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto fndecl = read_FuncDecl(parent);
    if_except_ref(fndecl);

    auto fdecl     = (*fndecl)->as<decls::FuncDecl>();
    auto old_ftype = fdecl->funcType->as<types::FuncType>();

    std::vector<types::FieldType> new_pars = old_ftype->pars;

    new_pars.insert(new_pars.begin(), {"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    fdecl->funcType = types::Type::make_Func(ctx, new_pars, old_ftype->ret);

    return {};
  }

  fun frontend::read_StructConstructorDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto self = decls::Decl::make_Constructor(ctx, parent, *LexLast(), nil, vis);

    expected(Lex(), "(");

    std::vector<types::FieldType> pars;
    pars.push_back({"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    // Params
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == ")") break;
      else
        LexStore(ID);

      // Name
      std::vector<std::string> Names;
      l_re:
      auto NameArg = Lex();
      Require(NameArg);
      Names.push_back(NameArg->str());

      // Colon
      auto Colon = Lex();
      Require(Colon);

      if (Colon->view() == ",") goto l_re;
      ef (Colon->view() == ":");
      else
        return errors::UnexpectedIdentifier(*Colon, Colon->str());

      // Type
      auto Type = read_Type(self, true);
      val_error(Type);

      // End
      auto End = Lex();
      Require(End);

      if (End->view() == ";");
      ef (End->view() == ")") LexStore(End);
      else return errors::UnexpectedIdentifier(*End, End->str());

      // Add
      for (auto &X: Names) {
        pars.push_back({ X, *Type });
      }
    }

    auto FType = types::Type::make_Func(ctx, pars, ctx->void_t());
    self->as<decls::ConstructorDecl>()->funcType = FType;

    auto Ret = Lex();
    Require(Ret);

    // Initializers
    if (Ret->view() == ":") {
      while (true) {
        auto Mem = Lex();
        Require(Mem);
        std::string mem_name = Mem->str();

        expected(Lex(), "(");
        auto InitExpr = read_Expr(self, Precedence::Lowest);
        val_error(InitExpr);
        expected(Lex(), ")");

        self->as<decls::ConstructorDecl>()->inits.push_back({mem_name, *InitExpr});

        auto Comma = Lex();
        Require(Comma);
        if (Comma->view() == ",") continue;
        else { LexStore(Comma); break; }
      }
      Ret = Lex();
      Require(Ret);
    }

    // Code & Decl
    if (Ret->view() == ";") return {};
    ef (Ret->view() == "{") {
      auto block_ret = read_CodeBlock(self);
      if (!block_ret) return std::unexpected(std::move(block_ret.error()));
      return {};
    }
    else
      return errors::ExpectedIdentifierBut2(*Ret, Ret->str(), ";", "{");
  }

  fun frontend::read_CodeBlock(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>
  {
    auto self = stmts::Stmt::make_CodeBlock(ctx, parent, *LexLast());

    // Read
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      ef (ID->view() == "ret") if_error(read_ReturnStmt(self))
      ef (ID->view() == "var") if_error(read_VarStmt(self))
      ef (ID->view() == "let") if_error(read_LetStmt(self))
      ef (ID->view() == "if") if_error(read_IfStmt(self))
      ef (ID->view() == "while") if_error(read_WhileStmt(self))
      ef (ID->view() == "break") {
        stmts::Stmt::make_Break(ctx, self, *ID);
        expected(Lex(), ";");
      }
      ef (ID->view() == "continue") {
        stmts::Stmt::make_Continue(ctx, self, *ID);
        expected(Lex(), ";");
      }
      else {
        LexStore(ID);
        auto expr = read_Expr(self, Precedence::Lowest);
        val_error(expr);
        stmts::Stmt::make_ExprStmt(ctx, self, *expr, *ID);
        expected(Lex(), ";");
      }
    }

    return self;
  }

  fun frontend::read_IfStmt(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>
  {
    word pos = *LexLast();
    expected(Lex(), "(");
    auto cond = read_Expr(parent, Precedence::Lowest);
    val_error(cond);
    expected(Lex(), ")");

    expected(Lex(), "{");
    
    auto if_stmt = stmts::Stmt::make_IfStmt(ctx, parent, pos, *cond, nullptr, nullptr);
    
    auto then_ret = read_CodeBlock(if_stmt);
    if (!then_ret) return std::unexpected(std::move(then_ret.error()));
    if_stmt->as<stmts::IfStmt>()->then_block = *then_ret;

    auto nxt = Lex();
    if (!nxt) return {};

    if (nxt->view() == "else") {
      auto nxt2 = Lex();
      Require(nxt2);

      if (nxt2->view() == "if") {
        auto elif_ret = read_IfStmt(if_stmt);
        if (!elif_ret) return std::unexpected(std::move(elif_ret.error()));
        if_stmt->as<stmts::IfStmt>()->else_block = *elif_ret;
      }
      ef (nxt2->view() == "{") {
        auto else_ret = read_CodeBlock(if_stmt);
        if (!else_ret) return std::unexpected(std::move(else_ret.error()));
        if_stmt->as<stmts::IfStmt>()->else_block = *else_ret;
      }
      else return errors::ExpectedIdentifierBut2(*nxt2, nxt2->str(), "if", "{");
    }
    ef (nxt->view() == "ef") {
      auto elif_ret = read_IfStmt(if_stmt);
      if (!elif_ret) return std::unexpected(std::move(elif_ret.error()));
      if_stmt->as<stmts::IfStmt>()->else_block = *elif_ret;
    }
    else {
      LexStore(nxt);
    }

    return if_stmt;
  }

  fun frontend::read_WhileStmt(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>>
  {
    word pos = *LexLast();
    expected(Lex(), "(");
    auto cond = read_Expr(parent, Precedence::Lowest);
    val_error(cond);
    expected(Lex(), ")");

    auto while_stmt = stmts::Stmt::make_WhileStmt(ctx, parent, pos, *cond, nullptr);

    expected(Lex(), "{");
    auto body_ret = read_CodeBlock(while_stmt);
    if (!body_ret) return std::unexpected(std::move(body_ret.error()));
    while_stmt->as<stmts::WhileStmt>()->body = *body_ret;

    return while_stmt;
  }

  fun frontend::read_VarStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    std::vector<word> Vars;

    // Name
    l_re:
    auto Name = Lex();
    Require(Name);
    Vars.push_back(*Name);

    auto Colon = Lex();
    Require(Colon);

    if (Colon->view() == ",") goto l_re;
    ef (Colon->view() == ":");
    else
      return errors::ExpectedIdentifierBut2(*Colon, Colon->str(), ",", ":");

    // Type
    auto Type = read_Type(parent, true);
    val_error(Type);

    // Value
    auto Assi = Lex();
    Require(Assi);

    if (Assi->view() == "=") {
      if (Vars.size() != 1)
        return std::unexpected(errors::OnlyOneVariableCanBeInitialized(*Assi, Assi->str()));

      auto Expr = read_Expr(parent, Precedence::Lowest);
      val_error(Expr);

      stmts::Stmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name, *Expr, Assi);
    }
    else {
      LexStore(Assi);

      for (auto &v : Vars)
        stmts::Stmt::make_CodeVar(ctx, parent, v.str(), *Type, v);
    }

    // End
    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_LetStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    std::vector<word> Vars;

    // Name
    l_re:
    auto Name = Lex();
    Require(Name);
    Vars.push_back(*Name);

    auto Colon = Lex();
    Require(Colon);

    if (Colon->view() == ",") goto l_re;
    ef (Colon->view() == ":");
    else
      expected(Colon, ",\", \":");

    // Type
    auto Type = read_Type(parent, true);
    val_error(Type);

    // Value
    auto Assi = Lex();
    Require(Assi);

    if (Assi->view() == "=") {
      if (Vars.size() != 1)
        return std::unexpected(errors::OnlyOneVariableCanBeInitialized(*Assi, Assi->str()));

      auto Expr = read_Expr(parent, Precedence::Lowest);
      val_error(Expr);

      stmts::Stmt::make_CodeVar(ctx, parent, Name->str(), *Type, *Name, *Expr, Assi);
    }
    else {
      LexStore(Assi);

      for (auto &v : Vars)
        stmts::Stmt::make_CodeVar(ctx, parent, v.str(), *Type, v);
    }

    // End
    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_ReturnStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto Pos = LexLast();

    auto Expr = read_Expr(parent, Precedence::Lowest);
    val_error(Expr);

    auto self = stmts::Stmt::make_Return(ctx, parent, *Pos, *Expr);

    // End
    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_Expr(identy *parent, Precedence min_prec) -> std::expected<exprs::Expr *, uptr<diagnostic::message>>
  {
    exprs::Expr *ret{};
    auto ID = Lex();
    Require(ID);

    if (ID->view() == "@" || ID->view() == "+" || ID->view() == "-" || ID->view() == "!" || ID->view() == "~") {
      exprs::UnaryOpEnum kind;
      if (ID->view() == "@") kind = exprs::UnaryOpEnum::AddrOf;
      ef (ID->view() == "+") kind = exprs::UnaryOpEnum::Plus;
      ef (ID->view() == "-") kind = exprs::UnaryOpEnum::Minus;
      ef (ID->view() == "!") kind = exprs::UnaryOpEnum::LNot;
      ef (ID->view() == "~") kind = exprs::UnaryOpEnum::BitNot;

      auto sub = read_Expr(parent, Precedence::Unary);
      val_error(sub);
      ret = exprs::Expr::make_UnaryOp(ctx, parent, kind, *sub, *ID);
    }
    ef (ID->view() == "true" || ID->view() == "false") {
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
      if (ID->view().size() == 2) return errors::EmptyCharacterConstant(*ID);
      ef (ID->view().size() > 3)  return errors::CharacterConstantTooLong(*ID, (std::string)ID->view().substr(1, ID->view().size() - 2));

      ret = exprs::Expr::make_CharLiteral(ctx, parent, ID->view()[1], *ID);
    }
    ef (ID->view()[0] == '\"') {
      ret = exprs::Expr::make_StringLiteral(
        ctx, parent, 
        std::string(ID->view().substr(1, ID->view().size() -2)),
        *ID
      );
    }
    ef (isWord(ID->view()[0])) {
      ret = exprs::Expr::make_Nick(ctx, parent, {ID->str()}, *ID);
    }
    ef (ID->view() == "(") {
      auto expr = read_Expr(parent, Precedence::Lowest);
      val_error(expr);
      ret = *expr;
      expected(Lex(), ")");
    }
    else
      diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());

    
    // POSTFIX
    while (true) {
      auto Op = Lex();
      if (!Op)
        break;

      if (Op->view() == "." || Op->view() == "::") {
        auto MemName = Lex();
        Require(MemName);

        auto kind = Op->view() == "." ? exprs::MemberOpEnum::Member : exprs::MemberOpEnum::NameS;
        
        if (kind == exprs::MemberOpEnum::NameS && ret->is<exprs::NickExpr>()) {
          ret->as<exprs::NickExpr>()->unresolved.push_back(MemName->str());
          continue;
        }

        auto MemExpr = exprs::Expr::make_Nick(ctx, parent, { MemName->str() }, *MemName);
        ret          = exprs::Expr::make_MemberOp(ctx, parent, kind, ret, MemExpr, *Op);
      }
      ef (Op->view() == "(" || Op->view() == "[") {
        auto kind = Op->view() == "(" ? exprs::PostfixOpEnum::Call : exprs::PostfixOpEnum::Array;
        auto clsb = Op->view() == "(" ? ")" : "]";

        std::vector<exprs::Expr *> ops;
        auto Next = Lex();
        Require(Next);

        while (Next->view() != clsb) {
          LexStore(Next);
          auto ex = read_Expr(parent, Precedence::Lowest);
          if_except_ref(ex);
          ops.push_back(*ex);

          Next = Lex();
          Require(Next);
          if (Next->view() == ",") {
            Next = Lex();
            Require(Next);
          }
        }
        ret = exprs::Expr::make_PostfixOp(ctx, parent, kind, ret, ops, *Next);
      }
      ef (Op->view() == "?") {
        ret = exprs::Expr::make_PostfixOp(ctx, parent, exprs::PostfixOpEnum::Deref, ret, {}, *Op);
      }
      ef (Op->view() == "<") {
        u0 old_off = Off; // save position
        
        std::vector<types::Type *> generic_args;
        bool success = true;
        
        while (true) {
          auto type = read_Type(parent, true); // attempt to read type
          if (!type) { // Not a type!
            success = false;
            break;
          }
          generic_args.push_back(*type);
          
          auto Next = Lex();
          if (!Next) { success = false; break; }
          
          if (Next->view() == ">") {
            break; // Finished successfully
          } ef (Next->view() == ",") {
            continue; // Next arg
          } else {
            success = false; // Expected ',' or '>'
            break;
          }
        }
        
        if (success) {
          ret = exprs::Expr::make_GenericOp(ctx, parent, ret, std::move(generic_args), *Op);
        } else {
          // Rollback to `<`
          Off = old_off;
          m_lexStore = std::nullopt; // clear lex cache
          LexStore(Op);
          break; // Let binary operator handle `<`
        }
      }
      else {
        LexStore(Op);
        break;
      }
    }

    // INFIX
    while (true) {
      auto Op = Lex();
      if (!Op)
        break;

      Precedence op_prec = Precedence::Lowest;
      exprs::BinaryOpEnum kind;

      if (Op->view() == "=") {
        op_prec = Precedence::Assign;
        kind    = exprs::BinaryOpEnum::Assign;
      }
      ef (Op->view() == "+=" || Op->view() == "-=" || Op->view() == "*=" || Op->view() == "/=" || Op->view() == "%=" || Op->view() == "&=" || Op->view() == "|=" || Op->view() == "^=" || Op->view() == "<<=" || Op->view() == ">>=") {
        op_prec = Precedence::Assign;
        if (Op->view() == "+=") kind = exprs::BinaryOpEnum::AddAssign;
        ef (Op->view() == "-=") kind = exprs::BinaryOpEnum::SubAssign;
        ef (Op->view() == "*=") kind = exprs::BinaryOpEnum::MulAssign;
        ef (Op->view() == "/=") kind = exprs::BinaryOpEnum::DivAssign;
        ef (Op->view() == "%=") kind = exprs::BinaryOpEnum::RemAssign;
        ef (Op->view() == "&=") kind = exprs::BinaryOpEnum::BitAndAssign;
        ef (Op->view() == "|=") kind = exprs::BinaryOpEnum::BitOrAssign;
        ef (Op->view() == "^=") kind = exprs::BinaryOpEnum::BitXorAssign;
        ef (Op->view() == "<<=") kind = exprs::BinaryOpEnum::ShlAssign;
        ef (Op->view() == ">>=") kind = exprs::BinaryOpEnum::ShrAssign;
      }
      ef(Op->view() == "==" || Op->view() == "!=") {
        op_prec = Precedence::Eq;
        kind    = Op->view() == "==" ? exprs::BinaryOpEnum::Eq : exprs::BinaryOpEnum::NEq;
      }
      ef(Op->view() == "<" || Op->view() == ">" || Op->view() == "<=" || Op->view() == ">=") {
        op_prec = Precedence::Rel;
        if (Op->view() == "<") kind = exprs::BinaryOpEnum::Lt;
        ef (Op->view() == ">") kind = exprs::BinaryOpEnum::Gt;
        ef (Op->view() == "<=") kind = exprs::BinaryOpEnum::LEq;
        ef (Op->view() == ">=") kind = exprs::BinaryOpEnum::GEq;
      }
      ef(Op->view() == "+" || Op->view() == "-") {
        op_prec = Precedence::Add;
        kind    = Op->view() == "+" ? exprs::BinaryOpEnum::Add : exprs::BinaryOpEnum::Sub;
      }
      ef(Op->view() == "*" || Op->view() == "/" || Op->view() == "%") {
        op_prec = Precedence::Mul;
        kind    = Op->view() == "*" ? exprs::BinaryOpEnum::Mul : Op->view() == "/" ? exprs::BinaryOpEnum::Div : exprs::BinaryOpEnum::Rem;
      }
      ef(Op->view() == "&") {
        op_prec = Precedence::BitAnd;
        kind    = exprs::BinaryOpEnum::BitAnd;
      }
      ef(Op->view() == "|") {
        op_prec = Precedence::BitOr;
        kind    = exprs::BinaryOpEnum::BitOr;
      }
      ef(Op->view() == "^") {
        op_prec = Precedence::BitXor;
        kind    = exprs::BinaryOpEnum::BitXor;
      }
      ef(Op->view() == "<<" || Op->view() == ">>") {
        op_prec = Precedence::Shift;
        kind    = Op->view() == "<<" ? exprs::BinaryOpEnum::Shl : exprs::BinaryOpEnum::LShr; // LShr or AShr will be determined in sema.cc
      }
      else {
        LexStore(Op);
        break;
      }

      if (op_prec < min_prec) {
        LexStore(Op);
        break;
      }

      Precedence next_prec = (kind == exprs::BinaryOpEnum::Assign) ? op_prec : op_prec + 1;
      auto r2              = read_Expr(parent, next_prec);
      val_error(r2);

      ret = exprs::Expr::make_BinaryOp(ctx, parent, kind, ret, *r2, *Op);
    }

    return ret;
  }

  fun frontend::read_Type(identy *parent, bool indecl) -> std::expected<types::Type *, uptr<diagnostic::message>>
  {
    // Select
    auto ID = Lex();
    Require(ID);

    std::expected<types::Type*, uptr<diagnostic::message>> ret;

    if (ID->view() == "struct") {
      ret = read_StructType(parent, indecl);
      goto l_fin;
    }
    if (ID->view() == "fun") {
      ret = read_FuncType(parent, indecl);
      goto l_fin;
    }

    else {
      if (indecl) {
        auto Nick = types::Type::make_Nick(ctx, { ID->str() });

        l_re:
        auto S = Lex();
        Require(S);

        if (S->view() == "::") {
          auto N = Lex();
          Require_Word(N, {
            ret = Nick;
            goto l_fin;
          });

          Nick->as<types::NickType>()->unresolved.push_back(N->str());
          goto l_re;
        }
        else
          LexStore(S);

        ret = Nick;
        goto l_fin;
      }
      else
        return errors::UnknownKeyword(*ID, ID->str());
    }

    l_fin:
    if (ret.has_value()) {
      types::Type *baseType = *ret;

      while (true) {
        auto Op = Lex();
        if (!Op)
          break;

        if (Op->view() == "^") baseType = types::Type::make_Pointer(ctx, baseType);
        ef (Op->view() == "&") baseType = types::Type::make_Reference(ctx, baseType);
        ef (Op->view() == "[") {
          auto SizeOrClose = Lex();
          Require(SizeOrClose);

          if (SizeOrClose->view() == "]") {
            baseType = types::Type::make_ZArray(ctx, baseType);
          }
          else {
            u32 size = std::stoul(SizeOrClose->str());

            auto Close = Lex();
            Require(Close);

            baseType = types::Type::make_PArray(ctx, baseType, size);
          }
        }
        else {
          LexStore(Op);
          break;
        }
      }

      ret = baseType;
    }

    return ret;
  }

  fun frontend::read_StructType(identy *parent, bool indecl) -> std::expected<types::Type *, uptr<diagnostic::message>>
  {
    auto Bracket = Lex();
    expected(Bracket, "{");

    Visibility visdef = Visibility::Public;
    std::vector<types::FieldType> vars;

    decls::Decl *recDecl = nil;
    if (parent && parent->type() == IdentyEnum::Decl) {
      auto pDecl = static_cast<decls::Decl *>(parent);
      recDecl    = decls::Decl::make_Struct(ctx, pDecl, "", pDecl->pos(), Visibility::Public);
    }

    auto selfType = types::Type::make_Struct(ctx, {}, {}, recDecl ? recDecl->as<decls::StructDecl>() : nil);

    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else
        LexStore(ID);

      auto visone = visdef;

      // Visibility
      auto Vis = Lex();
      if (auto id = Vis->view(); id == "pub" || id == "priv" || id == "prot" || id == "crate" || id == "group") {
        auto vis = visone;

        if (id == "pub")   vis = Visibility::Public;
        ef (id == "priv")  vis = Visibility::Private;
        ef (id == "prot")  vis = Visibility::Protected;
        ef (id == "crate") vis = Visibility::Crate;
        ef (id == "group") vis = Visibility::Group;

        auto Colon = Lex();
        Require(Colon);

        if (Colon->view() == ":") {
          visdef = vis;
          continue;
        }
        else {
          visone = vis;
          LexStore(Colon);
        }
      }
      else {
        LexStore(Vis);
      }

      // Is Function
      auto Kwd = Lex();
      Require(Kwd);
      if (Kwd->view() == "fun") {
        if_error(read_StructFuncDecl(recDecl, selfType, visone));
        continue;
      }
      ef (Kwd->view() == "init") {
        if_error(read_StructConstructorDecl(recDecl, selfType, visone));
        continue;
      }
      else {
        LexStore(Kwd);
      }

      // Name
      std::vector<std::string> Names;
    l_re:
      auto Name = Lex();
      Require_Word(Name, continue);
      Names.push_back(Name->str());

      // , :
      auto Colon = Lex();
      Require(Colon);
      if (Colon->view() == ",")
        goto l_re;
      ef(Colon->view() == ":");
      else return errors::ExpectedIdentifierBut2(*Colon, Colon->str(), ",", ":");

      // Type
      auto Type = read_Type(parent, true);
      val_error(Type);

      // End
      expected(Lex(), ";");

      // Add
      for (auto &X : Names) {
        vars.push_back({ X, *Type, visone });
      }
    }

    selfType->as<types::StructType>()->vars = vars;

    return selfType;
  }

  fun frontend::read_FuncType(identy *parent, bool indecl) -> std::expected<types::Type *, uptr<diagnostic::message>>
  {
    auto Bracket = Lex();
    expected(Bracket, "(");

    std::vector<types::FieldType> pars;

    // Params
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == ")") break;
      else
        LexStore(ID);

      // Name
      std::vector<std::string> Names;
      l_re:
      auto NameArg = Lex();
      Require(NameArg);
      Names.push_back(NameArg->str());

      // Colon
      auto Colon = Lex();
      Require(Colon);

      if (Colon->view() == ",") goto l_re;
      ef (Colon->view() == ":");
      else
        return errors::UnexpectedIdentifier(*Colon, Colon->str());

      // Type
      auto Type = read_Type(parent, true);
      val_error(Type);

      // End
      auto End = Lex();
      Require(End);

      if (End->view() == ";");
      ef (End->view() == ")") LexStore(End);
      else
        return errors::UnexpectedIdentifier(*End, End->str());

      // Add
      for (auto &X : Names) {
        pars.push_back({ X, *Type });
      }
    }

    // Return
    auto Ret = Lex();
    Require(Ret)

    types::Type *retType = ctx->void_t();
    
    if (Ret->view() == "->") {
      auto Type = read_Type(parent, true);
      val_error(Type);
      retType = *Type;

      Ret = Lex();
      Require(Ret);
    }
    else
      LexStore(Ret);

    // FuncDecl
    auto self = types::Type::make_Func(ctx, pars, retType);

    return self;
  }

}
