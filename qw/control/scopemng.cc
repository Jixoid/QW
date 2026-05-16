/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qw/basis.hh"
#include "qw/control/scopemng.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/diagnostic/i18n.hh"
#include "qw/pretype.hh"
#include <optional>
#include <string>
#include <variant>



namespace qw
{

  fun scopemng::mangling_abi_qw(identy *now) -> std::string
  {
    std::string ret;
    u32 anon{};

    auto nnow = now;
    while (now) {
      if (now->type() == IdentyEnum::Type) {
        if (now->parent() && now->parent()->type() == IdentyEnum::Decl && (
          static_cast<decls::Decl*>(now->parent())->subType() != decls::DeclEnum::Type &&
          static_cast<decls::Decl*>(now->parent())->subType() != decls::DeclEnum::Func
        ))
          anon++;
      }

      now = now->parent();
    }



    // Create Mangling
    auto IDecl = [&ret, &anon](decls::Decl *now)
    {
      if (!now->name().empty())
        ret = std::to_string(now->name().size()) + (std::string)now->name() +ret;
    };

    auto IType = [&ret, &anon](types::Type *now)
    {
      if (now->parent() && now->parent()->type() == IdentyEnum::Decl && (
        static_cast<decls::Decl*>(now->parent())->subType() != decls::DeclEnum::Type &&
        static_cast<decls::Decl*>(now->parent())->subType() != decls::DeclEnum::Func
      ))
        ret = "$_"+std::to_string(anon--)+"$" +ret;


      if (auto X = static_cast<types::Member*>(now); now->subType() == types::TypeEnum::MemberField)
        ret = std::to_string(X->name().size()) + X->name() +ret;
    };


    now = nnow;
    while (now) {
      if (now->type() == IdentyEnum::Decl) IDecl((decls::Decl*)now);
      ef (now->type() == IdentyEnum::Type) IType((types::Type*)now);

      now = now->parent();
    }


    return (ret == "4main") ? "main":"_Q"+ret;
  }

  fun scopemng::humanreadableTypes(types::Type *__now) -> std::string
  {
    identy *now{__now};

    std::string ret;


    // Create Mangling
    auto IDecl = [&ret](decls::Decl *now)
    {
      if (!now->name().empty())
        ret = "::" + (std::string)now->name() +ret;
    };

    auto IType = [&ret](types::Type *now) -> std::optional<std::string>
    {
      switch (now->subType())
      {
        case types::TypeEnum::IntU8:   return "u8";
        case types::TypeEnum::IntU16:  return "u16";
        case types::TypeEnum::IntU32:  return "u32";
        case types::TypeEnum::IntU64:  return "u64";
        case types::TypeEnum::IntU128: return "u128";

        case types::TypeEnum::IntS8:   return "i8";
        case types::TypeEnum::IntS16:  return "i16";
        case types::TypeEnum::IntS32:  return "i32";
        case types::TypeEnum::IntS64:  return "i64";
        case types::TypeEnum::IntS128: return "i128";

        case types::TypeEnum::Char: return "char";
        case types::TypeEnum::Bool: return "bool";
        case types::TypeEnum::Void: return "void";
        case types::TypeEnum::Ptr:  return "ptr";

        case types::TypeEnum::Reference: {
          return scopemng::humanreadableTypes(static_cast<types::Sub*>(now)->sub())+"&";
        };
      
        default: break;
      }


      if (now->parent() && now->parent()->type() == IdentyEnum::Decl && (
        static_cast<decls::Decl*>(now->parent())->subType() != decls::DeclEnum::Type &&
        static_cast<decls::Decl*>(now->parent())->subType() != decls::DeclEnum::Func
      ))
        ret = "::anonymous" +ret;


      if (auto X = static_cast<types::Member*>(now); now->subType() == types::TypeEnum::MemberField)
        ret = "::"+X->name() +ret;

      return {};
    };


    while (now) {
      if (now->type() == IdentyEnum::Decl) IDecl((decls::Decl*)now);
      ef (now->type() == IdentyEnum::Type) {
        auto ret = IType((types::Type*)now);

        if (ret.has_value()) return *ret;
      }

      now = now->parent();
    }


    return ret.substr(2);
  }


  

  fun scopemng::fetch_type(identy *ident) -> types::Type*
  {
    if (!ident) return nil;

    ef (ident->type() == IdentyEnum::Type)
      return (types::Type*)ident;

    ef (ident->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(ident)->subType() == decls::DeclEnum::Type)
      return static_cast<decls::Type*>(ident)->targetType();

    else return nil;
  }


  fun scopemng::fetch_expr(identy *ident) -> exprs::Expr*
  {
    if (!ident) return nil;

    ef (ident->type() == IdentyEnum::Expr)
      return (exprs::Expr*)ident;

    ef (ident->type() == IdentyEnum::Stmt && static_cast<stmts::Stmt*>(ident)->subType() == stmts::StmtEnum::CodeVar)
    {
      auto NType = types::Type::make_Reference(ctx, ident, static_cast<stmts::CodeVar*>(ident)->pos(), static_cast<stmts::CodeVar*>(ident)->targetType());

      return exprs::Expr::make_ValExpr(ctx, ident, NType, static_cast<stmts::CodeVar*>(ident)->llvm(), static_cast<stmts::CodeVar*>(ident)->pos());
    }


    else return nil;
  }



  inline fun __join_symbol(std::vector<std::string> &ps) -> std::string
  {
    std::string ret;

    for (auto x: ps)
      ret += std::to_string(x.size()) +x;

    return ret;
  }


  fun scopemng::lookup(identy *ident, std::vector<std::string> names) -> identy*
  {
    assert(names.size() != 0 && "size should not be 0");

    l_unwind:
    if (names.size() == 1) while (ident)
    {
      if (auto C = (decls::Decl*)ident; ident->type() == IdentyEnum::Decl) switch (C->subType())
      {
        case decls::DeclEnum::NameSpace: {
          for (auto &X: static_cast<decls::NameSpace*>(ident)->decls())
            if (X->name() == names[0])
              return X;

          ident = ident->parent();
          break;
        }
        
        case decls::DeclEnum::Var:  ident = ident->parent(); break;
        case decls::DeclEnum::Func: ident = ident->parent(); break;
        case decls::DeclEnum::Type: ident = ident->parent(); break;

        default:
          diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());
      }
      ef (auto C = (types::Type*)ident; ident->type() == IdentyEnum::Type) switch (C->subType())
      {
        case types::TypeEnum::Record: {
          for (auto &X: static_cast<types::Record*>(ident)->vars())
            if (X->name() == names[0])
              return X;

          ident = ident->parent();
          break;
        }

        case types::TypeEnum::Func: {
          for (auto &X: static_cast<types::Func*>(ident)->pars())
            if (X->name() == names[0])
              return X;

          ident = ident->parent();
          break;
        }


        default:
          diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
      }
      ef (auto C = (stmts::Stmt*)ident; ident->type() == IdentyEnum::Stmt) switch (C->subType())
      {
        case stmts::StmtEnum::CodeBlock: {
          for (auto &X: static_cast<stmts::CodeBlock*>(ident)->vars())
            if (X->name() == names[0])
              return X;

          ident = ident->parent();
          break;
        }


        default:
          diagnostic::fatal(fatals::Internal_UnknownStmt().error()->msg());
      }

      else
        diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
    }


    l_gst:
    identy *ret{};

    for (auto &ANS: m_ans)
      for (auto &GST: m_gst)
        if (auto it = GST->find(ANS + __join_symbol(names))) {
          ret = *it;
          goto l_end;
        }
    
    
    l_end:
    if (!ret) return nil;
    
    ef (ret->type() == IdentyEnum::Type && ((types::Type*)ret)->subType() == types::TypeEnum::Nick) { // fake tailcall
      names = ((types::Nick*)ret)->unResolvedName();
      ret = nil;
      goto l_gst;
    }

    else
      return ret;
  }

}
