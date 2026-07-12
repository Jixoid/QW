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
#include "qw/tree/types.hh"
#include <expected>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

#define ef else if

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

  fun TypeParser::read_Type(identy *parent, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto ID = lex();
    Require(ID);

    std::expected<types::Type*, uptr<diagnostic::message>> ret;

    if (ID.view() == "struct") {
      ret = read_StructType(parent, indecl);
      goto fin;
    }
    ef (ID.view() == "iface") {
      ret = read_IFaceType(parent, indecl);
      goto fin;
    }
    ef (ID.view() == "fun") {
      ret = read_FuncType(parent, indecl);
      goto fin;
    }
    ef (indecl) {
      auto Nick = types::Type::make_Nick(ctx, {ID.str()});
      Nick->owner_ident = parent;

      re:
      auto S = lex();
      Require(S);

      if (S.is(WordKind::Scope)) {
        auto N = lex();
        Require_Word(N, {ret = Nick; goto fin;});

        Nick->as<types::NickType>()->unresolved.push_back(N.str());
        goto re;
      }
      else
        lex.store(S);

      ret = Nick;
      goto fin;
    }
    else
      return errors::UnknownKeyword(ID, ID.str());

    
    fin:
    if (ret.has_value()) {
      types::Type *baseType = *ret;

      while (true) {
        auto Op = lex();
        if (!Op)
          break;

        if (Op.is(WordKind::BitwiseXor)) baseType = types::Type::make_Pointer(ctx, baseType);
        ef (Op.is(WordKind::BitwiseAnd)) baseType = types::Type::make_Reference(ctx, baseType);
        ef (Op.is(WordKind::SquareBracketBeg)) {
          auto SizeOrClose = lex();
          Require(SizeOrClose);

          if (SizeOrClose.is(WordKind::SquareBracketEnd)) {
            baseType = types::Type::make_ZArray(ctx, baseType);
          }
          else {
            u32 size = std::stoul(SizeOrClose.str());

            auto Close = lex();
            Require(Close);

            baseType = types::Type::make_PArray(ctx, baseType, size);
          }
        }
        ef (Op.is(WordKind::AngleBeg)) {
          u0 old_off = lex.offset();
          
          std::vector<types::Type *> generic_args;
          bool success = true;
          
          while (true) {
            auto type = read_Type(parent, true);
            if (!type) {
              success = false;
              break;
            }
            generic_args.push_back(*type);
            
            auto Next = lex();
            if (!Next) {
              success = false;
              break;
            }
            
            if (Next.is(WordKind::AngleEnd)) break;
            ef (Next.is(WordKind::Comma)) continue;
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
            lex.offset() = old_off;
            lex.store(Op);
            break;
          }
        }
        else {
          lex.store(Op);
          break;
        }
      }

      ret = baseType;
    }

    return ret;
  }
  

  fun TypeParser::read_StructType(identy *parent, bool indecl, std::vector<types::Type*> baseTypes, std::vector<word> baseTypePos) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto Bracket = lex();
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
      auto ID = lex();
      Require(ID);

      if (ID.is(WordKind::CurlyBracketEnd)) break;
      else lex.store(ID);

      auto visone = visdef;

      // Visibility
      auto Vis = lex();
      if (auto id = Vis.view(); id == "pub" || id == "priv" || id == "prot" || id == "crate" || id == "group") {
        auto vis = visone;

        if (id == "pub")   vis = Visibility::Public;
        ef (id == "priv")  vis = Visibility::Private;
        ef (id == "prot")  vis = Visibility::Protected;
        ef (id == "crate") vis = Visibility::Crate;
        ef (id == "group") vis = Visibility::Group;

        auto Colon = lex();
        Require(Colon);

        if (Colon.is(WordKind::Colon)) {
          visdef = vis;
          continue;
        }
        else {
          visone = vis;
          lex.store(Colon);
        }
      }
      else lex.store(Vis);

      // Is Function
      auto Kwd = lex();
      Require(Kwd);
      if (Kwd.view() == "fun") {
        if_error(pctx.decl->read_StructFuncDecl(recDecl, selfType, visone));
        continue;
      }
      ef (Kwd.view() == "init") {
        if_error(pctx.decl->read_StructConstructorDecl(recDecl, selfType, visone));
        continue;
      }
      ef (Kwd.view() == "fini") {
        if_error(pctx.decl->read_StructDestructorDecl(recDecl, selfType, visone));
        continue;
      }
      else {
        lex.store(Kwd);
      }

      // Name
      std::vector<std::string> Names;
      re:
      auto Name = lex();
      Require_Word(Name, continue);
      Names.push_back(Name.str());

      // , :
      auto Colon = lex();
      Require(Colon);
      if (Colon.is(WordKind::Comma)) goto re;
      ef (Colon.is(WordKind::Colon));
      else
        return errors::ExpectedIdentifierBut2(Colon, Colon.str(), ",", ":");

      // Type
      auto Type = read_Type(parent, true);
      val_error(Type);

      // End
      expected(lex(), ";");

      // Add
      for (auto &X: Names) vars.push_back({X, *Type, visone});
    }

    selfType->as<types::StructType>()->vars = vars;

    return selfType;
  }

  fun TypeParser::read_IFaceType(identy *parent, bool indecl, std::vector<types::Type*> baseTypes, std::vector<word> baseTypePos) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto Bracket = lex();
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
      auto ID = lex();
      Require(ID);

      if (ID.is(WordKind::CurlyBracketEnd)) break;
      else lex.store(ID);

      auto visone = visdef;

      // Visibility
      auto Vis = lex();
      if (auto id = Vis.view(); id == "pub" || id == "priv" || id == "prot" || id == "crate" || id == "group") {
        auto vis = visone;

        if (id == "pub")   vis = Visibility::Public;
        ef (id == "priv")  vis = Visibility::Private;
        ef (id == "prot")  vis = Visibility::Protected;
        ef (id == "crate") vis = Visibility::Crate;
        ef (id == "group") vis = Visibility::Group;

        auto Colon = lex();
        Require(Colon);

        if (Colon.is(WordKind::Colon)) {
          visdef = vis;
          continue;
        }
        else {
          visone = vis;
          lex.store(Colon);
        }
      }
      else lex.store(Vis);

      // Must be a Function in an interface
      auto Kwd = lex();
      Require(Kwd);
      if (Kwd.view() == "fun") {
        if_error(pctx.decl->read_StructFuncDecl(recDecl, selfType, visone));
        continue;
      }
      else
        return errors::OnlyFunAllowed(Kwd);
    }

    return selfType;
  }

  fun TypeParser::read_FuncType(identy *parent, bool indecl) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto Bracket = lex();
    expected(Bracket, "(");

    // Params
    auto parsed_pars = pctx.meta->read_FuncParams(parent);
    val_error(parsed_pars);
    std::vector<types::FieldType> pars = *parsed_pars;

    // Return
    auto Ret = lex();
    Require(Ret)

    types::Type *retType = ctx->void_t();
    
    if (Ret.is(WordKind::ArrowRigh)) {
      auto Type = read_Type(parent, true);
      val_error(Type);
      retType = *Type;

      Ret = lex();
      Require(Ret);
    }
    else
      lex.store(Ret);

    
    auto self = types::Type::make_Func(ctx, pars, retType);

    return self;
  }

}
