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
  static fun unescape_string(std::string_view s) -> std::string {
    std::string res;
    res.reserve(s.size());
    for (size_t i = 0; i < s.size(); ++i) {
      if (s[i] == '\\' && i + 1 < s.size()) {
        ++i;
        switch (s[i]) {
          case 'n': res += '\n'; break;
          case 'r': res += '\r'; break;
          case 't': res += '\t'; break;
          case '0': res += '\0'; break;
          case '\\': res += '\\'; break;
          case '\"': res += '\"'; break;
          case '\'': res += '\''; break;
          default: res += '\\'; res += s[i]; break;
        }
      }
      else {
        res += s[i];
      }
    }
    return res;
  }


  fun frontend::read_File(decls::Decl *self) -> std::expected<void, uptr<diagnostic::message>> {
    while (true) {
      auto ID = Lex();

      if (!ID.has_value()) return {};
      ef (ID->view() == "}") break;
      ef (ID->view() == "![") {
        if_error(read_FileAttributes(self));
      }
      else
        if_except(read_Route(
          ID->str(), ID.value(), self,
          Visibility::Public | Visibility::Private | Visibility::Crate | Visibility::Group
        ));
    }
    
    return {};
  }

  fun frontend::read_Route(std::string id, word w, decls::Decl *self, VisibilityFlag visflag) -> std::expected<void, uptr<diagnostic::message>> {
    if (id == "[[") {
      if_error(read_Attributes());
      auto Nx = Lex();
      Require(Nx);
      id = Nx->str();
      w = Nx.value();
    }

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

    if (id == "alias")     if_error(read_AliasDecl(self))
    ef (id == "var")       if_error(read_VarDecl(self))
    ef (id == "type")      if_error(read_TypeDecl(self))
    ef (id == "fun")       if_error(read_FuncDecl(self))
    ef (id == "struct")    if_error(read_StructDecl(self))
    ef (id == "iface")     if_error(read_IFaceDecl(self))
    ef (id == "enum")      if_error(read_EnumDecl(self))
    ef (id == "set")       if_error(read_SetDecl(self))
    ef (id == "mod")       if_error(read_ModDecl(self))
    else
      print(errors::UnknownKeyword(w, id))
        
    return {};
  }



  fun frontend::read_Attributes() -> std::expected<void, uptr<diagnostic::message>> {
    while (true) {
      auto attr_name = Lex();
      Require(attr_name);

      if (attr_name->view() == "]]") break;

      decls::Attribute attr;
      attr.name = attr_name->str();

      auto next = Lex();
      Require(next);

      if (next->view() == ":") {
        auto attr_val = Lex();
        Require(attr_val);
        attr.value = attr_val->str();

        next = Lex();
        Require(next);
      }

      m_current_attrs.push_back(attr);

      if (next->view() == "]]") break;
      ef (next->view() == ",") continue;
      else
        return errors::ExpectedIdentifierBut2(*next, next->str(), "]]", ",");
    }
    return {};
  }

  fun frontend::read_FileAttributes(decls::Decl *self) -> std::expected<void, uptr<diagnostic::message>> {
    while (true) {
      auto attr_name = Lex();
      Require(attr_name);

      if (attr_name->view() == "]") break;

      decls::Attribute attr;
      attr.name = attr_name->str();

      // "use no rtl" is 3 tokens, let's just collect all words until ] or , as the value?
      // But wait! QW attributes are single identifier or id:value.
      // If the user wants `![use no rtl]`, it parses as `use`, `no`, `rtl`?!
      // Let's just collect everything inside!
      auto next = Lex();
      Require(next);

      while (next->view() != "]" && next->view() != ",") {
        attr.value += (attr.value.empty() ? "" : " ") + next->str();
        next = Lex();
        Require(next);
      }

      self->attrs().push_back(attr);

      if (next->view() == "]") break;
    }
    return {};
  }



  fun frontend::read_TypeDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = Lex();
    Require(Name);
    expected(Lex(), "=");

    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);
    self->attrs() = std::exchange(m_current_attrs, {});

    auto Type = read_Type(self, true);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;
    (*Type)->owner_ident = self;

    expected(Lex(), ";");

    #ifdef _QW_use_verbose_for_frontend
      std::cerr << color::YELLOW << __func__ << " " << self << color::RESET << std::endl;
    #endif

    return {};
  }

  fun frontend::read_FuncParams(identy *parent) -> std::expected<std::vector<types::FieldType>, uptr<diagnostic::message>> {
    std::vector<types::FieldType> pars;

    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == ")") break;
      else
        LexStore(ID);

      // Name
      std::vector<std::string> Names;
      re:
      auto NameArg = Lex();
      Require(NameArg);
      Names.push_back(NameArg->str());

      // Colon
      auto Colon = Lex();
      Require(Colon);

      if (Colon->view() == ",") goto re;
      ef (Colon->view() == ":");
      else
        return errors::ExpectedIdentifierBut(*Colon, Colon->str(), ":");

      // Type
      auto Type = read_Type(parent, true);
      val_error(Type);

      // End
      auto End = Lex();
      Require(End);

      if (End->view() == ";");
      ef (End->view() == ")") LexStore(End);
      else
        return errors::ExpectedIdentifierBut2(*End, End->str(), ";", ")");

      // Push
      for (auto &X: Names) pars.push_back({X, *Type});
    }

    return pars;
  }

  fun frontend::read_FuncDecl(decls::Decl *parent) -> std::expected<decls::Decl*, uptr<diagnostic::message>> {
    auto Name = Lex();
    Require(Name);

    auto self = decls::Decl::make_Func(ctx, parent, Name->str(), *Name, nil);
    self->attrs() = std::exchange(m_current_attrs, {});

    auto generic_ctx = read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    expected(Lex(), "(");

    // Params
    auto parsed_pars = read_FuncParams(self);
    val_error(parsed_pars);
    std::vector<types::FieldType> pars = *parsed_pars;

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

    // Code || Decl
    if (Ret->view() == ";") return self;
    ef (Ret->view() == "{") {
      auto block_ret = read_CodeBlock(self);
      if (!block_ret)
        return std::unexpected(std::move(block_ret.error()));
      else
        return self;
    }
    else
      return errors::ExpectedIdentifierBut2(*Ret, Ret->str(), ";", "{");
  }

  fun frontend::read_AliasDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = Lex();
    Require(Name);
    expected(Lex(), "=");

    auto Target = Lex();
    Require(Target);
    auto Decl = exprs::Expr::make_Nick(ctx, parent, {Target->str()}, *Target);

    auto self = decls::Decl::make_Alias(ctx, parent, Name->str(), Decl, *Name);
    self->attrs() = std::exchange(m_current_attrs, {});

    expected(Lex(), ";");

    #ifdef _QW_use_verbose_for_frontend
      std::cerr << color::YELLOW << __func__ << " " << self << color::RESET << std::endl;
    #endif

    return {};
  }

  fun frontend::read_VarDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<word> Names;
    re:
    auto Name = Lex();
    Require(Name);
    Names.push_back(*Name);

    auto Colon = Lex();
    Require(Colon);

    if (Colon->view() == ":");
    ef (Colon->view() == ",") goto re;
    else
      return errors::ExpectedIdentifierBut2(*Colon, Colon->str(), ":", ",");

    auto Type = read_Type(parent, true);
    val_error(Type);

    auto Assi = Lex();
    Require(Assi);

    if (Assi->view() == "=") {
      if (Names.size() != 1)
        return std::unexpected(errors::OnlyOneVariableCanBeInitialized(*Assi, Assi->str()));

      auto Expr = read_Expr(parent, Precedence::Lowest);
      val_error(Expr);

      auto self = decls::Decl::make_Var(ctx, parent, Names[0].str(), Names[0], *Type, Visibility::Private, *Expr);
      self->attrs() = std::exchange(m_current_attrs, {});
    }
    else {
      LexStore(Assi);

      for (size_t i = 0; i < Names.size(); i++) {
        auto self = decls::Decl::make_Var(ctx, parent, Names[i].str(), Names[i], *Type);
        self->attrs() = m_current_attrs;
      }
      m_current_attrs.clear();
    }

    expected(Lex(), ";");
    return {};
  }

  fun frontend::read_IFaceDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = Lex();
    Require(Name);
    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);
    self->attrs() = std::exchange(m_current_attrs, {});

    auto generic_ctx = read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    std::vector<types::Type*> baseTypes;
    std::vector<word> baseTypePos;
    auto ColonOpt = Lex();

    if (ColonOpt && ColonOpt->view() == ":") while (true) {
      auto Pk = Lex();
      word pos = Pk ? *Pk : *Name;
      if (Pk) LexStore(Pk);

      auto parsedType = read_Type(parent, true);
      val_error(parsedType);
      baseTypes.push_back(*parsedType);
      baseTypePos.push_back(pos);

      auto CommaOpt = Lex();

      if (CommaOpt && CommaOpt->view() == ",") continue;
      ef (CommaOpt) {
        LexStore(CommaOpt);
        break;
      }
      else break;
    }
    ef (ColonOpt) LexStore(ColonOpt);

    auto Type = read_IFaceType(self, false, baseTypes, baseTypePos);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;
    (*Type)->owner_ident = self;

    return {};
  }

  fun frontend::read_StructDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = Lex();
    Require(Name);

    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);
    self->attrs() = std::exchange(m_current_attrs, {});

    auto generic_ctx = read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    std::vector<types::Type*> baseTypes;
    std::vector<word> baseTypePos;
    auto ColonOpt = Lex();

    if (ColonOpt && ColonOpt->view() == ":") while (true) {
      auto Pk = Lex();
      word pos = Pk ? *Pk : *Name;
      if (Pk) LexStore(Pk);

      auto parsedType = read_Type(parent, true);
      val_error(parsedType);
      baseTypes.push_back(*parsedType);
      baseTypePos.push_back(pos);

      auto CommaOpt = Lex();
      if (CommaOpt && CommaOpt->view() == ",") continue;
      ef (CommaOpt) {
        LexStore(CommaOpt);
        break;
      }
      else break;
    }
    ef (ColonOpt)
      LexStore(ColonOpt);

    auto Type = read_StructType(self, false, baseTypes, baseTypePos);
    val_error(Type);

    self->as<decls::TypeDecl>()->type = *Type;
    (*Type)->owner_ident = self;

    return {};
  }

  fun frontend::read_EnumDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = Lex();
    Require(Name);
    
    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);
    self->attrs() = std::exchange(m_current_attrs, {});

    auto generic_ctx = read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    auto ColonOpt = Lex();
    types::Type *baseType = nullptr;
    word baseTypePos{};

    if (ColonOpt && ColonOpt->view() == ":") {
      auto Pk = Lex();
      baseTypePos = Pk ? *Pk : *Name;
      if (Pk) LexStore(Pk);

      auto parsedType = read_Type(parent, true);
      val_error(parsedType);
      baseType = *parsedType;
      auto Bracket = Lex();
      expected(Bracket, "{");
    }
    ef (ColonOpt) {
      if (ColonOpt->view() != "{")
        return errors::ExpectedIdentifierBut(*ColonOpt, ColonOpt->str(), "{");
    }
    else
      return errors::ExpectedIdentifierBut2(*Name, Name->str(), "{", ":");

    auto enumDecl = decls::Decl::make_Enum(ctx, self, "", *Name, Visibility::Public);
    enumDecl->attrs() = std::exchange(m_current_attrs, {});
    
    std::vector<types::FieldCons> vals;
    std::vector<types::FieldType> typs;
    
    i64 next_val = 1;
    
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else LexStore(ID);

      // Name
      auto EnumConst = Lex();
      Require_Word(EnumConst, continue);
      
      i64 current_val = next_val;

      auto AssignOpt = Lex();
      if (AssignOpt && AssignOpt->view() == "=") {
          auto val_expr = read_Expr(parent, Precedence::Lowest);
          val_error(val_expr);
          if (!(*val_expr)->is<exprs::IntegerLiteral>())
            return errors::InvalidConstantValue(*AssignOpt, Name->str(), "enum assigned value must be an integer literal");

          auto lit = (*val_expr)->as<exprs::IntegerLiteral>();
          if (std::holds_alternative<u128>(lit->val))
            current_val = (i64)std::get<u128>(lit->val);
          else
            current_val = (i64)std::get<i128>(lit->val);
      }
      ef (AssignOpt) LexStore(AssignOpt);

      vals.push_back({EnumConst->str(), current_val});
      next_val = current_val + 1;

      auto Comma = Lex();
      if (Comma && Comma->view() == ",") continue;
      ef (Comma && Comma->view() == "}") LexStore(Comma);
      ef (Comma)
        return errors::UnexpectedIdentifier(*Comma, Comma->str());
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

  fun frontend::read_SetDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Name = Lex();
    Require(Name);
    auto self = decls::Decl::make_Type(ctx, parent, Name->str(), *Name);
    self->attrs() = std::exchange(m_current_attrs, {});

    auto generic_ctx = read_GenericParams(self);
    if_except_ref(generic_ctx);
    self->set_generic(*generic_ctx);

    auto ColonOpt = Lex();
    types::Type *baseType = nullptr;
    word baseTypePos{};

    if (ColonOpt && ColonOpt->view() == ":") {
      auto Pk = Lex();
      baseTypePos = Pk ? *Pk : *Name;
      if (Pk) LexStore(Pk);

      auto parsedType = read_Type(parent, true);
      val_error(parsedType);
      baseType = *parsedType;
      auto Bracket = Lex();
      expected(Bracket, "{");
    }
    ef (ColonOpt) {
      if (ColonOpt->view() != "{")
        return errors::ExpectedIdentifierBut(*ColonOpt, ColonOpt->str(), "{");
    }
    else
      return errors::ExpectedIdentifierBut2(*Name, Name->str(), "{", ":");

    
    auto setDecl = decls::Decl::make_Set(ctx, self, "", *Name, Visibility::Public);
    setDecl->attrs() = std::exchange(m_current_attrs, {});
    
    std::vector<types::FieldCons> vals;
    std::vector<types::FieldType> typs;
    
    i64 next_val = 1;
    
    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else LexStore(ID);

      // Name
      auto SetConst = Lex();
      Require_Word(SetConst, continue);
      
      i64 current_val = next_val;

      auto AssignOpt = Lex();
      if (AssignOpt && AssignOpt->view() == "=") {
          auto val_expr = read_Expr(parent, Precedence::Lowest);
          val_error(val_expr);
          if (!(*val_expr)->is<exprs::IntegerLiteral>())
            return errors::InvalidConstantValue(*AssignOpt, Name->str(), "set assigned value must be an integer literal");
        
          auto lit = (*val_expr)->as<exprs::IntegerLiteral>();
        if (std::holds_alternative<u128>(lit->val))
          current_val = (i64)std::get<u128>(lit->val);
        else
          current_val = (i64)std::get<i128>(lit->val);
      }
      ef (AssignOpt) LexStore(AssignOpt);

      vals.push_back({SetConst->str(), current_val});
      next_val = (i64)next_pow2((u64)current_val);

      auto Comma = Lex();
      if (Comma && Comma->view() == ",") continue;
      ef (Comma && Comma->view() == "}") LexStore(Comma);
      ef (Comma)
        return errors::UnexpectedIdentifier(*Comma, Comma->str());
    }

    std::string tname = self ? std::string(self->name()) : "set";
    auto setType = types::Type::make_Set(ctx, vals, typs, setDecl, baseType, baseTypePos, tname);
    self->as<decls::TypeDecl>()->type = setType;
    setType->owner_ident = self;

    return {};
  }

  fun frontend::read_ModDecl(decls::Decl *parent) -> std::expected<void, uptr<diagnostic::message>> {
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

    if (ctx->is_parsed(target_file)) {
      return errors::CyclicImport(*Name, mod_name);
    }
    ctx->mark_parsed(target_file);

    auto sub_mod = ctx->make_module(mod_name, target_file);
    auto sub_ns = decls::Decl::make_NameSpace(ctx, parent, mod_name, *Name);
    sub_ns->attrs() = std::exchange(m_current_attrs, {});

    frontend sub_front(sub_mod);
    auto err = sub_front.read_File(sub_ns);
    if (!err) return err;

    return {};
  }

  fun frontend::read_StructFuncDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>> {
    auto fndecl = read_FuncDecl(parent);
    if_except_ref(fndecl);

    auto fdecl     = (*fndecl)->as<decls::FuncDecl>();
    auto old_ftype = fdecl->funcType->as<types::FuncType>();

    std::vector<types::FieldType> new_pars = old_ftype->pars;

    new_pars.insert(new_pars.begin(), {"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    fdecl->funcType = types::Type::make_Func(ctx, new_pars, old_ftype->ret);

    return {};
  }

  fun frontend::read_StructConstructorDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>> {
    auto self = decls::Decl::make_Constructor(ctx, parent, *LexLast(), nil, vis);

    expected(Lex(), "(");

    std::vector<types::FieldType> pars;
    pars.push_back({"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    // Params
    auto parsed_pars = read_FuncParams(self);
    val_error(parsed_pars);
    pars.insert(pars.end(), parsed_pars->begin(), parsed_pars->end());

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
        if (Comma->view() == ",") {
          continue;
        }
        else {
          LexStore(Comma);
          break;
        }
      }

      Ret = Lex();
      Require(Ret);
    }

    // Code & Decl
    if (Ret->view() == ";") return {};
    ef (Ret->view() == "{") {
      auto block_ret = read_CodeBlock(self);
      if (!block_ret)
        return std::unexpected(std::move(block_ret.error()));
      else
        return {};
    }
    else
      return errors::ExpectedIdentifierBut2(*Ret, Ret->str(), ";", "{");
  }

  fun frontend::read_StructDestructorDecl(decls::Decl *parent, types::Type *recType, Visibility vis) -> std::expected<void, uptr<diagnostic::message>> {
    auto self = decls::Decl::make_Destructor(ctx, parent, *LexLast(), nil, vis);

    expected(Lex(), "(");

    std::vector<types::FieldType> pars;
    pars.push_back({"self", types::Type::make_Reference(ctx, recType), Visibility::Public});

    // Params
    auto parsed_pars = read_FuncParams(self);
    val_error(parsed_pars);
    pars.insert(pars.end(), parsed_pars->begin(), parsed_pars->end());

    auto FType = types::Type::make_Func(ctx, pars, ctx->void_t());
    self->as<decls::DestructorDecl>()->funcType = FType;

    auto Ret = Lex();
    Require(Ret);

    // Code & Decl
    if (Ret->view() == ";") return {};
    ef (Ret->view() == "{") {
      auto block_ret = read_CodeBlock(self);
      if (!block_ret)
        return std::unexpected(std::move(block_ret.error()));
      else
        return {};
    }
    else
      return errors::ExpectedIdentifierBut2(*Ret, Ret->str(), ";", "{");
  }



  fun frontend::read_CodeBlock(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
    auto self = stmts::Stmt::make_CodeBlock(ctx, parent, *LexLast());

    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      ef (ID->view() == "ret")   if_error(read_ReturnStmt(self))
      ef (ID->view() == "var")   if_error(read_VarStmt(self))
      ef (ID->view() == "let")   if_error(read_LetStmt(self))
      ef (ID->view() == "if")    if_error(read_IfStmt(self))
      ef (ID->view() == "while") if_error(read_WhileStmt(self))
      ef (ID->view() == "break") {
        stmts::Stmt::make_Break(ctx, self, *ID);
        expected(Lex(), ";");
      }
      ef (ID->view() == "continue") {
        stmts::Stmt::make_Continue(ctx, self, *ID);
        expected(Lex(), ";");
      }
      ef (ID->view() == "unsafe") {
        if_error(read_UnsafeStmt(self, *ID));
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

  fun frontend::read_IfStmt(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
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
      else
        return errors::ExpectedIdentifierBut2(*nxt2, nxt2->str(), "if", "{");
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

  fun frontend::read_WhileStmt(identy *parent) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
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

  fun frontend::read_VarStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<word> Vars;

    // Name
    re:
    auto Name = Lex();
    Require(Name);
    Vars.push_back(*Name);

    auto Colon = Lex();
    Require(Colon);

    if (Colon->view() == ",") goto re;
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

    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_LetStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<word> Vars;

    // Name
    re:
    auto Name = Lex();
    Require(Name);
    Vars.push_back(*Name);

    auto Colon = Lex();
    Require(Colon);

    if (Colon->view() == ",") goto re;
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

    expected(Lex(), ";");

    return {};
  }

  fun frontend::read_ReturnStmt(identy *parent) -> std::expected<void, uptr<diagnostic::message>> {
    auto Pos = LexLast();

    auto Expr = read_Expr(parent, Precedence::Lowest);
    val_error(Expr);
    expected(Lex(), ";");

    auto self = stmts::Stmt::make_Return(ctx, parent, *Pos, *Expr);

    return {};
  }

  fun frontend::read_UnsafeStmt(identy *parent, word pos) -> std::expected<stmts::Stmt*, uptr<diagnostic::message>> {
    auto next = Lex();
    Require(next);

    auto obj = stmts::Stmt::make_Unsafe(ctx, parent, pos, nullptr);

    if (next->view() == "{") {
      auto blk = read_CodeBlock(obj);
      if (!blk) return std::unexpected(std::move(blk.error()));
      obj->as<stmts::UnsafeStmt>()->stmt = *blk;
    }
    else {
      LexStore(next);
      auto expr = read_Expr(obj, Precedence::Lowest);
      if (!expr) return std::unexpected(std::move(expr.error()));
      obj->as<stmts::UnsafeStmt>()->stmt = stmts::Stmt::make_ExprStmt(ctx, obj, *expr, pos);
      expected(Lex(), ";");
    }

    return obj;
  }



  fun frontend::read_Expr(identy *parent, Precedence min_prec) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
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
      std::string_view view = ID->view();
      int base = 10;
      size_t offset = 0;
      
      if (view.size() > 2 && view[0] == '0') {
        if (view[1] == 'x' || view[1] == 'X') {
          base = 16;
          offset = 2;
        }
        ef (view[1] == 'b' || view[1] == 'B') {
          base = 2;
          offset = 2;
        }
      }
      
      auto eret = std::from_chars(view.begin() + offset, view.end(), val, base);
      if (eret.ec != std::errc() || eret.ptr != view.end()) {
        return errors::CantConvertInteger(*ID, ID->str());
      }
      ret = exprs::Expr::make_IntegerLiteral(ctx, parent, val, *ID);
    }
    ef (ID->view()[0] == '\'') {
      if (ID->view().size() == 2) return errors::EmptyCharacterConstant(*ID);
      std::string unescaped = unescape_string(ID->view().substr(1, ID->view().size() - 2));
      if (unescaped.size() == 0) return errors::EmptyCharacterConstant(*ID);
      if (unescaped.size() > 1)  return errors::CharacterConstantTooLong(*ID, (std::string)ID->view().substr(1, ID->view().size() - 2));

      ret = exprs::Expr::make_CharLiteral(ctx, parent, unescaped[0], *ID);
    }
    ef (ID->view()[0] == '\"') {
      ret = exprs::Expr::make_StringLiteral(
        ctx, parent, 
        unescape_string(ID->view().substr(1, ID->view().size() - 2)),
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

    
    auto p_ret = read_Expr_Postfix(parent, ret);
    val_error(p_ret);
    ret = *p_ret;

    auto i_ret = read_Expr_Infix(parent, min_prec, ret);
    val_error(i_ret);
    return *i_ret;
  }

  fun frontend::read_Expr_Postfix(identy *parent, exprs::Expr *ret) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
    
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
        u0 old_off = Off;
        
        std::vector<types::Type *> generic_args;
        bool success = true;
        
        while (true) {
          auto type = read_Type(parent, true);
          if (!type) {
            success = false;
            break;
          }
          generic_args.push_back(*type);
          
          auto Next = Lex();
          if (!Next) {
            success = false;
            break;
          }
          
          if (Next->view() == ">") break;
          ef (Next->view() == ",") continue;
          else {
            success = false;
            break;
          }
        }
        
        if (success)
          ret = exprs::Expr::make_GenericOp(ctx, parent, ret, std::move(generic_args), *Op);
        else {
          Off = old_off;
          m_lexStore = std::nullopt;
          LexStore(Op);
          break;
        }
      }
      else {
        LexStore(Op);
        break;
      }
    }
    return ret;
  }

  fun frontend::read_Expr_Infix(identy *parent, Precedence min_prec, exprs::Expr *ret) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
    
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
        if (Op->view() == "+=")  kind = exprs::BinaryOpEnum::AddAssign;
        ef (Op->view() == "-=")  kind = exprs::BinaryOpEnum::SubAssign;
        ef (Op->view() == "*=")  kind = exprs::BinaryOpEnum::MulAssign;
        ef (Op->view() == "/=")  kind = exprs::BinaryOpEnum::DivAssign;
        ef (Op->view() == "%=")  kind = exprs::BinaryOpEnum::RemAssign;
        ef (Op->view() == "&=")  kind = exprs::BinaryOpEnum::BitAndAssign;
        ef (Op->view() == "|=")  kind = exprs::BinaryOpEnum::BitOrAssign;
        ef (Op->view() == "^=")  kind = exprs::BinaryOpEnum::BitXorAssign;
        ef (Op->view() == "<<=") kind = exprs::BinaryOpEnum::ShlAssign;
        ef (Op->view() == ">>=") kind = exprs::BinaryOpEnum::ShrAssign;
      }
      ef (Op->view() == "==" || Op->view() == "!=") {
        op_prec = Precedence::Eq;
        kind    = Op->view() == "==" ? exprs::BinaryOpEnum::Eq : exprs::BinaryOpEnum::NEq;
      }
      ef (Op->view() == "<" || Op->view() == ">" || Op->view() == "<=" || Op->view() == ">=") {
        op_prec = Precedence::Rel;
        if (Op->view() == "<") kind = exprs::BinaryOpEnum::Lt;
        ef (Op->view() == ">") kind = exprs::BinaryOpEnum::Gt;
        ef (Op->view() == "<=") kind = exprs::BinaryOpEnum::LEq;
        ef (Op->view() == ">=") kind = exprs::BinaryOpEnum::GEq;
      }
      ef (Op->view() == "+" || Op->view() == "-") {
        op_prec = Precedence::Add;
        kind    = Op->view() == "+" ? exprs::BinaryOpEnum::Add : exprs::BinaryOpEnum::Sub;
      }
      ef (Op->view() == "*" || Op->view() == "/" || Op->view() == "%") {
        op_prec = Precedence::Mul;
        kind    = Op->view() == "*" ? exprs::BinaryOpEnum::Mul : Op->view() == "/" ? exprs::BinaryOpEnum::Div : exprs::BinaryOpEnum::Rem;
      }
      ef (Op->view() == "&") {
        op_prec = Precedence::BitAnd;
        kind    = exprs::BinaryOpEnum::BitAnd;
      }
      ef (Op->view() == "|") {
        op_prec = Precedence::BitOr;
        kind    = exprs::BinaryOpEnum::BitOr;
      }
      ef (Op->view() == "^") {
        op_prec = Precedence::BitXor;
        kind    = exprs::BinaryOpEnum::BitXor;
      }
      ef (Op->view() == "<<" || Op->view() == ">>") {
        op_prec = Precedence::Shift;
        kind    = Op->view() == "<<" ? exprs::BinaryOpEnum::Shl : exprs::BinaryOpEnum::LShr;
      }
      ef (Op->view() == "&&") {
        op_prec = Precedence::LogAnd;
        kind    = exprs::BinaryOpEnum::LogAnd;
      }
      ef (Op->view() == "||") {
        op_prec = Precedence::LogOr;
        kind    = exprs::BinaryOpEnum::LogOr;
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

  
  
  fun frontend::read_Type(identy *parent, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto ID = Lex();
    Require(ID);

    std::expected<types::Type*, uptr<diagnostic::message>> ret;

    if (ID->view() == "struct") {
      ret = read_StructType(parent, indecl);
      goto fin;
    }
    ef (ID->view() == "iface") {
      ret = read_IFaceType(parent, indecl);
      goto fin;
    }
    ef (ID->view() == "fun") {
      ret = read_FuncType(parent, indecl);
      goto fin;
    }
    ef (indecl) {
      auto Nick = types::Type::make_Nick(ctx, {ID->str()});
      Nick->owner_ident = parent;

      re:
      auto S = Lex();
      Require(S);

      if (S->view() == "::") {
        auto N = Lex();
        Require_Word(N, {ret = Nick; goto fin;});

        Nick->as<types::NickType>()->unresolved.push_back(N->str());
        goto re;
      }
      else
        LexStore(S);

      ret = Nick;
      goto fin;
    }
    else
      return errors::UnknownKeyword(*ID, ID->str());

    fin:
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
        ef (Op->view() == "<") {
          u0 old_off = Off;
          
          std::vector<types::Type *> generic_args;
          bool success = true;
          
          while (true) {
            auto type = read_Type(parent, true);
            if (!type) {
              success = false;
              break;
            }
            generic_args.push_back(*type);
            
            auto Next = Lex();
            if (!Next) {
              success = false;
              break;
            }
            
            if (Next->view() == ">") break;
            ef (Next->view() == ",") continue;
            else {
              success = false;
              break;
            }
          }
          
          if (success) {
            baseType = types::Type::make_Generic(ctx, baseType, std::move(generic_args));
            baseType->owner_ident = parent;
          }
          else {
            Off = old_off;
            m_lexStore = std::nullopt;
            LexStore(Op);
            break;
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

  fun frontend::read_StructType(identy *parent, bool indecl, std::vector<types::Type*> baseTypes, std::vector<word> baseTypePos) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto Bracket = Lex();
    expected(Bracket, "{");

    Visibility visdef = Visibility::Public;
    std::vector<types::FieldType> vars;

    decls::Decl *recDecl = nil;

    if (parent && parent->type() == IdentyEnum::Decl) {
      auto pDecl = static_cast<decls::Decl *>(parent);
      recDecl    = decls::Decl::make_Struct(ctx, pDecl, "", pDecl->pos(), Visibility::Public);
    }

    std::string tname = (parent && parent->type() == IdentyEnum::Decl) ? std::string(static_cast<decls::Decl *>(parent)->name()) : "struct";
    auto selfType = types::Type::make_Struct(ctx, {}, {}, recDecl ? recDecl : nullptr, baseTypes, baseTypePos, tname);

    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else LexStore(ID);

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
      else LexStore(Vis);

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
      ef (Kwd->view() == "fini") {
        if_error(read_StructDestructorDecl(recDecl, selfType, visone));
        continue;
      }
      else {
        LexStore(Kwd);
      }

      // Name
      std::vector<std::string> Names;
      re:
      auto Name = Lex();
      Require_Word(Name, continue);
      Names.push_back(Name->str());

      // , :
      auto Colon = Lex();
      Require(Colon);
      if (Colon->view() == ",") goto re;
      ef (Colon->view() == ":");
      else
        return errors::ExpectedIdentifierBut2(*Colon, Colon->str(), ",", ":");

      // Type
      auto Type = read_Type(parent, true);
      val_error(Type);

      // End
      expected(Lex(), ";");

      // Add
      for (auto &X: Names) vars.push_back({X, *Type, visone});
    }

    selfType->as<types::StructType>()->vars = vars;

    return selfType;
  }

  fun frontend::read_IFaceType(identy *parent, bool indecl, std::vector<types::Type*> baseTypes, std::vector<word> baseTypePos) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto Bracket = Lex();
    expected(Bracket, "{");

    Visibility visdef = Visibility::Public;
    std::vector<types::FieldType> typs;

    decls::Decl *recDecl = nil;
    if (parent && parent->type() == IdentyEnum::Decl) {
      auto pDecl = static_cast<decls::Decl *>(parent);
      recDecl    = decls::Decl::make_IFace(ctx, pDecl, "", pDecl->pos(), Visibility::Public);
    }

    std::string tname = (parent && parent->type() == IdentyEnum::Decl) ? std::string(static_cast<decls::Decl*>(parent)->name()) : "iface";
    auto selfType = types::Type::make_IFace(ctx, {}, recDecl ? recDecl : nullptr, baseTypes, baseTypePos, tname);

    while (true) {
      auto ID = Lex();
      Require(ID);

      if (ID->view() == "}") break;
      else LexStore(ID);

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
      else LexStore(Vis);

      // Must be a Function in an interface
      auto Kwd = Lex();
      Require(Kwd);
      if (Kwd->view() == "fun") {
        if_error(read_StructFuncDecl(recDecl, selfType, visone));
        continue;
      }
      else
        return errors::OnlyFunAllowed(*Kwd);
    }

    return selfType;
  }

  fun frontend::read_FuncType(identy *parent, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto Bracket = Lex();
    expected(Bracket, "(");

    // Params
    auto parsed_pars = read_FuncParams(parent);
    val_error(parsed_pars);
    std::vector<types::FieldType> pars = *parsed_pars;

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

    
    auto self = types::Type::make_Func(ctx, pars, retType);

    return self;
  }

  fun frontend::read_GenericParams(decls::Decl *parent) -> std::expected<decls::GenericContext*, uptr<diagnostic::message>> {
    auto OptAngle = Lex();
    if (!OptAngle || OptAngle->view() != "<") {
      if (OptAngle) LexStore(OptAngle);
      return nullptr;
    }
    
    auto ctx_obj = new decls::GenericContext{};
    
    while (true) {
      auto PName = Lex();
      Require(PName);
      
      auto param_decl = decls::Decl::make_TypeParam(ctx, parent, PName->str(), *PName, Visibility::Public);
      
      ctx_obj->params.push_back(param_decl);
      
      auto CommaOrEnd = Lex();
      Require(CommaOrEnd);
      if (CommaOrEnd->view() == ">") break;
      ef (CommaOrEnd->view() == ",") continue;
      else
        return errors::ExpectedIdentifierBut(*CommaOrEnd, CommaOrEnd->str(), "> veya ,");
    }
    
    return ctx_obj;
  }

}
