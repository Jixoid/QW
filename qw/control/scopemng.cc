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

  static std::string build_type_path(decls::Decl* d)
  {
    if (!d) return "";
    std::string source_name = std::to_string(d->name().size()) + std::string(d->name());
    
    char kind = 'S';
    if (d->is<decls::StructDecl>()) kind = 'S';
    ef (d->is<decls::IFaceDecl>()) kind = 'I';
    ef (d->is<decls::EnumDecl>()) kind = 'E';
    ef (d->is<decls::SetDecl>()) kind = 'B';
    ef (d->is<decls::TypeDecl>()) {
      auto tdecl = d->as<decls::TypeDecl>();
      if (tdecl->type->is<types::StructType>()) kind = 'S';
      ef (tdecl->type->is<types::IFaceType>()) kind = 'I';
      ef (tdecl->type->is<types::EnumType>()) kind = 'E';
      ef (tdecl->type->is<types::SetType>()) kind = 'B';
    }
    
    std::string path = "";
    identy *curr = d->parent();
    while (curr && curr->type() == IdentyEnum::Decl) {
      auto p = (decls::Decl*)curr;
      if (!p->name().empty()) {
        path = std::to_string(p->name().size()) + std::string(p->name()) + path;
      }
      curr = curr->parent();
    }
    
    return path + kind + source_name;
  }

  static std::string get_generic_suffix(decls::Decl* type_decl, types::Type* inst_type, decls::Decl* inst_decl) {
    if (!type_decl->is_generic()) return "";
    for (auto& [args, t] : type_decl->generic()->instantiations) {
      if (inst_type && t == inst_type) {
        std::string res = "G";
        for (auto a : args) res += scopemng::mangle_type(a, "");
        return res;
      }
      if (inst_decl) {
        decls::Decl* d = nullptr;
        if (t->is<types::StructType>()) d = t->as<types::StructType>()->decl;
        else if (t->is<types::EnumType>()) d = t->as<types::EnumType>()->decl;
        else if (t->is<types::IFaceType>()) d = t->as<types::IFaceType>()->decl;
        else if (t->is<types::SetType>()) d = t->as<types::SetType>()->decl;
        if (d == inst_decl) {
          std::string res = "G";
          for (auto a : args) res += scopemng::mangle_type(a, "");
          return res;
        }
      }
    }
    return "";
  }

  fun scopemng::mangle_type(types::Type *t, const std::string& ctx_type_path) -> std::string
  {
    if (!t) return "v";

    if (t->is<types::PrimitiveType>()) {
      auto kind = t->as<types::PrimitiveType>()->kind;
      switch (kind) {
        case types::PrimitiveEnum::I8:    return "St";
        case types::PrimitiveEnum::I16:   return "Ss";
        case types::PrimitiveEnum::I32:   return "Si";
        case types::PrimitiveEnum::I64:   return "Sl";
        case types::PrimitiveEnum::I128:  return "Sy";
        case types::PrimitiveEnum::U8:    return "Ut";
        case types::PrimitiveEnum::U16:   return "Us";
        case types::PrimitiveEnum::U32:   return "Ui";
        case types::PrimitiveEnum::U64:   return "Ul";
        case types::PrimitiveEnum::U128:  return "Uy";
        case types::PrimitiveEnum::USize: return "Un";
        case types::PrimitiveEnum::ISize: return "Sn";
        case types::PrimitiveEnum::F16:   return "h";
        case types::PrimitiveEnum::F32:   return "f";
        case types::PrimitiveEnum::F64:   return "d";
        case types::PrimitiveEnum::F128:  return "g";
        case types::PrimitiveEnum::Bool:  return "b";
        case types::PrimitiveEnum::Char:  return "c";
        case types::PrimitiveEnum::Void:  return "v";
        case types::PrimitiveEnum::Ptr:   return "p";
        case types::PrimitiveEnum::Null:  return "l";
      }
    }
    if (t->is<types::PointerType>())   return "P" + mangle_type(t->as<types::PointerType>()->sub, ctx_type_path);
    if (t->is<types::ReferenceType>()) return "R" + mangle_type(t->as<types::ReferenceType>()->sub, ctx_type_path);
    
    if (t->is<types::ZArrayType>()) return "Z" + mangle_type(t->as<types::ZArrayType>()->sub, ctx_type_path);
    if (t->is<types::PArrayType>()) {
      auto pa = t->as<types::PArrayType>();
      return "A" + std::to_string(pa->size) + "_" + mangle_type(pa->sub, ctx_type_path);
    }

    char kind = 'S';
    bool is_complex = false;
    
    if (t->is<types::StructType>()) { kind = 'S'; is_complex = true; }
    ef (t->is<types::IFaceType>()) { kind = 'I'; is_complex = true; }
    ef (t->is<types::EnumType>()) { kind = 'E'; is_complex = true; }
    ef (t->is<types::SetType>()) { kind = 'B'; is_complex = true; }

    if (is_complex) {
      std::string tp;
      if (t->owner_ident && t->owner_ident->type() == IdentyEnum::Decl) {
          tp = build_type_path((decls::Decl*)t->owner_ident);
          tp += get_generic_suffix((decls::Decl*)t->owner_ident, t, nullptr);
      } else {
          std::string name = std::string(t->typname());
          tp = kind + std::to_string(name.size()) + name;
      }
      
      if (!ctx_type_path.empty() && tp == ctx_type_path) {
          return "x";
      }
      return "N" + tp + "Z";
    }
    
    return "X";
  }

  fun scopemng::mangling_abi_qw(identy *now) -> std::string
  {
    if (now && now->type() == IdentyEnum::Decl) {
      auto d = (decls::Decl *)now;
      
      // Free functions checks (for RTL binding bypass)
      identy *curr = d;
      std::vector<std::string> path;
      while (curr) {
        if (curr->type() == IdentyEnum::Decl) {
          auto dp = (decls::Decl *)curr;
          if (!dp->name().empty()) {
            path.push_back(std::string(dp->name()));
          }
        }
        curr = curr->parent();
      }
      
      if (path.size() >= 2 && path.back() == "sys" && d->is<decls::FuncDecl>()) {
        std::string rtl_name = "qwrtl";
        for (int i = path.size() - 2; i >= 0; i--)
          rtl_name += "_" + path[i];
        
        return rtl_name;
      }

      if (d->is<decls::FuncDecl>() || d->is<decls::ConstructorDecl>() || d->is<decls::DestructorDecl>()) {
        types::FuncType *ftype = nullptr;
        if (d->is<decls::FuncDecl>()) {
          auto fdecl = d->as<decls::FuncDecl>();
          if (fdecl->funcType && fdecl->funcType->is<types::FuncType>())
            ftype = fdecl->funcType->as<types::FuncType>();
        }
        else if (d->is<decls::ConstructorDecl>()) {
          auto cdecl = d->as<decls::ConstructorDecl>();
          if (cdecl->funcType && cdecl->funcType->is<types::FuncType>())
            ftype = cdecl->funcType->as<types::FuncType>();
        }
        else if (d->is<decls::DestructorDecl>()) {
          auto ddecl = d->as<decls::DestructorDecl>();
          if (ddecl->funcType && ddecl->funcType->is<types::FuncType>())
            ftype = ddecl->funcType->as<types::FuncType>();
        }

        std::string mangled = "_qw_";
        
        decls::Decl* parent_type_decl = nullptr;
        if (d->parent() && d->parent()->type() == IdentyEnum::Decl) {
          auto p = (decls::Decl*)d->parent();
          if (p->is<decls::StructDecl>() || p->is<decls::IFaceDecl>() || p->is<decls::EnumDecl>() || p->is<decls::SetDecl>()) {
            if (p->parent() && p->parent()->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(p->parent())->is<decls::TypeDecl>()) {
                parent_type_decl = static_cast<decls::Decl*>(p->parent());
            }
          }
        }
        
        std::string ctx_type_path = "";
        
        if (parent_type_decl) {
          ctx_type_path = build_type_path(parent_type_decl);
          ctx_type_path += get_generic_suffix(parent_type_decl, nullptr, (decls::Decl*)d->parent());
          mangled += ctx_type_path;
        }
        else {
          std::string scopes = "";
          identy *sc = d->parent();
          while (sc && sc->type() == IdentyEnum::Decl) {
            auto p = (decls::Decl*)sc;
            if (!p->name().empty())
              scopes = std::to_string(p->name().size()) + std::string(p->name()) + scopes;
            
            sc = sc->parent();
          }
          mangled += scopes;
        }
        
        if (d->is<decls::ConstructorDecl>()) {
          mangled += "C";
          if (ftype) {
            size_t start_arg = 0;
            if (!ftype->pars.empty() && ftype->pars[0].name == "self") start_arg = 1;
            for (size_t i = start_arg; i < ftype->pars.size(); i++)
              mangled += mangle_type(ftype->pars[i].type, ctx_type_path);
          }
        }
        ef (d->is<decls::DestructorDecl>()) {
          mangled += "D";
        }
        else {
          mangled += "F" + std::to_string(d->name().size()) + std::string(d->name());
          
          if (ftype) {
            std::string self_type_str = "v"; 
            size_t start_arg = 0;
            
            if (!ftype->pars.empty() && ftype->pars[0].name == "self") {
              self_type_str = mangle_type(ftype->pars[0].type, ctx_type_path);
              start_arg = 1;
            }
            
            mangled += self_type_str;
            mangled += mangle_type(ftype->ret, ctx_type_path);
            
            for (size_t i = start_arg; i < ftype->pars.size(); i++)
              mangled += mangle_type(ftype->pars[i].type, ctx_type_path);
            
          }
          else {
            mangled += "vv"; // fallback (self: void, ret: void)
          }
        }
        
        if (d->name() == "main" && !parent_type_decl) return "qw_entry";
        return mangled;
      }
      
      if (d->is<decls::TypeDecl>())
        return "_qw_" + build_type_path(d);
      
      
      // Fallback for VarDecl, AliasDecl, etc.
      std::string ret = "";
      identy *curr_fb = d;
      while (curr_fb) {
        if (curr_fb->type() == IdentyEnum::Decl) {
          auto dp = (decls::Decl *)curr_fb;
          if (!dp->name().empty()) {
            ret = std::to_string(dp->name().size()) + std::string(dp->name()) + ret;
          }
        }
        curr_fb = curr_fb->parent();
      }
      if (ret.empty()) return "_qw_";
      return "_qw_" + ret;
    }
    
    return "_qw_UNKNOWN";
  }

  fun scopemng::fetch_type(identy *ident) -> types::Type*
  {
    if (!ident)
      return nil;

    if (ident->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(ident)->is<decls::TypeDecl>())
      return static_cast<decls::Decl*>(ident)->as<decls::TypeDecl>()->type;
    ef (ident->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(ident)->is<decls::TypeParamDecl>())
      return types::Type::make_TypeParam(ctx, static_cast<decls::Decl*>(ident), std::string(static_cast<decls::Decl*>(ident)->name()));

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
          if (C->is_generic()) {
            for (auto &gp : C->generic()->params) {
              if (gp->name() == names[0]) {
                return gp;
              }
            }
          }

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

          ef (C->is<decls::StructDecl>()) {
            for (auto &X : C->as<decls::StructDecl>()->func)
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
    
    // RTL Bindings intercept
    if (names.size() >= 2 && names[0] == "sys") {
      std::string rtl_name = "qwrtl";
      for (int i = 1; i < names.size(); i++) {
        rtl_name += "_" + names[i];
      }
      for (auto &GST: m_gst) {
        if (auto it = GST->find(rtl_name))
          return *it;
      }
    }

    for (auto &ANS: m_ans) {
      for (auto &GST: m_gst) {
        if (arg_types) {
          std::string prefix = ANS;
          for (size_t i = 0; i < names.size() - 1; i++) {
              prefix += std::to_string(names[i].size()) + names[i];
          }
          std::string func_prefix = prefix + "F" + std::to_string(names.back().size()) + names.back();
          
          for (const auto &pair : GST->idents()) {
              if (pair.first.find(func_prefix) == 0) {
                  auto fdecl = (decls::Decl*)pair.second;
                  if (fdecl->is<decls::FuncDecl>()) {
                    auto ftype = fdecl->as<decls::FuncDecl>()->funcType->as<types::FuncType>();
                    // Skip self parameter for free functions? We don't have enough context, but we check arg_types size
                    size_t ast_args = ftype->pars.size();
                    size_t req_args = arg_types->size();
                    size_t start_idx = (ast_args > 0 && ftype->pars[0].name == "self") ? 1 : 0;
                    if (ast_args - start_idx == req_args) {
                      bool match = true;
                      for (size_t i = 0; i < req_args; i++) {
                        auto t1 = ftype->pars[start_idx + i].type;
                        auto t2 = (*arg_types)[i];
                        if (t1->isReference()) t1 = t1->as<types::ReferenceType>()->sub;
                        if (t2->isReference()) t2 = t2->as<types::ReferenceType>()->sub;
                        if (t1->typname() != t2->typname()) { match = false; break; }
                      }
                      if (match) { ret = pair.second; goto l_end; }
                    }
                  }
              }
          }
        }
        else {
          std::string prefix = ANS;
          for (size_t i = 0; i < names.size() - 1; i++) {
            prefix += std::to_string(names[i].size()) + names[i];
          }
          std::string last_part = std::to_string(names.back().size()) + names.back();
          
          std::string bT = prefix + last_part;
          std::string bS = prefix + "S" + last_part;
          std::string bI = prefix + "I" + last_part;
          std::string bE = prefix + "E" + last_part;
          std::string bB = prefix + "B" + last_part;
          
          if (auto it = GST->find(bT)) { ret = *it; goto l_end; }
          if (auto it = GST->find(bS)) { ret = *it; goto l_end; }
          if (auto it = GST->find(bI)) { ret = *it; goto l_end; }
          if (auto it = GST->find(bE)) { ret = *it; goto l_end; }
          if (auto it = GST->find(bB)) { ret = *it; goto l_end; }
          
          for (const auto &pair : GST->idents()) {
            if (pair.first.find(bT) == 0 || pair.first == bT) {
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
