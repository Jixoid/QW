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

  inline fun inherits_from(types::Type* derived, types::Type* base) -> bool {
    if (derived == base || derived->typname() == base->typname()) 
      return true;

    std::vector<types::Type*> search_queue;
    search_queue.push_back(derived);

    for (size_t q = 0; q < search_queue.size(); ++q) {
      auto current = search_queue[q];
      if (current == base || current->typname() == base->typname()) {
        return true;
      }

      if (current->is<types::StructType>()) {
        auto strct = current->as<types::StructType>();
        for (auto bt: strct->baseTypes) {
          auto rbt = bt;
          while (rbt->isReference()) rbt = rbt->as<types::ReferenceType>()->sub;
          search_queue.push_back(rbt);
        }
      } 
      ef (current->is<types::IFaceType>()) {
        auto iface = current->as<types::IFaceType>();
        for (auto bt: iface->baseTypes) {
          auto rbt = bt;
          while (rbt->isReference()) rbt = rbt->as<types::ReferenceType>()->sub;
          search_queue.push_back(rbt);
        }
      }
    }
    return false;
  }

  fun ExprSema::sema_Convert(types::Type *typ, exprs::Expr *val, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    auto styp = val->targetType();

    if (typ == styp) return {};

    if (!typ->isReference() && styp->isReference()) styp = styp->as<types::ReferenceType>()->sub;

    if (typ == styp) return {};

    if (typ->is<types::PointerType>() && styp->is<types::PointerType>()) {
      auto t_sub = typ->as<types::PointerType>()->sub;
      auto s_sub = styp->as<types::PointerType>()->sub;

      if (inherits_from(s_sub, t_sub)) return {};
    }

    if (styp == ctx->null_t() && (typ->is<types::PointerType>() || typ == ctx->ptr_t())) return {};

    if (typ->isReference() && styp->isReference()) {
      auto t_sub = typ->as<types::ReferenceType>()->sub;
      auto s_sub = styp->as<types::ReferenceType>()->sub;

      if (inherits_from(s_sub, t_sub)) return {};
    }

    if (!typ->isReference() && styp->isReference()) styp = styp->as<types::ReferenceType>()->sub;

    if (typ->is<types::IFaceType>() && styp->is<types::StructType>()) {
      if (inherits_from(styp, typ)) return {};
    }
    
    if (typ->is<types::IFaceType>() && styp->is<types::IFaceType>()) {
      if (inherits_from(styp, typ)) return {};
    }

    if (typ == styp) return {};

    if (sctx.meta->unsafe_level > 0) {
      bool is_typ_ptr = typ->is<types::PointerType>() || typ == ctx->ptr_t();
      bool is_styp_ptr = styp->is<types::PointerType>() || styp == ctx->ptr_t();

      if (is_typ_ptr && is_styp_ptr) return {};

      if (is_typ_ptr && styp->is<types::PrimitiveType>() && styp->as<types::PrimitiveType>()->kind == types::PrimitiveEnum::U64)
        return {};
    }

    if (typ->isInteger() && styp->isInteger()) return {};
    ef (typ->isFloat() && styp->isFloat()) return {};
    
    return errors::NoMatchOperator(errpos, "=", std::string(typ->typname()), std::string(val->targetType()->typname()));
  }

  fun ExprSema::sema_Expr(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    begin:

    if (now->is<exprs::IntegerLiteral>()) {
      if (!now->targetType()) {
        auto lit = now->as<exprs::IntegerLiteral>();
        if (std::holds_alternative<u64>(lit->val)) {
          u64 v = std::get<u64>(lit->val);
          if (v <= 2147483647) now->targetType() = ctx->intS32_t();
          ef (v <= 4294967295ULL) now->targetType() = ctx->intU32_t();
          ef (v <= 9223372036854775807ULL) now->targetType() = ctx->intS64_t();
          ef (v <= 18446744073709551615ULL) now->targetType() = ctx->intU64_t();
          else now->targetType() = ctx->intU128_t();
        }
        else {
          i64 v = std::get<i64>(lit->val);
          if (v >= -2147483648LL && v <= 2147483647LL) now->targetType() = ctx->intS32_t();
          ef (v >= -9223372036854775807LL - 1 && v <= 9223372036854775807LL) now->targetType() = ctx->intS64_t();
          else now->targetType() = ctx->intS128_t();
        }
      }
    }
    ef (now->is<exprs::FloatingLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->flo32_t();
    }
    ef (now->is<exprs::CharLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->char_t();
    }
    ef (now->is<exprs::BoolLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->bool_t();
    }
    ef (now->is<exprs::PtrLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->null_t();
    }
    ef (now->is<exprs::StringLiteral>()) {
      if (!now->targetType())
        now->targetType() = types::Type::make_Pointer(ctx, ctx->char_t());
    }
    ef (now->is<exprs::VarExpr>()) {
      auto C = now->as<exprs::VarExpr>();

      if (C->var->type() == IdentyEnum::Stmt) {
        auto cvar = static_cast<stmts::Stmt*>(C->var)->as<stmts::CodeVar>();
        if (cvar->targetType)
          now->targetType() = types::Type::make_Reference(ctx, cvar->targetType);
        else
          now->targetType() = nullptr;
      }
      ef (C->var->type() == IdentyEnum::Decl) {
        auto vdecl = static_cast<decls::Decl*>(C->var)->as<decls::VarDecl>();
        now->targetType() = types::Type::make_Reference(ctx, vdecl->type);
      }
    }

    ef (now->is<exprs::UnaryOp>())   return sctx.expr->sema_Expr_UnaryOp(now);
    ef (now->is<exprs::BinaryOp>())  return sctx.expr->sema_Expr_BinaryOp(now);
    ef (now->is<exprs::MemberOp>())  return sctx.expr->sema_Expr_MemberOp(now);
    ef (now->is<exprs::PostfixOp>()) return sctx.expr->sema_Expr_PostfixOp(now);
    ef (now->is<exprs::ValExpr>())   return {};

    ef (now->is<exprs::NickExpr>()) {
      auto ret = sctx.expr->sema_NickExpr(now);

      if (ret.has_value()) {
        now = *ret;
        goto begin;
      }
      else
        return std::unexpected(std::move(ret.error()));
    }
    else
      diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());

    return {};
  }

  fun ExprSema::sema_Expr_UnaryOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->targetType()) return {};
    auto U = now->as<exprs::UnaryOp>();
    if_except(sctx.expr->sema_Expr(U->o1));

    if (U->kind == exprs::UnaryOpEnum::AddrOf) {
      if (!U->o1->targetType()->isReference())
        return errors::NoMatchOperator(now->pos(), "@", std::string(U->o1->targetType()->typname()), "operand must be an lvalue");
      
      auto base_type = U->o1->targetType()->as<types::ReferenceType>()->sub;
      now->targetType() = types::Type::make_Pointer(ctx, base_type);
    }
    ef (U->kind == exprs::UnaryOpEnum::Minus || U->kind == exprs::UnaryOpEnum::Plus) {
      auto t = U->o1->targetType();
      if (t->isReference()) t = t->as<types::ReferenceType>()->sub;
      if (!t->isInteger() && !t->isFloat())
        return errors::NoMatchOperator(now->pos(), U->kind == exprs::UnaryOpEnum::Minus ? "-" : "+", std::string(t->typname()), "operand must be numeric");
      
      now->targetType() = t;
    }
    ef (U->kind == exprs::UnaryOpEnum::LNot) {
      auto t = U->o1->targetType();
      if (t->isReference()) t = t->as<types::ReferenceType>()->sub;
      if (t->typname() != ctx->bool_t()->typname())
        return errors::NoMatchOperator(now->pos(), "!", std::string(t->typname()), "operand must be boolean");
      
      now->targetType() = t;
    }
    ef (U->kind == exprs::UnaryOpEnum::BitNot) {
      auto t = U->o1->targetType();
      if (t->isReference()) t = t->as<types::ReferenceType>()->sub;
      if (!t->isInteger() && !t->is<types::SetType>())
        return errors::NoMatchOperator(now->pos(), "~", std::string(t->typname()), "operand must be an integer or a set");
      
      now->targetType() = t;
    }

    return {};
  }

  fun ExprSema::sema_Expr_BinaryOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto C = now->as<exprs::BinaryOp>();

    bool is_compound_assign = (C->kind >= exprs::BinaryOpEnum::AddAssign && C->kind <= exprs::BinaryOpEnum::ShrAssign);

    if (C->kind == exprs::BinaryOpEnum::Assign || is_compound_assign) {
      if_except(sctx.expr->sema_Expr(C->o2));
      if_except(sctx.expr->sema_Expr(C->o1));

      if (C->kind == exprs::BinaryOpEnum::Assign && C->o1->is<exprs::VarExpr>()) {
        auto vexpr = C->o1->as<exprs::VarExpr>();
        if (!C->o1->targetType()) {
          auto stmt = static_cast<stmts::Stmt*>(vexpr->var);
          auto cvar = stmt->as<stmts::CodeVar>();
          cvar->targetType = C->o2->targetType();
          C->o1->targetType() = types::Type::make_Reference(ctx, cvar->targetType);
        }
      }

      if (!C->o1->targetType()->isReference())
        return errors::NoMatchOperator(now->pos(), is_compound_assign ? "compound assignment" : "=", std::string(C->o1->targetType()->typname()), "left-hand side of assignment must be an lvalue");

      auto lhs_concrete_type = C->o1->targetType()->as<types::ReferenceType>()->sub;

      if (is_compound_assign) {
        if (!lhs_concrete_type->isInteger() && !lhs_concrete_type->isFloat())
          return errors::NoMatchOperator(now->pos(), "compound assignment", std::string(lhs_concrete_type->typname()), "left-hand side must be numeric");
        if (lhs_concrete_type->isFloat() && (C->kind >= exprs::BinaryOpEnum::RemAssign))
          return errors::NoMatchOperator(now->pos(), "compound assignment", std::string(lhs_concrete_type->typname()), "bitwise or remainder operator not allowed for float");
        
        C->computationType = lhs_concrete_type;
      }

      if_except(sctx.expr->sema_Convert(lhs_concrete_type, C->o2, now->pos()));

      now->targetType() = lhs_concrete_type;
      return {};
    }

    if_except(sctx.expr->sema_Expr(C->o1));
    if_except(sctx.expr->sema_Expr(C->o2));

    auto t1 = C->o1->targetType(), t2 = C->o2->targetType();

    while (t1->isReference() || t2->isReference()) {
      if (t1->isReference()) t1 = t1->as<types::ReferenceType>()->sub;
      if (t2->isReference()) t2 = t2->as<types::ReferenceType>()->sub;
    }

    if (t1->isInteger() && t2->isInteger()) {
      types::Type *target{};

      if (t1 != t2)
        target = (t1->intBit(ctx) != t2->intBit(ctx)) ? (t1->intBit(ctx) > t2->intBit(ctx) ? t1 : t2) : (!t1->isSigned() ? t1 : t2);
      else
        target = t1;

      if (C->kind == exprs::BinaryOpEnum::Eq || C->kind == exprs::BinaryOpEnum::NEq ||
          C->kind == exprs::BinaryOpEnum::Lt || C->kind == exprs::BinaryOpEnum::Gt ||
          C->kind == exprs::BinaryOpEnum::LEq || C->kind == exprs::BinaryOpEnum::GEq
      ) {
        C->computationType = target;
        now->targetType() = ctx->bool_t();
      }
      else {
        now->targetType() = target;
      }
      return {};
    }
    ef (t1->isFloat() && t2->isFloat()) {
      types::Type *target{};

      if (t1 != t2)
        target = (t1->as<types::PrimitiveType>()->kind > t2->as<types::PrimitiveType>()->kind) ? t1 : t2;
      else
        target = t1;

      if (C->kind == exprs::BinaryOpEnum::Eq || C->kind == exprs::BinaryOpEnum::NEq ||
          C->kind == exprs::BinaryOpEnum::Lt || C->kind == exprs::BinaryOpEnum::Gt ||
          C->kind == exprs::BinaryOpEnum::LEq || C->kind == exprs::BinaryOpEnum::GEq
      ) {
        C->computationType = target;
        now->targetType() = ctx->bool_t();
      }
      else {
        now->targetType() = target;
      }
      return {};
    }

    
    if (t1->is<types::EnumType>() && t1 == t2) {
      if (C->kind == exprs::BinaryOpEnum::Eq || C->kind == exprs::BinaryOpEnum::NEq) {
        C->computationType = t1;
        now->targetType() = ctx->bool_t();
        return {};
      }
    }
    
    if (t1->is<types::SetType>() && t1 == t2) {
      if (C->kind == exprs::BinaryOpEnum::Eq || C->kind == exprs::BinaryOpEnum::NEq) {
        C->computationType = t1;
        now->targetType() = ctx->bool_t();
      }
      ef (C->kind == exprs::BinaryOpEnum::BitOr || C->kind == exprs::BinaryOpEnum::BitAnd || C->kind == exprs::BinaryOpEnum::BitXor ||
          C->kind == exprs::BinaryOpEnum::BitOrAssign || C->kind == exprs::BinaryOpEnum::BitAndAssign || C->kind == exprs::BinaryOpEnum::BitXorAssign
      ) {
        C->computationType = t1;
        now->targetType() = t1;
      }

      return {};
    }

    bool is_ptr1 = t1->is<types::PointerType>() || t1 == ctx->ptr_t() || t1 == ctx->null_t();
    bool is_ptr2 = t2->is<types::PointerType>() || t2 == ctx->ptr_t() || t2 == ctx->null_t();

    if (is_ptr1 && is_ptr2) {
      if (C->kind == exprs::BinaryOpEnum::Eq || C->kind == exprs::BinaryOpEnum::NEq) {
        C->computationType = ctx->ptr_t();
        now->targetType() = ctx->bool_t();
        return {};
      }
    }

    std::unordered_map<exprs::BinaryOpEnum, std::string> Ops = {
      {exprs::BinaryOpEnum::Add, "+"}, {exprs::BinaryOpEnum::Sub, "-"}, {exprs::BinaryOpEnum::Mul, "*"},
      {exprs::BinaryOpEnum::Div, "/"}, {exprs::BinaryOpEnum::Rem, "%"},
      {exprs::BinaryOpEnum::Eq, "=="}, {exprs::BinaryOpEnum::NEq, "!="},
      {exprs::BinaryOpEnum::Lt, "<"}, {exprs::BinaryOpEnum::Gt, ">"},
      {exprs::BinaryOpEnum::LEq, "<="}, {exprs::BinaryOpEnum::GEq, ">="},
    };

    return errors::NoMatchOperator(now->pos(), Ops[C->kind], std::string(C->o1->targetType()->typname()), std::string(C->o2->targetType()->typname()));
  }

  fun ExprSema::sema_Expr_MemberOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto M = now->as<exprs::MemberOp>();

    if (M->kind == exprs::MemberOpEnum::NameS && M->obj->is<exprs::NickExpr>()) {
      auto nick = M->obj->as<exprs::NickExpr>();
      auto ret = SMng.lookup(now->parent(), nick->unresolved);
      if (ret && ret->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(ret)->is<decls::TypeDecl>()) {
        auto typeDecl = static_cast<decls::Decl*>(ret)->as<decls::TypeDecl>();
        if (typeDecl->type->is<types::EnumType>()) {
          auto enum_t = typeDecl->type->as<types::EnumType>();
          auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];

          for (auto &v: enum_t->vals)
            if (v.cons == field_name) {
              std::variant<u64, i64> val128;
              if (std::holds_alternative<u64>(v.val)) val128 = (u64)std::get<u64>(v.val);
              else val128 = (i64)std::get<i64>(v.val);
              now->vari() = exprs::IntegerLiteral{ val128 };
              now->targetType() = typeDecl->type;
              return {};
            }
          
          return errors::IdentifierNotFound(M->mem->pos(), field_name);
        }
        ef (typeDecl->type->is<types::SetType>()) {
          auto set_t = typeDecl->type->as<types::SetType>();
          auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];
          for (auto &v: set_t->vals)
            if (v.cons == field_name) {
              std::variant<u64, i64> val128;
              if (std::holds_alternative<u64>(v.val)) val128 = (u64)std::get<u64>(v.val);
              else val128 = (i64)std::get<i64>(v.val);
              now->vari() = exprs::IntegerLiteral{ val128 };
              now->targetType() = typeDecl->type;
              return {};
            }
          
          return errors::IdentifierNotFound(M->mem->pos(), field_name);
        }
      }
    }

    if_except(sctx.expr->sema_Expr(M->obj));

    auto obj_type = M->obj->targetType();

    while (obj_type->isReference()) {
      obj_type = obj_type->as<types::ReferenceType>()->sub;
    }

    if (!obj_type->is<types::StructType>()) return errors::NoMatchOperator(now->pos(), M->kind == exprs::MemberOpEnum::NameS ? "::" : ".", std::string(M->obj->targetType()->typname()), M->mem->as<exprs::NickExpr>()->unresolved[0]);
    
    auto rec_type   = obj_type->as<types::StructType>();
    auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];

    bool found{};
    for (size_t i = 0; i < rec_type->vars.size(); i++) {
      if (rec_type->vars[i].name == field_name) {
        now->targetType() = types::Type::make_Reference(ctx, rec_type->vars[i].type);
        found             = true;
        break;
      }
    }

    if (!found)
      return errors::IdentifierNotFound(M->mem->pos(), field_name);
    else
      return {};
  }

  fun ExprSema::sema_Expr_PostfixOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto M = now->as<exprs::PostfixOp>();

    if (M->kind == exprs::PostfixOpEnum::Deref) {
      if_except(sctx.expr->sema_Expr(M->obj));
      auto t = M->obj->targetType();
      if (t->isReference()) t = t->as<types::ReferenceType>()->sub;

      if (!t->is<types::PointerType>()) return errors::NoMatchOperator(now->pos(), "?", std::string(t->typname()), "operand must be a pointer");
    
      auto base_type = t->as<types::PointerType>()->sub;
      now->targetType() = types::Type::make_Reference(ctx, base_type);
      return {};
    }
    ef (M->kind == exprs::PostfixOpEnum::Call)  return sctx.expr->sema_Expr_PostfixOp_Call(now);
    ef (M->kind == exprs::PostfixOpEnum::Array) return sctx.expr->sema_Expr_PostfixOp_Array(now);
    else
      diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
  }

  fun ExprSema::sema_Expr_PostfixOp_Call(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto M = now->as<exprs::PostfixOp>();
    bool is_sys = false;
    std::string sys_name;
    std::vector<types::Type*> sys_gargs;
    
    auto check_sys_nick = [&](exprs::Expr *E) {
      if (E->is<exprs::NickExpr>()) {
        auto nick = E->as<exprs::NickExpr>();
        if (nick->unresolved.size() == 2 && nick->unresolved[0] == "sys" && nick->unresolved[1] != "syscall") {
          is_sys = true;
          sys_name = nick->unresolved[1];
        }
      }
      ef (E->is<exprs::MemberOp>()) {
        auto memOp = E->as<exprs::MemberOp>();
        if (memOp->kind == exprs::MemberOpEnum::NameS) {
          if (memOp->obj->is<exprs::NickExpr>() && memOp->mem->is<exprs::NickExpr>()) {
            auto n1 = memOp->obj->as<exprs::NickExpr>();
            auto n2 = memOp->mem->as<exprs::NickExpr>();
            if (n1->unresolved.size() == 1 && n1->unresolved[0] == "sys" && n2->unresolved.size() == 1) {
              is_sys = true;
              sys_name = n2->unresolved[0];
            }
          }
        }
      }
    };

    if (M->obj->is<exprs::GenericOp>()) {
      auto gop = M->obj->as<exprs::GenericOp>();
      check_sys_nick(gop->obj);
      if (is_sys) sys_gargs = gop->args;
    }
    else
      check_sys_nick(M->obj);
    
    if (is_sys)
      return sctx.expr->sema_SysIntrinsic(now, sys_name, sys_gargs, M->operands);

    if (M->obj->is<exprs::MemberOp>()) {
      auto memOp = M->obj->as<exprs::MemberOp>();
      if_except(sctx.expr->sema_Expr(memOp->obj));

      auto obj_type = memOp->obj->targetType();
      while (obj_type->isReference()) obj_type = obj_type->as<types::ReferenceType>()->sub;

      if (obj_type->is<types::StructType>()) {
        auto rec_type    = obj_type->as<types::StructType>();
        auto member_name = memOp->mem->as<exprs::NickExpr>()->unresolved[0];

        for (size_t i = 0; i < M->operands.size(); i++) if_except(sctx.expr->sema_Expr(M->operands[i]));

        std::vector<types::Type *> arg_types;
        arg_types.push_back(memOp->obj->targetType());
        for (auto op : M->operands) arg_types.push_back(op->targetType());

        decls::Decl *found_func = sctx.expr->sema_Resolve_StructMethod(rec_type, member_name, arg_types);

        if (found_func) {
          auto fdecl = found_func->as<decls::FuncDecl>();
          auto ftype = fdecl->funcType->as<types::FuncType>();
          M->operands.insert(M->operands.begin(), memOp->obj);

          for (size_t i = 0; i < M->operands.size(); ++i) {
            if_except(sctx.expr->sema_Expr(M->operands[i]));
            if_except(sctx.expr->sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
          }

          now->targetType()    = ftype->ret;
          M->obj->targetType() = fdecl->funcType;
          return {};
        }

        return errors::NoMatchOperator(now->pos(), "call", member_name, "no matching method overload found");
      }
      ef (obj_type->is<types::IFaceType>()) {
        auto iface_type  = obj_type->as<types::IFaceType>();
        auto member_name = memOp->mem->as<exprs::NickExpr>()->unresolved[0];

        for (size_t i = 0; i < M->operands.size(); i++) if_except(sctx.expr->sema_Expr(M->operands[i]));

        std::vector<types::Type *> arg_types;
        arg_types.push_back(memOp->obj->targetType());
        for (auto op : M->operands) arg_types.push_back(op->targetType());

        decls::Decl *found_func = sctx.expr->sema_Resolve_IFaceMethod(iface_type, member_name, arg_types);

        if (found_func) {
          auto fdecl = found_func->as<decls::FuncDecl>();
          auto ftype = fdecl->funcType->as<types::FuncType>();
          M->operands.insert(M->operands.begin(), memOp->obj);

          for (size_t i = 0; i < M->operands.size(); ++i) {
            if_except(sctx.expr->sema_Expr(M->operands[i]));
            if_except(sctx.expr->sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
          }

          now->targetType()    = ftype->ret;
          M->obj->targetType() = fdecl->funcType;
          return {};
        }

        return errors::NoMatchOperator(now->pos(), "call", member_name, "no matching method overload found");
      }
    }

    if (M->obj->is<exprs::NickExpr>()) {
      auto nick = M->obj->as<exprs::NickExpr>();

      for (size_t i = 0; i < M->operands.size(); ++i) {
        if_except(sctx.expr->sema_Expr(M->operands[i]));
      }

      std::vector<types::Type *> arg_types;
      for (auto op : M->operands)
        arg_types.push_back(op->targetType());

      auto ret = SMng.lookup(now->parent(), nick->unresolved, &arg_types);
      if (!ret) ret = SMng.lookup(now->parent(), nick->unresolved, nullptr);
      
      if (ret && ret->type() == IdentyEnum::Decl && static_cast<decls::Decl *>(ret)->is<decls::FuncDecl>()) {
        auto fdecl = static_cast<decls::Decl *>(ret)->as<decls::FuncDecl>();
        auto ftype = fdecl->funcType->as<types::FuncType>();

        for (size_t i = 0; i < M->operands.size(); ++i) {
          if_except(sctx.expr->sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
        }

        now->targetType()    = ftype->ret;
        M->obj->targetType() = fdecl->funcType;
        return {};
      }
    }

    if_except(sctx.expr->sema_Expr(M->obj));
    auto callee_type = M->obj->targetType();

    if (callee_type->is<types::FuncType>()) {
      auto ftype = callee_type->as<types::FuncType>();
      if (M->operands.size() != ftype->pars.size()) return errors::NoMatchOperator(now->pos(), "call", "function pointer", "arguments count mismatch");
      
      for (size_t i = 0; i < M->operands.size(); ++i) {
        if_except(sctx.expr->sema_Expr(M->operands[i]));
        if_except(sctx.expr->sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
      }
      now->targetType() = ftype->ret;
      return {};
    }

    return errors::NoMatchOperator(now->pos(), "call", std::string(M->obj->targetType()->typname()), "not a callable type");
  }

  fun ExprSema::sema_Expr_PostfixOp_Array(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto M = now->as<exprs::PostfixOp>();
    if_except(sctx.expr->sema_Expr(M->obj));

    auto obj_type = M->obj->targetType();
    while (obj_type->isReference()) {
      obj_type = obj_type->as<types::ReferenceType>()->sub;
    }

    if (M->operands.size() != 1) return errors::NoMatchOperator(now->pos(), "[]", std::string(M->obj->targetType()->typname()), "dimension mismatch");

    if_except(sctx.expr->sema_Expr(M->operands[0]));
    if (!M->operands[0]->targetType()->isInteger()) {
      return errors::NoMatchOperator(
        M->operands[0]->pos(), "[]", std::string(M->operands[0]->targetType()->typname()), "array index must be an integer"
      );
    }

    types::Type *sub_type = nullptr;
    if (obj_type->is<types::PArrayType>()) {
      sub_type = obj_type->as<types::PArrayType>()->sub;
    }
    ef (obj_type->is<types::ZArrayType>()) {
      sub_type = obj_type->as<types::ZArrayType>()->sub;
    }
    else
      return errors::NoMatchOperator(now->pos(), "[]", std::string(M->obj->targetType()->typname()), "not an array type");

    now->targetType() = types::Type::make_Reference(ctx, sub_type);
    return {};
  }


  fun ExprSema::sema_Resolve_StructMethod(types::StructType *rec_type, const std::string &member_name, const std::vector<types::Type*> &arg_types) -> decls::Decl* {
    std::vector<types::StructType*> search_queue;
    search_queue.push_back(rec_type);
    
    for (size_t q = 0; q < search_queue.size(); q++) {
      auto search_rec = search_queue[q];
      if (!search_rec->decl) continue;

      for (auto &F: search_rec->decl->as<decls::StructDecl>()->func) {
        if (F->name() != member_name) continue;

        auto ftype = F->as<decls::FuncDecl>()->funcType->as<types::FuncType>();
        if (ftype->pars.size() == arg_types.size()) {
          bool match = true;
          for (size_t i = 0; i < arg_types.size(); i++) {
            auto t1 = ftype->pars[i].type;
            auto t2 = arg_types[i];
            if (t1->isReference()) t1 = t1->as<types::ReferenceType>()->sub;
            if (t2->isReference()) t2 = t2->as<types::ReferenceType>()->sub;
            if (t1->typname() != t2->typname()) {
              match = false;
              break;
            }
          }
          
          if (match) return F;
        }
      }

      for (auto bt: search_rec->baseTypes) {
        auto rbt = bt;
        while(rbt->isReference()) rbt = rbt->as<types::ReferenceType>()->sub;
        if (rbt->is<types::StructType>()) {
          search_queue.push_back(rbt->as<types::StructType>());
        }
      }
    }
    return nullptr;
  }

  fun ExprSema::sema_Resolve_IFaceMethod(types::IFaceType *iface_type, const std::string &member_name, const std::vector<types::Type*> &arg_types) -> decls::Decl* {
    std::vector<types::IFaceType*> search_queue;
    search_queue.push_back(iface_type);
    
    for (size_t q = 0; q < search_queue.size(); q++) {
      auto search_iface = search_queue[q];
      if (!search_iface->decl) continue;

      for (auto &F: search_iface->decl->as<decls::IFaceDecl>()->func) {
        if (F->name() != member_name) continue;

        auto ftype = F->as<decls::FuncDecl>()->funcType->as<types::FuncType>();
        if (ftype->pars.size() == arg_types.size()) {
          bool match = true;
          for (size_t i = 0; i < arg_types.size(); i++) {
            auto t1 = ftype->pars[i].type;
            auto t2 = arg_types[i];
            if (t1->isReference()) t1 = t1->as<types::ReferenceType>()->sub;
            if (t2->isReference()) t2 = t2->as<types::ReferenceType>()->sub;
            if (t1->typname() != t2->typname()) {
              match = false;
              break;
            }
          }
          
          if (match) return F;
        }
      }

      for (auto bt: search_iface->baseTypes) {
        auto rbt = bt;
        while(rbt->isReference()) rbt = rbt->as<types::ReferenceType>()->sub;
        if (rbt->is<types::IFaceType>()) {
          search_queue.push_back(rbt->as<types::IFaceType>());
        }
      }
    }
    return nullptr;
  }


  fun ExprSema::sema_NickExpr(exprs::Expr *now) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
    auto nick = now->as<exprs::NickExpr>();
    
    const auto join_identifier_path = [](const std::vector<std::string> &ps) -> std::string {
      std::string ret = ps[0];
      for (size_t i = 1; i < ps.size(); i++)
        ret += "::" + ps[i];
      return ret;
    };

    auto ret = SMng.lookup(now->parent(), nick->unresolved);

    if (!ret && nick->unresolved.size() > 1) {
      std::vector<std::string> parent_nick = nick->unresolved;
      std::string field_name = parent_nick.back();
      parent_nick.pop_back();

      auto parent_ret = SMng.lookup(now->parent(), parent_nick);
      if (parent_ret && parent_ret->type() == IdentyEnum::Decl && static_cast<decls::Decl *>(parent_ret)->is<decls::TypeDecl>()) {
        auto typeDecl = static_cast<decls::Decl *>(parent_ret)->as<decls::TypeDecl>();

        if (typeDecl->type->is<types::EnumType>()) {
          auto enum_t = typeDecl->type->as<types::EnumType>();
          for (auto &v: enum_t->vals)
            if (v.cons == field_name) {
              exprs::Expr *lit = nullptr;
              if (std::holds_alternative<u64>(v.val)) {
                lit = exprs::Expr::make_IntegerLiteral(ctx, now->parent(), (u64)std::get<u64>(v.val), now->pos());
              } else {
                lit = exprs::Expr::make_IntegerLiteral(ctx, now->parent(), (i64)std::get<i64>(v.val), now->pos());
              }
              lit->targetType() = typeDecl->type;
              return lit;
            }
        }
        ef (typeDecl->type->is<types::SetType>()) {
          auto set_t = typeDecl->type->as<types::SetType>();
          for (auto &v: set_t->vals)
            if (v.cons == field_name) {
              exprs::Expr *lit = nullptr;
              if (std::holds_alternative<u64>(v.val)) {
                lit = exprs::Expr::make_IntegerLiteral(ctx, now->parent(), (u64)std::get<u64>(v.val), now->pos());
              } else {
                lit = exprs::Expr::make_IntegerLiteral(ctx, now->parent(), (i64)std::get<i64>(v.val), now->pos());
              }
              lit->targetType() = typeDecl->type;
              return lit;
            }
        }
      }
    }

    if (!ret)
      return errors::IdentifierNotFound(now->pos(), join_identifier_path(nick->unresolved));
    

    if (ret->type() == IdentyEnum::Stmt && static_cast<stmts::Stmt *>(ret)->is<stmts::CodeVar>()) {
      auto stmt = static_cast<stmts::Stmt *>(ret);
      auto cvar = stmt->as<stmts::CodeVar>();

      auto var_expr = exprs::Expr::make_VarExpr(ctx, now->parent(), stmt, now->pos());
      if (cvar->targetType)
        var_expr->targetType() = types::Type::make_Reference(ctx, cvar->targetType);
      else
        var_expr->targetType() = nullptr;

      return var_expr;
    }

    if (ret->type() == IdentyEnum::Decl && static_cast<decls::Decl *>(ret)->is<decls::VarDecl>()) {
      auto vdecl = static_cast<decls::Decl *>(ret);
      auto var = vdecl->as<decls::VarDecl>();

      auto var_expr = exprs::Expr::make_VarExpr(ctx, now->parent(), vdecl, now->pos());
      var_expr->targetType() = types::Type::make_Reference(ctx, var->type);

      return var_expr;
    }

    if (ret->type() == IdentyEnum::Decl && static_cast<decls::Decl *>(ret)->is<decls::FuncDecl>()) {
      auto fdecl = static_cast<decls::Decl *>(ret)->as<decls::FuncDecl>();
      auto func_expr = exprs::Expr::make_ValExpr(ctx, now->parent(), fdecl->funcType, nullptr, now->pos());
      
      return func_expr;
    }

    if (auto expr = SMng.fetch_expr(ret)) 
      return expr;

    return errors::IdentifierNExpr(now->pos(), join_identifier_path(nick->unresolved));
  }

  fun ExprSema::sema_SysIntrinsic(exprs::Expr *&now, const std::string &intrin, const std::vector<types::Type*> &gargs, const std::vector<exprs::Expr*> &args) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<types::Type*> resolved_gargs;

    for (auto t : gargs) {
      if_except(sctx.type->sema_Type(t, now->pos()));
      resolved_gargs.push_back(t);
    }

    if (intrin == "is_size") {
      if (resolved_gargs.size() != 1) return errors::GenericArgumentCountMismatch(now->pos(), "is_size", "1");
      now->vari() = exprs::IntegerLiteral{ (u64)8 };
      now->targetType() = ctx->intU64_t();
      return {};
    }
    
    bool res = false;
    
    if (intrin == "is_int" || intrin == "is_float" || intrin == "is_bool" || intrin == "is_char" || intrin == "is_void" || intrin == "is_ptr" || intrin == "is_signed" || intrin == "is_unsigned") {
      if (resolved_gargs.size() != 1) return errors::GenericArgumentCountMismatch(now->pos(), intrin, "1");
      auto T = resolved_gargs[0];
      
      if (intrin == "is_int") res = T->is<types::PrimitiveType>() && T->as<types::PrimitiveType>()->kind >= types::PrimitiveEnum::I8 && T->as<types::PrimitiveType>()->kind <= types::PrimitiveEnum::U128;
      ef (intrin == "is_float") res = T->isFloat();
      ef (intrin == "is_bool") res = T->is<types::PrimitiveType>() && T->as<types::PrimitiveType>()->kind == types::PrimitiveEnum::Bool;
      ef (intrin == "is_char") res = T->is<types::PrimitiveType>() && T->as<types::PrimitiveType>()->kind == types::PrimitiveEnum::Char;
      ef (intrin == "is_void") res = T->is<types::PrimitiveType>() && T->as<types::PrimitiveType>()->kind == types::PrimitiveEnum::Void;
      ef (intrin == "is_ptr") res = T->is<types::PrimitiveType>() && T->as<types::PrimitiveType>()->kind == types::PrimitiveEnum::Ptr;
      ef (intrin == "is_signed") res = T->isSigned();
      ef (intrin == "is_unsigned") res = T->isUnSigned();
    }
    ef (intrin == "is_pointer" || intrin == "is_reference" || intrin == "is_array" || intrin == "is_struct" || intrin == "is_function" || intrin == "is_enum" || intrin == "is_set") {
      if (resolved_gargs.size() != 1) return errors::GenericArgumentCountMismatch(now->pos(), intrin, "1");
      auto T = resolved_gargs[0];
      
      if (intrin == "is_pointer") res = T->is<types::PointerType>();
      ef (intrin == "is_reference") res = T->is<types::ReferenceType>();
      ef (intrin == "is_array") res = T->is<types::ZArrayType>() || T->is<types::PArrayType>();
      ef (intrin == "is_struct") res = T->is<types::StructType>();
      ef (intrin == "is_function") res = T->is<types::FuncType>();
      ef (intrin == "is_enum") res = T->is<types::EnumType>();
      ef (intrin == "is_set") res = T->is<types::SetType>();
    }
    ef (intrin == "is_same") {
      if (resolved_gargs.size() != 2) return errors::GenericArgumentCountMismatch(now->pos(), intrin, "2");
      res = (resolved_gargs[0]->typname() == resolved_gargs[1]->typname());
    }
    ef (intrin == "is_convertible") {
      if (resolved_gargs.size() != 2) return errors::GenericArgumentCountMismatch(now->pos(), intrin, "2");
      auto dummy = exprs::Expr::make_IntegerLiteral(ctx, now, (i64)0, now->pos()); // Dummy
      dummy->targetType() = resolved_gargs[0];
      res = sctx.expr->sema_Convert(resolved_gargs[1], dummy, now->pos()).has_value();
    }
    ef (intrin == "cast") {
      if (resolved_gargs.size() != 1) return errors::GenericArgumentCountMismatch(now->pos(), "cast", "1");
      if (args.size() != 1) return errors::ArgumentCountMismatch(now->pos(), "cast", "1");
      
      auto target_type = resolved_gargs[0];
      auto val = args[0];
      if_except(sctx.expr->sema_Expr(val));
      
      auto src_type = val->targetType();
      while (src_type->isReference()) src_type = src_type->as<types::ReferenceType>()->sub;
      while (target_type->isReference()) target_type = target_type->as<types::ReferenceType>()->sub;

      if (
        (target_type->is<types::EnumType>() && src_type->isInteger()) ||
        (target_type->isInteger() && src_type->is<types::EnumType>()) ||
        (target_type->is<types::SetType>() && src_type->isInteger()) ||
        (target_type->isInteger() && src_type->is<types::SetType>()) ||
        (target_type->isInteger() && src_type->isInteger()) ||
        (target_type->isFloat() && src_type->isFloat()) ||
        (target_type->isInteger() && src_type->isFloat()) ||
        (target_type->isFloat() && src_type->isInteger()) ||
        (target_type->isInteger() && (src_type->isPointer() || src_type == ctx->ptr_t() || src_type == ctx->null_t())) ||
        ((target_type->isPointer() || target_type == ctx->ptr_t() || target_type == ctx->null_t()) && src_type->isInteger())
      ){
        
        if (target_type->is<types::EnumType>() && val->is<exprs::IntegerLiteral>()) {
          auto enum_t = target_type->as<types::EnumType>();
          auto iv = val->as<exprs::IntegerLiteral>();
          
          i64 target_val = 0;
          if (std::holds_alternative<u64>(iv->val)) target_val = (i64)std::get<u64>(iv->val);
          else target_val = std::get<i64>(iv->val);

          bool found = false;
          for (auto &v : enum_t->vals) {
            i64 enum_v = 0;
            
            if (std::holds_alternative<u64>(v.val))
              enum_v = (i64)std::get<u64>(v.val);
            else
              enum_v = (i64)std::get<i64>(v.val);

            if (target_val == enum_v) { found = true; break; }
          }
          if (!found)
            return errors::CastBoundsError(now->pos(), "enum cast");
        }

        if (target_type->is<types::SetType>() && val->is<exprs::IntegerLiteral>()) {
          auto set_t = target_type->as<types::SetType>();
          auto iv = val->as<exprs::IntegerLiteral>();
          
          i64 target_val = 0;
          if (std::holds_alternative<u64>(iv->val))
            target_val = (i64)std::get<u64>(iv->val);
          else
            target_val = std::get<i64>(iv->val);

          i64 max_v = 0;
          for (auto &v : set_t->vals) {
            i64 set_v = 0;
            if (std::holds_alternative<u64>(v.val))
              set_v = (i64)std::get<u64>(v.val);
            else
              set_v = (i64)std::get<i64>(v.val);
            
            if (set_v > max_v) max_v = set_v;
          }
          
          i64 allowed_max = (max_v << 1) - 1;
          if (target_val < 0 || target_val > allowed_max)
            return errors::CastBoundsError(now->pos(), "set cast");
        }
        
        if (target_type->isInteger() && target_type->is<types::PrimitiveType>() && val->is<exprs::IntegerLiteral>()) {
          auto iv = val->as<exprs::IntegerLiteral>();
          auto pt = target_type->as<types::PrimitiveType>();
          
          u64 max_val = 0;
          switch(pt->kind) {
            case types::PrimitiveEnum::I8:
            case types::PrimitiveEnum::U8: max_val = 0xFF; break;
            case types::PrimitiveEnum::I16:
            case types::PrimitiveEnum::U16: max_val = 0xFFFF; break;
            case types::PrimitiveEnum::I32:
            case types::PrimitiveEnum::U32: max_val = 0xFFFFFFFF; break;
            case types::PrimitiveEnum::I64:
            case types::PrimitiveEnum::U64: max_val = 0xFFFFFFFFFFFFFFFF; break;
            default: max_val = (u64)-1; break;
          }
          if (std::holds_alternative<u64>(iv->val)) {
            if (std::get<u64>(iv->val) > max_val)
              return errors::CastBoundsError(now->pos(), "int cast");
          }
          else {
            if ((u64)std::get<i64>(iv->val) > max_val)
              return errors::CastBoundsError(now->pos(), "int cast");
          }
        }
      
        now = exprs::Expr::make_UnaryOp(ctx, now->parent(), exprs::UnaryOpEnum::Plus, val, now->pos());
        now->targetType() = target_type;
        return {};
      }

      bool is_ptr_target = resolved_gargs[0]->is<types::PointerType>() || resolved_gargs[0] == ctx->ptr_t() || resolved_gargs[0] == ctx->null_t() || resolved_gargs[0]->isReference();
      bool is_ptr_source = val->targetType()->is<types::PointerType>() || val->targetType() == ctx->ptr_t() || val->targetType() == ctx->null_t() || val->targetType()->isReference();

      if (is_ptr_target && is_ptr_source) {
        now = exprs::Expr::make_UnaryOp(ctx, now->parent(), exprs::UnaryOpEnum::Plus, val, now->pos());
        now->targetType() = target_type;
        return {};
      }

      if (
        (resolved_gargs[0]->isReference() || resolved_gargs[0]->is<types::PointerType>()) && 
        (val->targetType()->isReference() || val->targetType()->is<types::PointerType>())
      ){
      
        auto get_sub = [](types::Type *t) -> types::Type* {
          while(t->isReference() || t->is<types::PointerType>()) {
            if (t->isReference()) t = t->as<types::ReferenceType>()->sub;
            ef (t->isPointer()) t = t->as<types::PointerType>()->sub;
          }
          return t;
        };
        auto t_sub = get_sub(resolved_gargs[0]);
        auto s_sub = get_sub(val->targetType());

        if ((t_sub->is<types::StructType>() || t_sub->is<types::IFaceType>()) && s_sub->is<types::StructType>()) {
          std::vector<types::Type*> search_queue;
          search_queue.push_back(s_sub);
          bool found = false;
          for (size_t q = 0; q < search_queue.size(); ++q) {
            auto search_type = search_queue[q];
            if (search_type == t_sub) { found = true; break; }
            if (search_type->is<types::StructType>()) {
              auto search_rec = search_type->as<types::StructType>();
              for (auto bt : search_rec->baseTypes) {
                auto rbt = bt;
                
                while (rbt->isReference()) rbt = rbt->as<types::ReferenceType>()->sub;
                
                if (rbt->is<types::StructType>() || rbt->is<types::IFaceType>()) {
                  search_queue.push_back(rbt);
                }
              }
            }
            ef (search_type->is<types::IFaceType>()) {
              auto search_rec = search_type->as<types::IFaceType>();
              for (auto bt : search_rec->baseTypes) {
                auto rbt = bt;
                
                while(rbt->isReference()) rbt = rbt->as<types::ReferenceType>()->sub;

                if (rbt->is<types::IFaceType>()) {
                  search_queue.push_back(rbt);
                }
              }
            }
          }
          if (found) {
            now = exprs::Expr::make_UnaryOp(ctx, now->parent(), exprs::UnaryOpEnum::Plus, val, now->pos());
            now->targetType() = resolved_gargs[0];
            return {};
          }
        }
      }

      return errors::UnsupportedCast(now->pos());
    }
    else
      return errors::UnsupportedIntrinsic(now->pos(), intrin);
    
    now->vari() = exprs::BoolLiteral{res};
    now->targetType() = ctx->bool_t();
    
    return {};
  }

}
