/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/sema/sema.hh"
#include "qw/tree/clone.hh"
#include "qw/basis.hh"
#include "qw/control/scopemng.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/types.hh"
#include <cassert>
#include <expected>
#include <memory>
#include <string>
#include <unordered_map>
#include <vector>

#define if_error(X)                                                                                                                                  \
  {                                                                                                                                                  \
    auto E = X;                                                                                                                                      \
    if (!E.has_value()) {                                                                                                                            \
      if (E.error()->type() == qw::diagnostic::MsgType::Fatal)                                                                                       \
        return std::unexpected(std::move(E.error()));                                                                                                \
      else {                                                                                                                                         \
        sum.add(E.error().get());                                                                                                                    \
        std::cerr << E.error();                                                                                                                      \
      }                                                                                                                                              \
    }                                                                                                                                                \
  }

#define if_except(X)                                                                                                                                 \
  {                                                                                                                                                  \
    auto E = X;                                                                                                                                      \
    if (!E.has_value())                                                                                                                              \
      return std::unexpected(std::move(E.error()));                                                                                                  \
  }

#define val_error(X)                                                                                                                                 \
  {                                                                                                                                                  \
    if (!X.has_value())                                                                                                                              \
      return std::unexpected(std::move(X.error()));                                                                                                  \
  }



namespace qw
{

  fun TypeSema::sema_Type(types::Type *&now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    assert(now && "null parameter.");

    l_begin:
    if (now->is<types::PrimitiveType>())
      return {};
    ef (now->is<types::TypeParamType>())
      return {};

    ef (now->is<types::StructType>()) return sctx.type->sema_StructType(now, errpos);
    ef (now->is<types::IFaceType>())  return sctx.type->sema_IFaceType(now, errpos);
    ef (now->is<types::EnumType>())   return sctx.type->sema_EnumType(now, errpos);
    ef (now->is<types::SetType>())    return sctx.type->sema_SetType(now, errpos);

    ef (now->is<types::FuncType>()) {
      auto ftype = now->as<types::FuncType>();
      std::vector<types::FieldType> resolved_pars = ftype->pars;
      
      for (auto &X : resolved_pars)
        if_except(sctx.type->sema_Type(X.type, errpos));

      auto resolved_ret = ftype->ret;
      if_except(sctx.type->sema_Type(resolved_ret, errpos));

      now = types::Type::make_Func(ctx, resolved_pars, resolved_ret);
    }
    ef(now->is<types::PointerType>()) {
      auto sub = now->as<types::PointerType>()->sub;
      if_except(sctx.type->sema_Type(sub, errpos));
      now = types::Type::make_Pointer(ctx, sub);
    }
    ef(now->is<types::ReferenceType>()) {
      auto sub = now->as<types::ReferenceType>()->sub;
      if_except(sctx.type->sema_Type(sub, errpos));
      now = types::Type::make_Reference(ctx, sub);
    }
    ef(now->is<types::ZArrayType>()) {
      auto sub = now->as<types::ZArrayType>()->sub;
      if_except(sctx.type->sema_Type(sub, errpos));
      now = types::Type::make_ZArray(ctx, sub);
    }
    ef(now->is<types::PArrayType>()) {
      auto parray = now->as<types::PArrayType>();
      auto sub = parray->sub;
      u64 size = parray->size;
      if_except(sctx.type->sema_Type(sub, errpos));
      now = types::Type::make_PArray(ctx, sub, size);
    }

    ef(now->is<types::GenericType>()) {
      auto gen = now->as<types::GenericType>();
      if_except(sctx.type->sema_Type(gen->sub, errpos));
      for (auto &f : gen->fields) {
        if_except(sctx.type->sema_Type(f, errpos));
      }

      decls::Decl* base_decl = nullptr;
      if (gen->sub->is<types::StructType>()) base_decl = gen->sub->as<types::StructType>()->decl;
      ef (gen->sub->is<types::EnumType>()) base_decl = gen->sub->as<types::EnumType>()->decl;
      ef (gen->sub->is<types::SetType>()) base_decl = gen->sub->as<types::SetType>()->decl;
      ef (gen->sub->is<types::IFaceType>()) base_decl = gen->sub->as<types::IFaceType>()->decl;

      if (!base_decl) {
        return errors::TypeCannotBeGeneric(errpos, std::string(gen->sub->typname()));
      }

      if (!base_decl->is_generic() && base_decl->parent() && base_decl->parent()->type() == IdentyEnum::Decl) {
        auto parent_decl = static_cast<decls::Decl*>(base_decl->parent());
        if (parent_decl->is_generic()) base_decl = parent_decl;
      }

      if (!base_decl->is_generic()) {
        return errors::TypeCannotBeGeneric(errpos, std::string(gen->sub->typname()));
      }

      if (base_decl->generic()->params.size() != gen->fields.size()) {
        return errors::GenericParamsNotEqual(errpos, std::to_string(base_decl->generic()->params.size()), std::to_string(gen->fields.size()));
      }

      auto &instantiations = base_decl->generic()->instantiations;
      if (instantiations.count(gen->fields)) {
        now = instantiations[gen->fields];
      } else {
        tree::Cloner cloner(ctx);
      for (size_t i = 0; i < gen->fields.size(); ++i) {
        cloner.type_map[base_decl->generic()->params[i]] = gen->fields[i];
      }

        auto inst_type = cloner.clone_Type(gen->sub);
        instantiations[gen->fields] = inst_type;
        
        if_except(sctx.type->sema_Type(inst_type, errpos));
        now = inst_type;
      }
    }
    ef(now->is<types::NickType>()) {
      auto ret = sctx.type->sema_NickType(now, errpos);

      if (ret.has_value()) {
        now = *ret;
        goto l_begin;
      }
      else
        return std::unexpected(std::move(ret.error()));
    }
    else
      diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());

    return {};
  }

  fun TypeSema::sema_EnumType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    now->sema() = StageStatus::Checking;
    
    auto enumType = now->as<types::EnumType>();
    
    now->sema() = StageStatus::Checked;
    
    if (enumType->decl) {
      for (auto &F: enumType->decl->as<decls::EnumDecl>()->func) {
        if_except(sctx.decl->sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }

    i64 min_val = 0, max_val = 0;
    if (!enumType->vals.empty()) {
      min_val = std::holds_alternative<u64>(enumType->vals[0].val) ? (i64)std::get<u64>(enumType->vals[0].val) : (i64)std::get<i64>(enumType->vals[0].val);
      max_val = min_val;
    }

    for (auto &v: enumType->vals) {
      i64 cv = std::holds_alternative<u64>(v.val) ? (i64)std::get<u64>(v.val) : (i64)std::get<i64>(v.val);
      if (cv < min_val) min_val = cv;
      if (cv > max_val) max_val = cv;
    }

    if (enumType->baseType) {
      word base_pos = enumType->baseTypePos ? enumType->baseTypePos : errpos;
      if_except(sctx.type->sema_Type(enumType->baseType, base_pos));
      if (!enumType->baseType->isInteger()) {
        return errors::InvalidBaseType(base_pos, std::string(now->typname()), "must be an integer");
      }
      
      bool is_signed = enumType->baseType->isSigned();
      u32 bits = enumType->baseType->intBit(ctx);
      
      i64 b_min = is_signed ? -((i64)1 << (bits - 1)) : 0;
      i64 b_max = is_signed ? ((i64)1 << (bits - 1)) - 1 : (bits == 128 ? (i64)-1 : ((i64)1 << bits) - 1);
      
      if (min_val < b_min || max_val > b_max)
        return errors::InvalidConstantValue(base_pos, std::string(now->typname()), "enum values do not fit in the specified base type");
    }
    else {
      if (min_val >= 0) {
        if (max_val <= 255) enumType->baseType = ctx->intU8_t();
        ef (max_val <= 65535) enumType->baseType = ctx->intU16_t();
        ef (max_val <= 4294967295ULL) enumType->baseType = ctx->intU32_t();
        else enumType->baseType = ctx->intU64_t();
      }
      else {
        if (min_val >= -128 && max_val <= 127) enumType->baseType = ctx->intS8_t();
        ef (min_val >= -32768 && max_val <= 32767) enumType->baseType = ctx->intS16_t();
        ef (min_val >= -2147483648LL && max_val <= 2147483647LL) enumType->baseType = ctx->intS32_t();
        else enumType->baseType = ctx->intS64_t();
      }
    }

    now->llvm() = enumType->baseType->llvm();
    return {};
  }

  fun TypeSema::sema_SetType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    now->sema() = StageStatus::Checking;
    
    auto setType = now->as<types::SetType>();
    
    now->sema() = StageStatus::Checked;
    
    if (setType->decl) {
      for (auto &F: setType->decl->as<decls::SetDecl>()->func) {
        if_except(sctx.decl->sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }

    i64 max_val = 0;
    if (!setType->vals.empty()) {
      max_val = std::holds_alternative<u64>(setType->vals[0].val) ? (i64)std::get<u64>(setType->vals[0].val) : (i64)std::get<i64>(setType->vals[0].val);
    }

    for (auto &v: setType->vals) {
      i64 cv = std::holds_alternative<u64>(v.val) ? (i64)std::get<u64>(v.val) : (i64)std::get<i64>(v.val);
      if (cv > max_val) max_val = cv;
    }

    if (setType->baseType) {
      word base_pos = setType->baseTypePos ? setType->baseTypePos : errpos;
      if_except(sctx.type->sema_Type(setType->baseType, base_pos));
      if (!setType->baseType->isInteger()) {
        return errors::InvalidBaseType(base_pos, std::string(now->typname()), "must be an integer");
      }
      
      bool is_signed = setType->baseType->isSigned();
      u32 bits = setType->baseType->intBit(ctx);
      
      i64 b_max = is_signed ? ((i64)1 << (bits - 1)) - 1 : (bits == 128 ? (i64)-1 : ((i64)1 << bits) - 1);
      
      if (max_val > b_max)
        return errors::InvalidConstantValue(base_pos, std::string(now->typname()), "set values do not fit in the specified base type");
    }
    else {
      if (max_val <= 255) setType->baseType = ctx->intU8_t();
      ef (max_val <= 65535) setType->baseType = ctx->intU16_t();
      ef (max_val <= 4294967295ULL) setType->baseType = ctx->intU32_t();
      else setType->baseType = ctx->intU64_t();
    }

    now->llvm() = setType->baseType->llvm();

    return {};
  }

  fun TypeSema::sema_StructType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    ef (now->sema() == StageStatus::Checking) return {};

    now->sema() = StageStatus::Checking;

    auto strct = now->as<types::StructType>();

    std::vector<types::FieldType> flattened_vars;
    for (size_t i = 0; i < strct->baseTypes.size(); ++i) {
      auto &baseType = strct->baseTypes[i];
      word base_pos = strct->baseTypePos.size() > i ? strct->baseTypePos[i] : errpos;
      if_except(sctx.type->sema_Type(baseType, base_pos));
      auto resolved_base = baseType;

      while (resolved_base->isReference())
        resolved_base = resolved_base->as<types::ReferenceType>()->sub;

      if (resolved_base->is<types::StructType>()) {
        auto bstrct = resolved_base->as<types::StructType>();
        flattened_vars.insert(flattened_vars.end(), bstrct->vars.begin(), bstrct->vars.end());
      }
      ef (resolved_base->is<types::IFaceType>()) {
        auto iface = resolved_base->as<types::IFaceType>();
        if (!iface->decl || !strct->decl) continue;

        for (auto &iface_func : iface->decl->as<decls::IFaceDecl>()->func) {
          auto iface_fdecl = iface_func->as<decls::FuncDecl>();
          auto iface_ftype = iface_fdecl->funcType->as<types::FuncType>();

          bool implemented = false;
          
          std::vector<types::StructType*> search_queue;
          search_queue.push_back(strct);
          
          while (!search_queue.empty()) {
            auto search_rec = search_queue.back();
            search_queue.pop_back();

            for (auto &rec_func : search_rec->decl->as<decls::StructDecl>()->func) {
              auto rec_fdecl = rec_func->as<decls::FuncDecl>();
              if (std::string(rec_func->name()) == std::string(iface_func->name())) {
                auto rec_ftype = rec_fdecl->funcType->as<types::FuncType>();
                
                if (rec_ftype->pars.size() == iface_ftype->pars.size()) {
                  bool match = true;
                  for (size_t i = 1; i < rec_ftype->pars.size(); ++i) {
                    if (rec_ftype->pars[i].type != iface_ftype->pars[i].type) {
                      match = false;
                      break;
                    }
                  }
                  if (match && rec_ftype->ret == iface_ftype->ret) {
                    implemented = true;
                    break;
                  }
                }
              }
            }
            if (implemented) break;

            for (auto bt: search_rec->baseTypes) {
              auto rbt = bt;
              while(rbt->isReference()) rbt = rbt->as<types::ReferenceType>()->sub;
              if (rbt->is<types::StructType>()) {
                search_queue.push_back(rbt->as<types::StructType>());
              }
            }
          }
          
          if (!implemented) {
            return errors::MissingInterfaceMethod(base_pos, std::string(now->typname()), std::string(iface_func->name()), std::string(resolved_base->typname()));
          }
        }
      }
      else
        return errors::InvalidBaseType(base_pos, std::string(now->typname()), "must be a struct or interface");
    }

    for (auto &X: strct->vars) {
      if_except(sctx.type->sema_Type(X.type, errpos));
      flattened_vars.push_back(X);
    }

    strct->vars = flattened_vars;

    now->sema() = StageStatus::Checked;

    if (strct->decl) {
      for (auto &F: strct->decl->as<decls::StructDecl>()->func) {
        if_except(sctx.decl->sema_Attributes(F));
        if_except(sctx.decl->sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
      for (auto &C: strct->decl->as<decls::StructDecl>()->constructors) {
        if_except(sctx.decl->sema_Attributes(C));
        if_except(sctx.decl->sema_ConstructorDecl(C));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(C), C);
      }
      for (auto &D: strct->decl->as<decls::StructDecl>()->destructors) {
        if_except(sctx.decl->sema_Attributes(D));
        if_except(sctx.decl->sema_DestructorDecl(D));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(D), D);
      }
    }

    return {};
  }

  fun TypeSema::sema_IFaceType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    ef (now->sema() == StageStatus::Checking) return {};
    
    now->sema() = StageStatus::Checking;
    auto iface = now->as<types::IFaceType>();

    for (size_t i = 0; i < iface->baseTypes.size(); ++i) {
      auto &baseType = iface->baseTypes[i];
      word base_pos = iface->baseTypePos.size() > i ? iface->baseTypePos[i] : errpos;
      if_except(sctx.type->sema_Type(baseType, base_pos));
      auto resolved_base = baseType;

      while (resolved_base->isReference())
        resolved_base = resolved_base->as<types::ReferenceType>()->sub;

      if (!resolved_base->is<types::IFaceType>())
        return errors::InvalidBaseType(base_pos, std::string(now->typname()), "must be an interface");
    }

    if (iface->decl) {
      for (auto &F: iface->decl->as<decls::IFaceDecl>()->func) {
        if_except(sctx.decl->sema_Attributes(F));
        if_except(sctx.decl->sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }

    now->sema() = StageStatus::Checked;
    return {};
  }

  fun TypeSema::sema_NickType(types::Type *now, word errpos) -> std::expected<types::Type*, uptr<diagnostic::message>> {
    auto nick = now->as<types::NickType>();
    const auto __join_human = [](std::vector<std::string> &ps) -> std::string {
      std::string ret = ps[0];
      for (int i = 1; i < ps.size(); i++)
        ret += "::" + ps[i];
      return ret;
    };

    auto ret = SMng.lookup(now->owner_ident, nick->unresolved);

    if (!ret)
      return errors::IdentifierNotFound(errpos, __join_human(nick->unresolved));
    ef (auto typ = SMng.fetch_type(ret))
      return typ;
    else
      return errors::IdentifierNType(errpos, __join_human(nick->unresolved));
  }

  fun TypeSema::sema_FuncType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    auto ftype = now->as<types::FuncType>();

    for (auto &X: ftype->pars)
      if_except(sctx.type->sema_Type(X.type, errpos));

    if_except(sctx.type->sema_Type(ftype->ret, errpos));
    return {};
  }

}
