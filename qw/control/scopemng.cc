/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/control/scopemng.hh"
#include "qw/basis.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include <optional>
#include <string>



namespace qw
{

  static std::string mangle_type(types::Type *t)
  {
    if (!t) return "v";

    if (t->is<types::PrimitiveType>()) {
      auto kind = t->as<types::PrimitiveType>()->kind;
      switch (kind) {
        case types::PrimitiveEnum::I8:   return "i8";
        case types::PrimitiveEnum::I16:  return "i16";
        case types::PrimitiveEnum::I32:  return "i32";
        case types::PrimitiveEnum::I64:  return "i64";
        case types::PrimitiveEnum::I128: return "i128";
        case types::PrimitiveEnum::U8:   return "u8";
        case types::PrimitiveEnum::U16:  return "u16";
        case types::PrimitiveEnum::U32:  return "u32";
        case types::PrimitiveEnum::U64:  return "u64";
        case types::PrimitiveEnum::U128: return "u128";
        case types::PrimitiveEnum::F16:  return "f16";
        case types::PrimitiveEnum::F32:  return "f32";
        case types::PrimitiveEnum::F64:  return "f64";
        case types::PrimitiveEnum::F128: return "f128";
        case types::PrimitiveEnum::Bool: return "b";
        case types::PrimitiveEnum::Char: return "c";
        case types::PrimitiveEnum::Void: return "v";
        case types::PrimitiveEnum::Ptr:  return "p";
      }
    }
    if (t->is<types::PointerType>())   return "P" + mangle_type(t->as<types::PointerType>()->sub);
    if (t->is<types::ReferenceType>()) return "R" + mangle_type(t->as<types::ReferenceType>()->sub);
    
    return "X";
  }

  fun scopemng::mangling_abi_qw(identy *now) -> std::string
  {
    std::string ret;
    std::string params_mangled = "";

    if (now && now->type() == IdentyEnum::Decl) {
      auto d = (decls::Decl *)now;
      if (d->is<decls::FuncDecl>()) {
        auto fdecl = d->as<decls::FuncDecl>();
        if (fdecl->funcType && fdecl->funcType->is<types::FuncType>()) {
          auto ftype     = fdecl->funcType->as<types::FuncType>();
          params_mangled = "E"; // Parametre başlangıç belirteci
          for (const auto &p : ftype->pars) {
            params_mangled += "_" + mangle_type(p.type);
          }
        }
      }
    }

    identy *curr = now;
    while (curr) {
      if (curr->type() == IdentyEnum::Decl) {
        auto d = (decls::Decl *)curr;
        if (!d->name().empty())
          ret = std::to_string(d->name().size()) + (std::string)d->name() + ret;
      }
      curr = curr->parent();
    }
    return (ret == "4main") ? "main" : "_Q" + ret + params_mangled;
  }

  fun scopemng::fetch_type(identy *ident) -> types::Type*
  {
    if (!ident)
      return nil;

    if (ident->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(ident)->is<decls::TypeDecl>())
      return static_cast<decls::Decl*>(ident)->as<decls::TypeDecl>()->type;

    else
      return nil;
  }

  fun scopemng::fetch_expr(identy *ident) -> exprs::Expr*
  {
    if (!ident)
      return nil;

    if (ident->type() == IdentyEnum::Expr)
      return (exprs::Expr *)ident;

    ef(ident->type() == IdentyEnum::Stmt && static_cast<stmts::Stmt*>(ident)->is<stmts::CodeVar>())
    {
      auto cvar  = static_cast<stmts::Stmt*>(ident)->as<stmts::CodeVar>();
      auto NType = types::Type::make_Reference(ctx, cvar->targetType);
      return exprs::Expr::make_ValExpr(ctx, ident, NType, cvar->llvm, ident->pos());
    }

    else return nil;
  }

  inline fun __join_symbol(std::vector<std::string> &ps) -> std::string
  {
    std::string ret;

    for (auto x : ps)
      ret += std::to_string(x.size()) + x;

    return ret;
  }

  fun scopemng::lookup(identy *ident, std::vector<std::string> names, std::vector<types::Type*> *arg_types) -> identy*
  {
    assert(names.size() != 0 && "size should not be 0");

    l_unwind:
    if (names.size() == 1)
      while (ident) {
        if (ident->type() == IdentyEnum::Decl) {
          auto C = (decls::Decl*)ident;
          if (C->is<decls::NameSpaceDecl>()) {
            for (auto &X: C->as<decls::NameSpaceDecl>()->decls) {
              if (X->name() == names[0]) {
                if (arg_types && X->is<decls::FuncDecl>()) {
                  auto ftype = X->as<decls::FuncDecl>()->funcType->as<types::FuncType>();
                  if (ftype->pars.size() == arg_types->size()) {
                    bool match = true;
                    for (size_t i = 0; i < arg_types->size(); i++) {
                      auto t1 = ftype->pars[i].type;
                      auto t2 = (*arg_types)[i];
                      if (t1->isReference()) t1 = t1->as<types::ReferenceType>()->sub;
                      if (t2->isReference()) t2 = t2->as<types::ReferenceType>()->sub;
                      if (t1->typname() != t2->typname()) {
                        match = false;
                        break;
                      }
                    }
                    if (match)
                      return X;
                  }
                } else
                  return X;
              }
            }
            ident = ident->parent();
          }

          ef (C->is<decls::RecordDecl>()) {
            for (auto &X : C->as<decls::RecordDecl>()->func)
              if (X->name() == names[0]) {
                if (arg_types && X->is<decls::FuncDecl>()) {
                  auto ftype = X->as<decls::FuncDecl>()->funcType->as<types::FuncType>();
                  if (ftype->pars.size() == arg_types->size()) {
                    bool match = true;
                    for (size_t i = 0; i < arg_types->size(); ++i) {
                      auto t1 = ftype->pars[i].type;
                      auto t2 = (*arg_types)[i];
                      if (t1->isReference()) t1 = t1->as<types::ReferenceType>()->sub;
                      if (t2->isReference()) t2 = t2->as<types::ReferenceType>()->sub;
                      if (t1->typname() != t2->typname()) {
                        match = false;
                        break;
                      }
                    }
                    if (match)
                      return X;
                  }
                } else
                  return X;
              }
            
            ident = ident->parent();
          }
          else {
            ident = ident->parent();
          }
        }
        ef (ident->type() == IdentyEnum::Stmt) {
          auto C = (stmts::Stmt*)ident;
          if (C->is<stmts::CodeBlock>()) {
            for (auto &X: C->as<stmts::CodeBlock>()->vars)
              if (static_cast<stmts::Stmt*>(X)->as<stmts::CodeVar>()->name == names[0])
                return X;
            ident = ident->parent();
          }
          else {
            ident = ident->parent();
          }
        }
        else {
          diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());
        }
      }

    l_gst:
    identy *ret{};
    std::string base_target = __join_symbol(names);
    for (auto &ANS: m_ans) {
      for (auto &GST: m_gst) {
        if (arg_types) {
          std::string full_key = ANS + base_target + "E";
          for (auto t : *arg_types)
            full_key += "_" + mangle_type(t);
          if (auto it = GST->find(full_key)) {
            ret = *it;
            goto l_end;
          }
        }
        else {
          if (auto it = GST->find(ANS + base_target)) {
            ret = *it;
            goto l_end;
          }
          for (const auto &pair : GST->idents()) {
            if (pair.first.rfind(ANS + base_target + "E_", 0) == 0 || pair.first == ANS + base_target) {
              ret = pair.second;
              goto l_end;
            }
          }
        }
      }
    }

    l_end:
    if (!ret)
      return nil;
    return ret;
  }

}
