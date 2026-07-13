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
#include "qw/lexer/lexer.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/types.hh"
#include <expected>
#include <fcntl.h>
#include <filesystem>
#include <iostream>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if

namespace fs = std::filesystem;

#define Require(X) \
  if (!X) \
    return fatals::FileEndedButContextNotFinished();

#define Require_Word(X, C) \
  { \
    Require(X); \
    if (lex.kind(X.view()[0]) != CharKind::Word) { \
      auto E = errors::ExpectedAWord(X, X.str()); \
      sum.add(E.error().get()); \
      std::cerr << E.error(); \
      C; \
    } \
  }

#define expected(LEX, T) \
  { \
    auto X = LEX; \
    Require(X) ef(X.view() != T) return errors::ExpectedIdentifierBut(X, X.str(), T); \
  }

#define expected2(LEX, T, T2) \
  { \
    auto X = LEX; \
    Require(X) ef(X.view() != T || X.view() != T2) return errors::ExpectedIdentifierBut2(X, X.str(), T, T2); \
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
  
  fun DeclParser::read_Decl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto ID = lex();
    Require(ID);

    if (ID.view() == "alias")     if_error(read_AliasDecl(parent))
    ef (ID.view() == "var")       if_error(read_VarDecl(parent))
    ef (ID.view() == "type")      if_error(read_TypeDecl(parent))
    ef (ID.view() == "fun")       if_error(read_FuncDecl(parent))
    ef (ID.view() == "struct")    if_error(read_StructDecl(parent))
    ef (ID.view() == "iface")     if_error(read_IFaceDecl(parent))
    ef (ID.view() == "enum")      if_error(read_EnumDecl(parent))
    ef (ID.view() == "set")       if_error(read_SetDecl(parent))
    ef (ID.view() == "mod")       if_error(read_ModDecl(parent))
    else {
      print(errors::UnknownKeyword(ID, ID.str()))
      // Panic mode recovery
      int brace_depth = 0;
      while (true) {
        auto next = lex();
        if (!next) break;

        if (next.is(WordKind::CurlyBracketBeg)) {
          brace_depth++;
        }
        ef (next.is(WordKind::CurlyBracketEnd)) {
          if (brace_depth > 0) brace_depth--;
        }

        if (brace_depth == 0) {
          auto v = next.view();
          if (v == "alias" || v == "var" || v == "type" || v == "fun" || 
              v == "struct" || v == "iface" || v == "enum" || v == "set" || v == "mod") {
            lex.store(next);
            break;
          }
        }
      }
    }

    return {};
  }


  fun DeclParser::read_TypeDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);
    expected(lex(), "=");

    auto self = decls::Decl::make_Type(ctx, parent, Name.str(), Name);
    self->attrs() = std::exchange(pctx.meta->attrs(), {});

    auto Type = pctx.type->read_Type(self, true);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;
    (*Type)->owner_ident = self;

    expected(lex(), ";");

    return {};
  }

  fun DeclParser::read_FuncDecl(decls::Decl *parent) -> std::expected<decls::Decl*, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);

    auto self = decls::Decl::make_Func(ctx, parent, Name.str(), Name, nil);
    self->attrs() = std::exchange(pctx.meta->attrs(), {});

    auto generic_ctx = pctx.meta->read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    expected(lex(), "(");

    // Params
    auto parsed_pars = pctx.meta->read_FuncParams(self);
    val_error(parsed_pars);
    std::vector<types::FieldType> pars = *parsed_pars;

    // Return
    auto Ret = lex();
    Require(Ret)

    types::Type *retType = ctx->void_t();
    
    if (Ret.is(WordKind::ArrowRigh)) {
      auto Type = pctx.type->read_Type(self, true);
      val_error(Type);
      retType = *Type;

      Ret = lex();
      Require(Ret);
    }

    auto FType = types::Type::make_Func(ctx, pars, retType);
    self->as<decls::FuncDecl>()->funcType = FType;

    // Code || Decl
    if (Ret.is(WordKind::Semicolon)) return self;
    ef (Ret.is(WordKind::CurlyBracketBeg)) {
      auto block_ret = pctx.stmt->read_CodeBlock(self);
      if (!block_ret)
        return std::unexpected(std::move(block_ret.error()));
      else
        return self;
    }
    else
      return errors::ExpectedIdentifierBut2(Ret, Ret.str(), ";", "{");
  }

  fun DeclParser::read_AliasDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);
    expected(lex(), "=");

    auto Target = lex();
    Require(Target);
    auto Decl = exprs::Expr::make_Nick(ctx, parent, {Target.str()}, Target);

    auto self = decls::Decl::make_Alias(ctx, parent, Name.str(), Decl, Name);
    self->attrs() = std::exchange(pctx.meta->attrs(), {});

    expected(lex(), ";");

    #ifdef _QW_use_verbose_for_frontend
      std::cerr << color::YELLOW << __func__ << " " << self << color::RESET << std::endl;
    #endif

    return {};
  }

  fun DeclParser::read_VarDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<word> Names;
    re:
    auto Name = lex();
    Require(Name);
    Names.push_back(Name);

    auto Colon = lex();
    Require(Colon);

    types::Type *parsedType = nullptr;

    if (Colon.is(WordKind::Assign) || Colon.is(WordKind::Semicolon)) {
      lex.store(Colon);
    }
    else {
      if (Colon.is(WordKind::Colon)) {}
      ef (Colon.is(WordKind::Comma)) goto re;
      else
        return std::unexpected(errors::ExpectedIdentifierBut2(Colon, Colon.str(), ":", ","));

      auto Type = pctx.type->read_Type(parent, true);
      val_error(Type);
      parsedType = *Type;
    }

    auto Assi = lex();
    Require(Assi);

    if (Assi.is(WordKind::Assign)) {
      if (Names.size() != 1)
        return std::unexpected(errors::OnlyOneVariableCanBeInitialized(Assi, Assi.str()));

      auto Expr = pctx.expr->read_Expr(parent, Precedence::Lowest);
      val_error(Expr);

      auto self = decls::Decl::make_Var(ctx, parent, Names[0].str(), Names[0], parsedType, Visibility::Private, *Expr);
      self->attrs() = std::exchange(pctx.meta->attrs(), {});
    }
    else {
      if (!parsedType)
        return std::unexpected(errors::TypeRequiredWithoutAssignment(Assi, Names[0].str()));
      lex.store(Assi);

      for (size_t i = 0; i < Names.size(); i++) {
        auto self = decls::Decl::make_Var(ctx, parent, Names[i].str(), Names[i], parsedType);
        self->attrs() = pctx.meta->attrs();
      }
      pctx.meta->attrs().clear();
    }

    expected(lex(), ";");
    return {};
  }

  fun DeclParser::read_IFaceDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);
    auto self = decls::Decl::make_Type(ctx, parent, Name.str(), Name);
    self->attrs() = std::exchange(pctx.meta->attrs(), {});

    auto generic_ctx = pctx.meta->read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    std::vector<types::Type*> baseTypes;
    std::vector<word> baseTypePos;
    auto ColonOpt = lex();

    if (ColonOpt && ColonOpt.is(WordKind::Colon)) while (true) {
      auto Pk = lex();
      word pos = Pk ? Pk : Name;
      if (Pk) lex.store(Pk);

      auto parsedType = pctx.type->read_Type(parent, true);
      val_error(parsedType);
      baseTypes.push_back(*parsedType);
      baseTypePos.push_back(pos);

      auto CommaOpt = lex();

      if (CommaOpt && CommaOpt.is(WordKind::Comma)) continue;
      ef (CommaOpt) {
        lex.store(CommaOpt);
        break;
      }
      else break;
    }
    ef (ColonOpt) lex.store(ColonOpt);

    auto Type = pctx.type->read_IFaceType(self, false, baseTypes, baseTypePos);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;
    (*Type)->owner_ident = self;

    return {};
  }

  fun DeclParser::read_StructDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);

    auto self = decls::Decl::make_Type(ctx, parent, Name.str(), Name);
    self->attrs() = std::exchange(pctx.meta->attrs(), {});

    auto generic_ctx = pctx.meta->read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    std::vector<types::Type*> baseTypes;
    std::vector<word> baseTypePos;
    auto ColonOpt = lex();

    if (ColonOpt && ColonOpt.is(WordKind::Colon)) while (true) {
      auto Pk = lex();
      word pos = Pk ? Pk : Name;
      if (Pk) lex.store(Pk);

      auto parsedType = pctx.type->read_Type(parent, true);
      val_error(parsedType);
      baseTypes.push_back(*parsedType);
      baseTypePos.push_back(pos);

      auto CommaOpt = lex();
      if (CommaOpt && CommaOpt.is(WordKind::Comma)) continue;
      ef (CommaOpt) {
        lex.store(CommaOpt);
        break;
      }
      else break;
    }
    ef (ColonOpt)
      lex.store(ColonOpt);

    auto Type = pctx.type->read_StructType(self, false, baseTypes, baseTypePos);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;
    (*Type)->owner_ident = self;

    return {};
  }

  fun DeclParser::read_EnumDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);
    
    auto self = decls::Decl::make_Type(ctx, parent, Name.str(), Name);
    self->attrs() = std::exchange(pctx.meta->attrs(), {});

    auto generic_ctx = pctx.meta->read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    auto ColonOpt = lex();
    types::Type *baseType = nullptr;
    word baseTypePos{};

    if (ColonOpt && ColonOpt.is(WordKind::Colon)) {
      auto Pk = lex();
      baseTypePos = Pk ? Pk : Name;
      if (Pk) lex.store(Pk);

      auto parsedType = pctx.type->read_Type(parent, true);
      val_error(parsedType);
      baseType = *parsedType;
      auto Bracket = lex();
      expected(Bracket, "{");
    }
    ef (ColonOpt) {
      if (!ColonOpt.is(WordKind::CurlyBracketBeg))
        return errors::ExpectedIdentifierBut(ColonOpt, ColonOpt.str(), "{");
    }
    else
      return errors::ExpectedIdentifierBut2(Name, Name.str(), "{", ":");

    auto enumDecl = decls::Decl::make_Enum(ctx, self, "", Name, Visibility::Public);
    enumDecl->attrs() = std::exchange(pctx.meta->attrs(), {});
    
    std::vector<types::FieldCons> vals;
    std::vector<types::FieldType> typs;
    
    i64 next_val = 1;
    
    while (true) {
      auto ID = lex();
      Require(ID);

      if (ID.is(WordKind::CurlyBracketEnd)) break;
      else lex.store(ID);

      // Name
      auto EnumConst = lex();
      Require_Word(EnumConst, continue);
      
      i64 current_val = next_val;

      auto AssignOpt = lex();
      if (AssignOpt && AssignOpt.is(WordKind::Assign)) {
        auto val_expr = pctx.expr->read_Expr(parent, Precedence::Lowest);
        val_error(val_expr);
        if (!(*val_expr)->is<exprs::IntegerLiteral>())
          return errors::InvalidConstantValue(AssignOpt, Name.str(), "enum assigned value must be an integer literal");

        auto lit = (*val_expr)->as<exprs::IntegerLiteral>();
        if (std::holds_alternative<u64>(lit->val))
          current_val = (i64)std::get<u64>(lit->val);
        else
          current_val = (i64)std::get<i64>(lit->val);
      }
      ef (AssignOpt) lex.store(AssignOpt);

      vals.push_back({EnumConst.str(), current_val});
      next_val = current_val + 1;

      auto Comma = lex();
      if (Comma && Comma.is(WordKind::Comma)) continue;
      ef (Comma && Comma.is(WordKind::CurlyBracketEnd)) lex.store(Comma);
      ef (Comma)
        return errors::UnexpectedIdentifier(Comma, Comma.str());
    }

    std::string tname = self ? std::string(self->name()) : "enum";
    auto enumType = types::Type::make_Enum(ctx, vals, typs, enumDecl, baseType, baseTypePos, tname);
    self->as<decls::TypeDecl>()->type = enumType;
    enumType->owner_ident = self;

    return {};
  }

  static fun next_pow2(u64 n) -> u64 {
    if (n == 0) return 1;
    u64 p = 1;

    while (p <= n) p <<= 1;
    
    return p;
  }

  fun DeclParser::read_SetDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);
    auto self = decls::Decl::make_Type(ctx, parent, Name.str(), Name);
    self->attrs() = std::exchange(pctx.meta->attrs(), {});

    auto generic_ctx = pctx.meta->read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    auto ColonOpt = lex();
    types::Type *baseType = nullptr;
    word baseTypePos{};

    if (ColonOpt && ColonOpt.is(WordKind::Colon)) {
      auto Pk = lex();
      baseTypePos = Pk ? Pk : Name;
      if (Pk) lex.store(Pk);

      auto parsedType = pctx.type->read_Type(parent, true);
      val_error(parsedType);
      baseType = *parsedType;
      auto Bracket = lex();
      expected(Bracket, "{");
    }
    ef (ColonOpt) {
      if (!ColonOpt.is(WordKind::CurlyBracketBeg))
        return errors::ExpectedIdentifierBut(ColonOpt, ColonOpt.str(), "{");
    }
    else
      return errors::ExpectedIdentifierBut2(Name, Name.str(), "{", ":");

    
    auto setDecl = decls::Decl::make_Set(ctx, self, "", Name, Visibility::Public);
    setDecl->attrs() = std::exchange(pctx.meta->attrs(), {});
    
    std::vector<types::FieldCons> vals;
    std::vector<types::FieldType> typs;
    
    i64 next_val = 1;
    
    while (true) {
      auto ID = lex();
      Require(ID);

      if (ID.is(WordKind::CurlyBracketEnd)) break;
      else lex.store(ID);

      // Name
      auto SetConst = lex();
      Require_Word(SetConst, continue);
      
      i64 current_val = next_val;

      auto AssignOpt = lex();
      if (AssignOpt && AssignOpt.is(WordKind::Assign)) {
          auto val_expr = pctx.expr->read_Expr(parent, Precedence::Lowest);
          val_error(val_expr);
          if (!(*val_expr)->is<exprs::IntegerLiteral>())
            return errors::InvalidConstantValue(AssignOpt, Name.str(), "set assigned value must be an integer literal");
        
          auto lit = (*val_expr)->as<exprs::IntegerLiteral>();
        if (std::holds_alternative<u64>(lit->val))
          current_val = (i64)std::get<u64>(lit->val);
        else
          current_val = (i64)std::get<i64>(lit->val);
      }
      ef (AssignOpt) lex.store(AssignOpt);

      vals.push_back({SetConst.str(), current_val});
      next_val = (i64)next_pow2((u64)current_val);

      auto Comma = lex();
      if (Comma && Comma.is(WordKind::Comma)) continue;
      ef (Comma && Comma.is(WordKind::CurlyBracketEnd)) lex.store(Comma);
      ef (Comma)
        return errors::UnexpectedIdentifier(Comma, Comma.str());
    }

    std::string tname = self ? std::string(self->name()) : "set";
    auto setType = types::Type::make_Set(ctx, vals, typs, setDecl, baseType, baseTypePos, tname);
    self->as<decls::TypeDecl>()->type = setType;
    setType->owner_ident = self;

    return {};
  }

  fun DeclParser::read_ModDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = lex();
    Require(Name);
    expected(lex(), ";");

    std::string mod_name = Name.str();

    fs::path curr_path(pctx.mod->fpath());
    fs::path base_dir = curr_path.parent_path();

    std::string target_file1 = (base_dir / (mod_name + ".qw")).string();
    std::string target_file2 = (base_dir / mod_name / "mod.qw").string();

    std::string target_file = fs::exists(target_file1) ? target_file1 : target_file2;
    if (!fs::exists(target_file)) {
      return errors::ModuleNotFound(Name, mod_name);
    }

    if (ctx->is_parsed(target_file)) {
      return errors::CyclicImport(Name, mod_name);
    }
    ctx->mark_parsed(target_file);

    auto sub_mod = ctx->make_module(mod_name, target_file);
    auto sub_ns = decls::Decl::make_NameSpace(ctx, parent, mod_name, Name);
    sub_ns->attrs() = std::exchange(pctx.meta->attrs(), {});

    frontend sub_front(sub_mod);
    auto err = sub_front.read_File(sub_ns);
    if (!err) return err;

    return {};
  }

  fun DeclParser::read_StructFuncDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>> {
    auto fndecl = read_FuncDecl(parent);
    if_except_ref(fndecl);

    auto fdecl     = (*fndecl)->as<decls::FuncDecl>();
    auto old_ftype = fdecl->funcType->as<types::FuncType>();

    std::vector<types::FieldType> new_pars = old_ftype->pars;

    new_pars.insert(new_pars.begin(), {"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    fdecl->funcType = types::Type::make_Func(ctx, new_pars, old_ftype->ret);

    return {};
  }

  fun DeclParser::read_StructConstructorDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>> {
    auto self = decls::Decl::make_Constructor(ctx, parent, lex.last(), nil, vis);

    expected(lex(), "(");

    std::vector<types::FieldType> pars;
    pars.push_back({"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    // Params
    auto parsed_pars = pctx.meta->read_FuncParams(self);
    val_error(parsed_pars);
    pars.insert(pars.end(), parsed_pars->begin(), parsed_pars->end());

    auto FType = types::Type::make_Func(ctx, pars, ctx->void_t());
    self->as<decls::ConstructorDecl>()->funcType = FType;

    auto Ret = lex();
    Require(Ret);

    // Initializers
    if (Ret.is(WordKind::Colon)) {
      while (true) {
        auto Mem = lex();
        Require(Mem);
        std::string mem_name = Mem.str();

        expected(lex(), "(");
        auto InitExpr = pctx.expr->read_Expr(self, Precedence::Lowest);
        val_error(InitExpr);
        expected(lex(), ")");

        self->as<decls::ConstructorDecl>()->inits.push_back({mem_name, *InitExpr});

        auto Comma = lex();
        Require(Comma);
        if (Comma.is(WordKind::Comma)) {
          continue;
        }
        else {
          lex.store(Comma);
          break;
        }
      }

      Ret = lex();
      Require(Ret);
    }

    // Code & Decl
    if (Ret.is(WordKind::Semicolon)) return {};
    ef (Ret.is(WordKind::CurlyBracketBeg)) {
      auto block_ret = pctx.stmt->read_CodeBlock(self);
      if (!block_ret)
        return std::unexpected(std::move(block_ret.error()));
      else
        return {};
    }
    else
      return errors::ExpectedIdentifierBut2(Ret, Ret.str(), ";", "{");
  }

  fun DeclParser::read_StructDestructorDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>> {
    auto self = decls::Decl::make_Destructor(ctx, parent, lex.last(), nil, vis);

    expected(lex(), "(");

    std::vector<types::FieldType> pars;
    pars.push_back({"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    // Params
    auto parsed_pars = pctx.meta->read_FuncParams(self);
    val_error(parsed_pars);
    pars.insert(pars.end(), parsed_pars->begin(), parsed_pars->end());

    auto FType = types::Type::make_Func(ctx, pars, ctx->void_t());
    self->as<decls::DestructorDecl>()->funcType = FType;

    auto Ret = lex();
    Require(Ret);

    // Code & Decl
    if (Ret.is(WordKind::Semicolon)) return {};
    ef (Ret.is(WordKind::CurlyBracketBeg)) {
      auto block_ret = pctx.stmt->read_CodeBlock(self);
      if (!block_ret)
        return std::unexpected(std::move(block_ret.error()));
      else
        return {};
    }
    else
      return errors::ExpectedIdentifierBut2(Ret, Ret.str(), ";", "{");
  }

}
