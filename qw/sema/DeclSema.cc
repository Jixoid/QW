/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/sema/sema.hh"
#include "qw/basis.hh"
#include "qw/control/scopemng.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include <cassert>
#include <expected>
#include <string>
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

  fun DeclSema::sema_Attributes(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    for (auto &attr: now->const_attrs()) {
      auto make_err = [&](auto err) {
        u32 start_off = attr.pos.off();
        u32 end_off = now->pos().off() + now->pos().size();
        word ctx_pos(now->pos().mod(), start_off, end_off - start_off);
        err.error()->ctx() = ctx_pos;
        return err;
      };

      if (attr.name == "symbol") {
        if (attr.value != "bare" && attr.value != "qw")
          return make_err(errors::InvalidAttributeValue(attr.pos, attr.name, attr.value));
      }
      ef (attr.name == "thread_local") {
        if (!now->is<decls::VarDecl>())
          return make_err(errors::AttributeNotSupported(attr.pos, attr.name));
        if (!attr.value.empty())
          return make_err(errors::InvalidAttributeValue(attr.pos, attr.name, attr.value));
      }
      ef (attr.name == "rtl") {
        if (now->is<decls::VarDecl>())
          return make_err(errors::AttributeNotSupported(attr.pos, attr.name));
        if (!attr.value.empty())
          return make_err(errors::InvalidAttributeValue(attr.pos, attr.name, attr.value));
      }
      ef (attr.name == "weak") {
        if (!attr.value.empty())
          return make_err(errors::InvalidAttributeValue(attr.pos, attr.name, attr.value));
      }
      ef (attr.name == "calling") {
        if (now->is<decls::VarDecl>())
          return make_err(errors::AttributeNotSupported(attr.pos, attr.name));
        if (attr.value != "fast" && attr.value != "cdecl" && attr.value != "cold")
          return make_err(errors::InvalidAttributeValue(attr.pos, attr.name, attr.value));
      }
      ef (attr.name == "on") {
        if (now->is<decls::VarDecl>())
          return make_err(errors::AttributeNotSupported(attr.pos, attr.name));
        if (attr.value != "init" && attr.value != "fini")
          return make_err(errors::InvalidAttributeValue(attr.pos, attr.name, attr.value));
      }
      else
        return make_err(errors::UnknownAttribute(attr.pos, attr.name));
    }
    
    return {};
  }

  fun DeclSema::sema_TypeDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    if_except(sctx.type->sema_Type(now->as<decls::TypeDecl>()->type, now->pos()));

    return {};
  }

  fun DeclSema::sema_FuncDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto fdecl = now->as<decls::FuncDecl>();
    types::Type *ftype = fdecl->funcType;

    if_except(sctx.type->sema_FuncType(ftype, now->pos()));
    
    bool is_rtl = false;
    word rtl_pos;
    for (auto &attr: now->const_attrs()) {
      if (attr.name == "rtl") {
        is_rtl = true;
        rtl_pos = attr.pos;
        break;
      }
    }

    if (is_rtl && now->pos().mod() && !now->pos().mod()->is_rtl()) {
      u32 start_off = rtl_pos.off();
      u32 end_off = now->pos().off() + now->pos().size();
      word ctx_pos(now->pos().mod(), start_off, end_off - start_off);
      auto err = errors::RtlAttributeOnlyInRtlModule(rtl_pos);
      err.error()->ctx() = ctx_pos;
      return err;
    }

    bool skip_gst = false;
    if (is_rtl) {
      decls::Decl* intrinsic_decl = nullptr;
      if (now->parent()->type() == IdentyEnum::Decl) {
        auto parent_decl = (decls::Decl*)now->parent();
        if (parent_decl->is<decls::NameSpaceDecl>()) {
          auto parent_ns = parent_decl->as<decls::NameSpaceDecl>();
          for (auto decl : parent_ns->decls) {
            if (decl != now && decl->is<decls::FuncDecl>() && decl->name() == now->name()) {
              bool is_other_rtl = false;
              for (auto &attr : decl->const_attrs()) {
                if (attr.name == "rtl") {
                  is_other_rtl = true;
                  break;
                }
              }

              if (is_other_rtl) {
                if (decl->pos().off() < now->pos().off()) {
                  auto err = errors::RtlCannotBeOverloaded(now->pos(), std::string(now->name()));
                  err.error()->notes().push_back(diagnostic::note::make(decl->pos(), _("first definition is here.")));
                  return err;
                }
                else
                  continue;
              }

              if (decl->as<decls::FuncDecl>()->body != nullptr) continue;

              intrinsic_decl = decl;
              break;
            }
          }
        }
      }


      if (intrinsic_decl) {
        auto intrinsic_fdecl = intrinsic_decl->as<decls::FuncDecl>();
        intrinsic_fdecl->body = fdecl->body;
        fdecl->body = nullptr;
        if (intrinsic_fdecl->body) {
          intrinsic_fdecl->body->parent() = intrinsic_decl;
        }
        if (intrinsic_fdecl->funcType->is<types::FuncType>() && ftype->is<types::FuncType>()) {
          auto int_ftype = intrinsic_fdecl->funcType->as<types::FuncType>();
          auto parsed_ftype = ftype->as<types::FuncType>();
          if (int_ftype->pars.size() == parsed_ftype->pars.size()) {
            for (size_t i = 0; i < int_ftype->pars.size(); i++) {
              int_ftype->pars[i].name = parsed_ftype->pars[i].name;
            }
          }
        }
        
        now = intrinsic_decl;
        fdecl = intrinsic_fdecl;
        ftype = fdecl->funcType;
        skip_gst = true;
      }
    }

    if (!skip_gst)
      ctx->gst().add_ident(scopemng::mangling_abi_qw(now), now);

    if (fdecl->body) {
      auto block         = fdecl->body->as<stmts::CodeBlock>();
      auto ftype_concrete = ftype->as<types::FuncType>();

      for (const auto &p: ftype_concrete->pars) {
        bool exists = false;
        for (auto v: block->vars)
          if (v->as<stmts::CodeVar>()->name == p.name) {
            exists = true;
            break;
          }

        if (!exists) {
          stmts::Stmt::make_CodeVar(ctx, fdecl->body, p.name, p.type, fdecl->body->pos());
        }
      }

      if_except(sctx.stmt->sema_CodeBlock(fdecl->body, ftype_concrete->ret));
    }

    for (auto &attr: now->const_attrs()) {
      if (attr.name == "on") {
        auto ftype_concrete = ftype->as<types::FuncType>();
        bool returns_void = false;
        if (ftype_concrete->ret->is<types::PrimitiveType>()) {
          returns_void = ftype_concrete->ret->as<types::PrimitiveType>()->kind == types::PrimitiveEnum::Void;
        }

        if (!ftype_concrete->pars.empty() || !returns_void) {
          auto make_err = [&](auto err) {
            u32 start_off = attr.pos.off();
            u32 end_off = now->pos().off() + now->pos().size();
            word ctx_pos(now->pos().mod(), start_off, end_off - start_off);
            err.error()->ctx() = ctx_pos;
            return err;
          };
          return make_err(errors::InvalidOnAttributeSignature(attr.pos));
        }

        if (attr.value == "init") {
          now->pos().mod()->global_ctors().push_back(fdecl);
        }
        ef (attr.value == "fini") {
          now->pos().mod()->global_dtors().push_back(fdecl);
        }
      }
    }

    return {};
  }

  fun DeclSema::sema_ConstructorDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto cdecl = now->as<decls::ConstructorDecl>();
    types::Type *ctype = cdecl->funcType;

    if_except(sctx.type->sema_FuncType(ctype, now->pos()));
    
    ctx->gst().add_ident(scopemng::mangling_abi_qw(now), now);

    auto ftype_concrete = ctype->as<types::FuncType>();
    
    // Check inits
    if (ftype_concrete->pars.size() > 0) {
      auto self_ref = ftype_concrete->pars[0].type->as<types::ReferenceType>();
      if (self_ref && self_ref->sub->is<types::StructType>()) {
        auto recType = self_ref->sub->as<types::StructType>();

        for (auto &init_pair : cdecl->inits) {
          bool found = false;
          types::Type *target_type = nullptr;
          for (auto &v : recType->vars) {
            if (v.name == init_pair.first) {
              found = true;
              target_type = v.type;
              break;
            }
          }
          
          if (!found) {
            return errors::IdentifierNotFound(now->pos(), init_pair.first);
          }

          if_except(sctx.expr->sema_Expr(init_pair.second));
          if_except(sctx.expr->sema_Convert(target_type, init_pair.second, now->pos()));
        }
      }
    }

    if (cdecl->body) {
      auto block = cdecl->body->as<stmts::CodeBlock>();

      for (const auto &p: ftype_concrete->pars) {
        bool exists = false;
        for (auto v : block->vars) {
          if (v->as<stmts::CodeVar>()->name == p.name) {
            exists = true;
            break;
          }
        }
        if (!exists) {
          stmts::Stmt::make_CodeVar(ctx, cdecl->body, p.name, p.type, cdecl->body->pos());
        }
      }

      if_except(sctx.stmt->sema_CodeBlock(cdecl->body, ftype_concrete->ret));
    }

    return {};
  }

  fun DeclSema::sema_DestructorDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto cdecl = now->as<decls::DestructorDecl>();
    types::Type *ctype = cdecl->funcType;

    if_except(sctx.type->sema_FuncType(ctype, now->pos()));
    
    ctx->gst().add_ident(scopemng::mangling_abi_qw(now), now);

    auto ftype_concrete = ctype->as<types::FuncType>();

    if (cdecl->body) {
      auto block = cdecl->body->as<stmts::CodeBlock>();

      for (const auto &p: ftype_concrete->pars) {
        bool exists = false;
        for (auto v: block->vars) {
          if (v->as<stmts::CodeVar>()->name == p.name) {
            exists = true;
            break;
          }
        }
        if (!exists) {
          stmts::Stmt::make_CodeVar(ctx, cdecl->body, p.name, p.type, cdecl->body->pos());
        }
      }

      if_except(sctx.stmt->sema_CodeBlock(cdecl->body, ftype_concrete->ret));
    }

    return {};
  }

  fun DeclSema::sema_VarDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto var = now->as<decls::VarDecl>();
    if (var->type)
      if_except(sctx.type->sema_Type(var->type, now->pos()));
    
    if (var->initer) {
        if_except(sctx.expr->sema_Expr(var->initer));
        if (!var->type) {
            var->type = var->initer->targetType();
            if (!var->type)
                return std::unexpected(errors::TypeCannotBeInferred(now->pos(), std::string(now->name())));
        } else {
            if_except(sctx.expr->sema_Convert(var->type, var->initer, var->initer->pos()));
        }
    }
    return {};
  }

}
