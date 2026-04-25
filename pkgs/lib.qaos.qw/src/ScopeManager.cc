/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "ScopeManager.hh"
#include "Basis.hh"
#include "Decls.hh"
#include "Msgs.hh"
#include "Stmts.hh"
#include "I18N.hh"
#include "Types.hh"
#include "Typs.hh"
#include "Diagnostic.hh"
#include <optional>
#include <string>
#include <variant>

using namespace std;



namespace qw
{

  fun scopeManager::mangling_abi_qw(qIdentifier *now) -> string
  {
    string ret;
    u32 anon{};

    auto nnow = now;
    while (now) {
      if (now->type() == qIdentifier::qie_Type) {
        if (now->parent() && now->parent()->type() == qIdentifier::qie_Decl && (
          static_cast<decls::qDecl*>(now->parent())->subType() != decls::qDecl::qde_TypeDecl &&
          static_cast<decls::qDecl*>(now->parent())->subType() != decls::qDecl::qde_FuncDecl
        ))
          anon++;
      }

      now = now->parent();
    }



    // Create Mangling
    auto IDecl = [&ret, &anon](decls::qDecl *now)
    {
      if (!now->name().empty())
        ret = to_string(now->name().size()) + now->name() +ret;
    };

    auto IType = [&ret, &anon](types::qType *now)
    {
      if (now->parent() && now->parent()->type() == qIdentifier::qie_Decl && (
        static_cast<decls::qDecl*>(now->parent())->subType() != decls::qDecl::qde_TypeDecl &&
        static_cast<decls::qDecl*>(now->parent())->subType() != decls::qDecl::qde_FuncDecl
      ))
        ret = "$_"+to_string(anon--)+"$" +ret;


      if (auto X = static_cast<types::qMemberType*>(now); now->subType() == types::qType::qte_MemberField)
        ret = to_string(X->name().size()) + X->name() +ret;
    };


    now = nnow;
    while (now) {
      if (now->type() == qIdentifier::qie_Decl) IDecl((decls::qDecl*)now);
      ef (now->type() == qIdentifier::qie_Type) IType((types::qType*)now);

      now = now->parent();
    }


    return (ret == "4main") ? "main":"_Q"+ret;
  }

  fun scopeManager::humanreadableTypes(types::qType *__now) -> string
  {
    qIdentifier *now{__now};

    string ret;


    // Create Mangling
    auto IDecl = [&ret](decls::qDecl *now)
    {
      if (!now->name().empty())
        ret = "::"+now->name() +ret;
    };

    auto IType = [&ret](types::qType *now) -> optional<string>
    {
      switch (now->subType())
      {
        case types::qType::qte_IntU8:   return "u8";
        case types::qType::qte_IntU16:  return "u16";
        case types::qType::qte_IntU32:  return "u32";
        case types::qType::qte_IntU64:  return "u64";
        case types::qType::qte_IntU128: return "u128";

        case types::qType::qte_IntS8:   return "i8";
        case types::qType::qte_IntS16:  return "i16";
        case types::qType::qte_IntS32:  return "i32";
        case types::qType::qte_IntS64:  return "i64";
        case types::qType::qte_IntS128: return "i128";

        case types::qType::qte_Char: return "char";
        case types::qType::qte_Bool: return "bool";
        case types::qType::qte_Void: return "void";
        case types::qType::qte_Ptr:  return "ptr";

        case types::qType::qte_Reference: {
          return scopeManager::humanreadableTypes(static_cast<types::qSubType*>(now)->sub())+"&";
        };
      
        default: break;
      }


      if (now->parent() && now->parent()->type() == qIdentifier::qie_Decl && (
        static_cast<decls::qDecl*>(now->parent())->subType() != decls::qDecl::qde_TypeDecl &&
        static_cast<decls::qDecl*>(now->parent())->subType() != decls::qDecl::qde_FuncDecl
      ))
        ret = "::anonymous" +ret;


      if (auto X = static_cast<types::qMemberType*>(now); now->subType() == types::qType::qte_MemberField)
        ret = "::"+X->name() +ret;

      return {};
    };


    while (now) {
      if (now->type() == qIdentifier::qie_Decl) IDecl((decls::qDecl*)now);
      ef (now->type() == qIdentifier::qie_Type) {
        auto ret = IType((types::qType*)now);

        if (ret.has_value()) return *ret;
      }

      now = now->parent();
    }


    return ret.substr(2);
  }


  

  fun scopeManager::fetch_type(qIdentifier *ident) -> types::qType*
  {
    if (!ident) return Nil;

    ef (ident->type() == qIdentifier::qie_Type)
      return (types::qType*)ident;

    ef (ident->type() == qIdentifier::qie_Decl && static_cast<decls::qDecl*>(ident)->subType() == decls::qDecl::qde_TypeDecl)
      return static_cast<decls::qTypeDecl*>(ident)->targetType();

    else return Nil;
  }


  fun scopeManager::fetch_expr(qIdentifier *ident) -> exprs::qExpr*
  {
    if (!ident) return Nil;

    ef (ident->type() == qIdentifier::qie_Expr)
      return (exprs::qExpr*)ident;

    ef (ident->type() == qIdentifier::qie_Stmt && static_cast<stmts::qStmt*>(ident)->subType() == stmts::qStmt::qse_CodeVar)
    {
      auto NType = types::qType::make_Reference(ctx, ident, static_cast<stmts::qCodeVar*>(ident)->pos(), static_cast<stmts::qCodeVar*>(ident)->targetType());

      return exprs::qExpr::make_ValExpr(ctx, ident, NType, static_cast<stmts::qCodeVar*>(ident)->llvm(), static_cast<stmts::qCodeVar*>(ident)->pos());
    }


    else return Nil;
  }



  inline fun __join_symbol(vector<string> &ps) -> string
  {
    string ret;

    for (auto x: ps)
      ret += to_string(x.size()) +x;

    return ret;
  }


  fun scopeManager::lookup(qIdentifier *ident, vector<string> names) -> qIdentifier*
  {
    assert(names.size() != 0 && "size should not be 0");

    l_unwind:
    if (names.size() == 1) while (ident)
    {
      if (auto C = (decls::qDecl*)ident; ident->type() == qIdentifier::qie_Decl) switch (C->subType())
      {
        case decls::qDecl::qde_NameSpaceDecl: {
          for (auto &X: static_cast<decls::qNameSpaceDecl*>(ident)->decls())
            if (X->name() == names[0])
              return X;

          ident = ident->parent();
          break;
        }
        
        case decls::qDecl::qde_VarDecl:  ident = ident->parent(); break;
        case decls::qDecl::qde_FuncDecl: ident = ident->parent(); break;
        case decls::qDecl::qde_TypeDecl: ident = ident->parent(); break;

        default:
          diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());
      }
      ef (auto C = (types::qType*)ident; ident->type() == qIdentifier::qie_Type) switch (C->subType())
      {
        case types::qType::qte_Record: {
          for (auto &X: static_cast<types::qRecordType*>(ident)->vars())
            if (X->name() == names[0])
              return X;

          ident = ident->parent();
          break;
        }

        case types::qType::qte_Func: {
          for (auto &X: static_cast<types::qFuncType*>(ident)->pars())
            if (X->name() == names[0])
              return X;

          ident = ident->parent();
          break;
        }


        default:
          diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
      }
      ef (auto C = (stmts::qStmt*)ident; ident->type() == qIdentifier::qie_Stmt) switch (C->subType())
      {
        case stmts::qStmt::qse_CodeBlock: {
          for (auto &X: static_cast<stmts::qCodeBlock*>(ident)->vars())
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
    qIdentifier *ret{};

    for (auto &ANS: m_ans)
      for (auto &GST: m_gst)
        if (auto it = GST->find(ANS + __join_symbol(names))) {
          ret = *it;
          goto l_end;
        }
    
    
    l_end:
    if (!ret) return Nil;
    
    ef (ret->type() == qIdentifier::qie_Type && ((types::qType*)ret)->subType() == types::qType::qte_Nick) { // fake tailcall
      names = ((types::qNickType*)ret)->unResolvedName();
      ret = Nil;
      goto l_gst;
    }

    else
      return ret;
  }

}
