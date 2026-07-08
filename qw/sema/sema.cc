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
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include <cassert>
#include <expected>
#include <iostream>
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

  // Decl
  fun Sema::sema_Attributes(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    for (auto &attr: now->const_attrs())
      if (attr.name == "symbol") {
        if (attr.value != "bare" && attr.value != "qw")
          return errors::InvalidAttributeValue(now->pos(), attr.name, attr.value);
      }
      ef (attr.name == "rtl") {
        if (!attr.value.empty())
          return errors::InvalidAttributeValue(now->pos(), attr.name, attr.value);
      }
      ef (attr.name == "weak") {
        if (!attr.value.empty())
          return errors::InvalidAttributeValue(now->pos(), attr.name, attr.value);
      }
      ef (attr.name == "calling") {
        if (attr.value != "fast" && attr.value != "cdecl" && attr.value != "cold")
          return errors::InvalidAttributeValue(now->pos(), attr.name, attr.value);
      }
      else
        return errors::InvalidAttribute(now->pos(), attr.name);
    
    return {};
  }

  fun Sema::sema_NameSpace(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    SMng.ans().push_back(scopemng::mangling_abi_qw(now));

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      auto name = scopemng::mangling_abi_qw(X);
      auto existing = ctx->gst().find(name);
      if (!existing.has_value())
        ctx->gst().add_ident(name, X);
    }

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      if_except(sema_Attributes(X));
      if (X->is<decls::NameSpaceDecl>()) if_error(sema_NameSpace(X))
      ef (X->is<decls::FuncDecl>()) if_error(sema_FuncDecl(X))
      ef (X->is<decls::TypeDecl>()) if_error(sema_TypeDecl(X))
      ef (X->is<decls::VarDecl>()) if_error(sema_VarDecl(X))
      ef (X->is<decls::AliasDecl>()) {}
      else
        diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());
    }

    SMng.ans().pop_back();
    return {};
  }



  fun Sema::sema_TypeDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    if_except(sema_Type(now->as<decls::TypeDecl>()->type, now->pos()));

    return {};
  }

  fun Sema::sema_FuncDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto fdecl = now->as<decls::FuncDecl>();
    types::Type *ftype = fdecl->funcType;

    if_except(sema_FuncType(ftype, now->pos()));
    
    bool is_rtl = false;
    for (auto &attr: now->const_attrs()) {
      if (attr.name == "rtl") {
        is_rtl = true;
        break;
      }
    }

    if (is_rtl && now->pos().mod() && !now->pos().mod()->llvm()->getName().starts_with("__qwrtl")) {
      is_rtl = false;
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
        intrinsic_fdecl->body->parent() = intrinsic_decl;
        
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

      if_except(sema_CodeBlock(fdecl->body, ftype_concrete->ret));
    }

    return {};
  }

  fun Sema::sema_ConstructorDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto cdecl = now->as<decls::ConstructorDecl>();
    types::Type *ctype = cdecl->funcType;

    if_except(sema_FuncType(ctype, now->pos()));
    
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

          if_except(sema_Expr(init_pair.second));
          if_except(sema_Convert(target_type, init_pair.second, now->pos()));
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

      if_except(sema_CodeBlock(cdecl->body, ftype_concrete->ret));
    }

    return {};
  }

  fun Sema::sema_DestructorDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto cdecl = now->as<decls::DestructorDecl>();
    types::Type *ctype = cdecl->funcType;

    if_except(sema_FuncType(ctype, now->pos()));
    
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

      if_except(sema_CodeBlock(cdecl->body, ftype_concrete->ret));
    }

    return {};
  }

  fun Sema::sema_VarDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    return sema_Type(now->as<decls::VarDecl>()->type, now->pos());
  }



  // Stat
  fun Sema::sema_Stmt(stmts::Stmt *X, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    if (X->is<stmts::CodeVar>())      return sema_VarStmt(X);
    ef (X->is<stmts::ReturnStmt>())   return sema_ReturnStmt(X, expected_ret);
    ef (X->is<stmts::ExprStmt>())     return sema_Expr(X->as<stmts::ExprStmt>()->expr);
    ef (X->is<stmts::IfStmt>())       return sema_IfStmt(X, expected_ret);
    ef (X->is<stmts::WhileStmt>())    return sema_WhileStmt(X, expected_ret);
    ef (X->is<stmts::CodeBlock>())    return sema_CodeBlock(X, expected_ret);
    ef (X->is<stmts::BreakStmt>())    return sema_BreakStmt(X, expected_ret);
    ef (X->is<stmts::ContinueStmt>()) return sema_ContinueStmt(X, expected_ret);
    ef (X->is<stmts::UnsafeStmt>())   return sema_UnsafeStmt(X, expected_ret);
    else return std::unexpected(fatals::Internal_UnknownStmt().error());
  }

  fun Sema::sema_CodeBlock(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto block = now->as<stmts::CodeBlock>();

    for (auto X: block->vars)
      if_error(sema_VarStmt(X));

    for (auto &X : block->codes) {
      if_error(sema_Stmt(X, expected_ret));
    }

    return {};
  }

  fun Sema::sema_VarStmt(stmts::Stmt *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto cvar = now->as<stmts::CodeVar>();
    if_except(sema_Type(cvar->targetType, now->pos()));

    return {};
  }

  fun Sema::sema_ReturnStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto Ret = now->as<stmts::ReturnStmt>();
    if_except(sema_Expr(Ret->expr));
    if_except(sema_Convert(expected_ret, Ret->expr, now->pos()));

    return {};
  }

  fun Sema::sema_IfStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto if_stmt = now->as<stmts::IfStmt>();
    if_except(sema_Expr(if_stmt->condition));

    auto cond_type = if_stmt->condition->targetType();
    while (cond_type->isReference()) {
      cond_type = cond_type->as<types::ReferenceType>()->sub;
    }

    if (!cond_type->isBool()) return errors::NoMatchOperator(now->pos(), "if", std::string(if_stmt->condition->targetType()->typname()), "condition must be boolean");

    if (if_stmt->then_block)
      if_except(sema_CodeBlock(if_stmt->then_block, expected_ret));
    
    if (if_stmt->else_block) {
      if (if_stmt->else_block->is<stmts::IfStmt>())
        if_except(sema_IfStmt(if_stmt->else_block, expected_ret))
      else
        if_except(sema_CodeBlock(if_stmt->else_block, expected_ret))
    }

    return {};
  }

  fun Sema::sema_WhileStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto while_stmt = now->as<stmts::WhileStmt>();
    if_except(sema_Expr(while_stmt->condition));

    auto cond_type = while_stmt->condition->targetType();
    while (cond_type->isReference()) {
      cond_type = cond_type->as<types::ReferenceType>()->sub;
    }

    if (!cond_type->isBool()) return errors::NoMatchOperator(now->pos(), "while", std::string(while_stmt->condition->targetType()->typname()), "condition must be boolean");

    if (while_stmt->body) {
      loop_stack.push_back(now);
      auto res = sema_CodeBlock(while_stmt->body, expected_ret);
      loop_stack.pop_back();
      if_except(std::move(res));
    }

    return {};
  }

  fun Sema::sema_BreakStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    if (loop_stack.empty()) return errors::NoMatchOperator(now->pos(), "break", "", "break statement not within loop");
  
    now->as<stmts::BreakStmt>()->loop = loop_stack.back();
    return {};
  }

  fun Sema::sema_ContinueStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    if (loop_stack.empty()) return errors::NoMatchOperator(now->pos(), "continue", "", "continue statement not within loop");
    
    auto cnt = now->as<stmts::ContinueStmt>();
    cnt->loop = loop_stack.back();
    return {};
  }

  fun Sema::sema_UnsafeStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>> {
    auto u_stmt = now->as<stmts::UnsafeStmt>();
    unsafe_level++;

    auto X = u_stmt->stmt;
    auto ret = sema_Stmt(X, expected_ret);

    unsafe_level--;
    return ret;
  }



  // Expr
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

  fun Sema::sema_Convert(types::Type *typ, exprs::Expr *val, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
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

    if (unsafe_level > 0) {
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

  fun Sema::sema_Expr(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    begin:

    if (now->is<exprs::IntegerLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->intS32_t();
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

      if (!now->targetType()) {
        auto cvar = C->var->as<stmts::CodeVar>();
        now->targetType() = types::Type::make_Reference(ctx, cvar->targetType);
      }
    }

    ef (now->is<exprs::UnaryOp>())   return sema_Expr_UnaryOp(now);
    ef (now->is<exprs::BinaryOp>())  return sema_Expr_BinaryOp(now);
    ef (now->is<exprs::MemberOp>())  return sema_Expr_MemberOp(now);
    ef (now->is<exprs::PostfixOp>()) return sema_Expr_PostfixOp(now);
    ef (now->is<exprs::ValExpr>())   return {};

    ef (now->is<exprs::NickExpr>()) {
      auto ret = sema_NickExpr(now);

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

  fun Sema::sema_Expr_UnaryOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->targetType()) return {};
    auto U = now->as<exprs::UnaryOp>();
    if_except(sema_Expr(U->o1));

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

  fun Sema::sema_Expr_BinaryOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto C = now->as<exprs::BinaryOp>();

    bool is_compound_assign = (C->kind >= exprs::BinaryOpEnum::AddAssign && C->kind <= exprs::BinaryOpEnum::ShrAssign);

    if (C->kind == exprs::BinaryOpEnum::Assign || is_compound_assign) {
      if_except(sema_Expr(C->o1));
      if_except(sema_Expr(C->o2));

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

      if_except(sema_Convert(lhs_concrete_type, C->o2, now->pos()));

      now->targetType() = lhs_concrete_type;
      return {};
    }

    if_except(sema_Expr(C->o1));
    if_except(sema_Expr(C->o2));

    auto t1 = C->o1->targetType(), t2 = C->o2->targetType();

    while (t1->isReference() || t2->isReference()) {
      if (t1->isReference()) t1 = t1->as<types::ReferenceType>()->sub;
      if (t2->isReference()) t2 = t2->as<types::ReferenceType>()->sub;
    }

    if (t1->isInteger() && t2->isInteger()) {
      types::Type *target{};

      if (t1 != t2)
        target = (t1->intBit() != t2->intBit()) ? (t1->intBit() > t2->intBit() ? t1 : t2) : (!t1->isSigned() ? t1 : t2);
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

  fun Sema::sema_Expr_MemberOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
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
              std::variant<u128, i128> val128;
              if (std::holds_alternative<u64>(v.val)) val128 = (u128)std::get<u64>(v.val);
              else val128 = (i128)std::get<i64>(v.val);
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
              std::variant<u128, i128> val128;
              if (std::holds_alternative<u64>(v.val)) val128 = (u128)std::get<u64>(v.val);
              else val128 = (i128)std::get<i64>(v.val);
              now->vari() = exprs::IntegerLiteral{ val128 };
              now->targetType() = typeDecl->type;
              return {};
            }
          
          return errors::IdentifierNotFound(M->mem->pos(), field_name);
        }
      }
    }

    if_except(sema_Expr(M->obj));

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

  fun Sema::sema_Expr_PostfixOp(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto M = now->as<exprs::PostfixOp>();

    if (M->kind == exprs::PostfixOpEnum::Deref) {
      if_except(sema_Expr(M->obj));
      auto t = M->obj->targetType();
      if (t->isReference()) t = t->as<types::ReferenceType>()->sub;

      if (!t->is<types::PointerType>()) return errors::NoMatchOperator(now->pos(), "?", std::string(t->typname()), "operand must be a pointer");
    
      auto base_type = t->as<types::PointerType>()->sub;
      now->targetType() = types::Type::make_Reference(ctx, base_type);
      return {};
    }
    ef (M->kind == exprs::PostfixOpEnum::Call)  return sema_Expr_PostfixOp_Call(now);
    ef (M->kind == exprs::PostfixOpEnum::Array) return sema_Expr_PostfixOp_Array(now);
    else
      diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
  }

  fun Sema::sema_Expr_PostfixOp_Call(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
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
      return sema_SysIntrinsic(now, sys_name, sys_gargs, M->operands);

    if (M->obj->is<exprs::MemberOp>()) {
      auto memOp = M->obj->as<exprs::MemberOp>();
      if_except(sema_Expr(memOp->obj));

      auto obj_type = memOp->obj->targetType();
      while (obj_type->isReference()) obj_type = obj_type->as<types::ReferenceType>()->sub;

      if (obj_type->is<types::StructType>()) {
        auto rec_type    = obj_type->as<types::StructType>();
        auto member_name = memOp->mem->as<exprs::NickExpr>()->unresolved[0];

        for (size_t i = 0; i < M->operands.size(); i++) if_except(sema_Expr(M->operands[i]));

        std::vector<types::Type *> arg_types;
        arg_types.push_back(memOp->obj->targetType());
        for (auto op : M->operands) arg_types.push_back(op->targetType());

        decls::Decl *found_func = sema_Resolve_StructMethod(rec_type, member_name, arg_types);

        if (found_func) {
          auto fdecl = found_func->as<decls::FuncDecl>();
          auto ftype = fdecl->funcType->as<types::FuncType>();
          M->operands.insert(M->operands.begin(), memOp->obj);

          for (size_t i = 0; i < M->operands.size(); ++i) {
            if_except(sema_Expr(M->operands[i]));
            if_except(sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
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

        for (size_t i = 0; i < M->operands.size(); i++) if_except(sema_Expr(M->operands[i]));

        std::vector<types::Type *> arg_types;
        arg_types.push_back(memOp->obj->targetType());
        for (auto op : M->operands) arg_types.push_back(op->targetType());

        decls::Decl *found_func = sema_Resolve_IFaceMethod(iface_type, member_name, arg_types);

        if (found_func) {
          auto fdecl = found_func->as<decls::FuncDecl>();
          auto ftype = fdecl->funcType->as<types::FuncType>();
          M->operands.insert(M->operands.begin(), memOp->obj);

          for (size_t i = 0; i < M->operands.size(); ++i) {
            if_except(sema_Expr(M->operands[i]));
            if_except(sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
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
        if_except(sema_Expr(M->operands[i]));
      }

      std::vector<types::Type *> arg_types;
      for (auto op : M->operands)
        arg_types.push_back(op->targetType());

      auto ret = SMng.lookup(now->parent(), nick->unresolved, &arg_types);
      if (ret && ret->type() == IdentyEnum::Decl && static_cast<decls::Decl *>(ret)->is<decls::FuncDecl>()) {
        auto fdecl = static_cast<decls::Decl *>(ret)->as<decls::FuncDecl>();
        auto ftype = fdecl->funcType->as<types::FuncType>();

        for (size_t i = 0; i < M->operands.size(); ++i) {
          if_except(sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
        }

        now->targetType()    = ftype->ret;
        M->obj->targetType() = fdecl->funcType;
        return {};
      }
    }

    if_except(sema_Expr(M->obj));
    auto callee_type = M->obj->targetType();

    if (callee_type->is<types::FuncType>()) {
      auto ftype = callee_type->as<types::FuncType>();
      if (M->operands.size() != ftype->pars.size()) return errors::NoMatchOperator(now->pos(), "call", "function pointer", "arguments count mismatch");
      
      for (size_t i = 0; i < M->operands.size(); ++i) {
        if_except(sema_Expr(M->operands[i]));
        if_except(sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
      }
      now->targetType() = ftype->ret;
      return {};
    }

    return errors::NoMatchOperator(now->pos(), "call", std::string(M->obj->targetType()->typname()), "not a callable type");
  }

  fun Sema::sema_Expr_PostfixOp_Array(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>> {
    auto M = now->as<exprs::PostfixOp>();
    if_except(sema_Expr(M->obj));

    auto obj_type = M->obj->targetType();
    while (obj_type->isReference()) {
      obj_type = obj_type->as<types::ReferenceType>()->sub;
    }

    if (M->operands.size() != 1) return errors::NoMatchOperator(now->pos(), "[]", std::string(M->obj->targetType()->typname()), "dimension mismatch");

    if_except(sema_Expr(M->operands[0]));
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


  fun Sema::sema_Resolve_StructMethod(types::StructType *rec_type, const std::string &member_name, const std::vector<types::Type*> &arg_types) -> decls::Decl* {
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

  fun Sema::sema_Resolve_IFaceMethod(types::IFaceType *iface_type, const std::string &member_name, const std::vector<types::Type*> &arg_types) -> decls::Decl* {
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


  fun Sema::sema_NickExpr(exprs::Expr *now) -> std::expected<exprs::Expr*, uptr<diagnostic::message>> {
    auto nick = now->as<exprs::NickExpr>();
    
    const auto join_identifier_path = [](const std::vector<std::string> &ps) -> std::string {
      std::string ret = ps[0];
      for (size_t i = 1; i < ps.size(); i++)
        ret += "::" + ps[i];
      return ret;
    };

    auto ret = SMng.lookup(now->parent(), nick->unresolved);

    if (!ret)
      return errors::IdentifierNotFound(now->pos(), join_identifier_path(nick->unresolved));
    

    if (ret->type() == IdentyEnum::Stmt && static_cast<stmts::Stmt *>(ret)->is<stmts::CodeVar>()) {
      auto stmt = static_cast<stmts::Stmt *>(ret);
      auto cvar = stmt->as<stmts::CodeVar>();

      auto var_expr = exprs::Expr::make_VarExpr(ctx, now->parent(), stmt, now->pos());
      var_expr->targetType() = types::Type::make_Reference(ctx, cvar->targetType);

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

  fun Sema::sema_SysIntrinsic(exprs::Expr *&now, const std::string &intrin, const std::vector<types::Type*> &gargs, const std::vector<exprs::Expr*> &args) -> std::expected<void, uptr<diagnostic::message>> {
    std::vector<types::Type*> resolved_gargs;

    for (auto t : gargs) {
      if_except(sema_Type(t, now->pos()));
      resolved_gargs.push_back(t);
    }

    if (intrin == "is_size") {
      if (resolved_gargs.size() != 1) return errors::GenericArgumentCountMismatch(now->pos(), "is_size", "1");
      now->vari() = exprs::IntegerLiteral{ (u128)8 };
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
      auto dummy = exprs::Expr::make_IntegerLiteral(ctx, now, (i128)0, now->pos()); // Dummy
      dummy->targetType() = resolved_gargs[0];
      res = sema_Convert(resolved_gargs[1], dummy, now->pos()).has_value();
    }
    ef (intrin == "cast") {
      if (resolved_gargs.size() != 1) return errors::GenericArgumentCountMismatch(now->pos(), "cast", "1");
      if (args.size() != 1) return errors::ArgumentCountMismatch(now->pos(), "cast", "1");
      
      auto target_type = resolved_gargs[0];
      auto val = args[0];
      if_except(sema_Expr(val));
      
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
          
          i128 target_val = 0;
          if (std::holds_alternative<u128>(iv->val)) target_val = (i128)std::get<u128>(iv->val);
          else target_val = std::get<i128>(iv->val);

          bool found = false;
          for (auto &v : enum_t->vals) {
            i128 enum_v = 0;
            
            if (std::holds_alternative<u64>(v.val))
              enum_v = (i128)std::get<u64>(v.val);
            else
              enum_v = (i128)std::get<i64>(v.val);

            if (target_val == enum_v) { found = true; break; }
          }
          if (!found)
            return errors::CastBoundsError(now->pos(), "enum cast");
        }

        if (target_type->is<types::SetType>() && val->is<exprs::IntegerLiteral>()) {
          auto set_t = target_type->as<types::SetType>();
          auto iv = val->as<exprs::IntegerLiteral>();
          
          i128 target_val = 0;
          if (std::holds_alternative<u128>(iv->val))
            target_val = (i128)std::get<u128>(iv->val);
          else
            target_val = std::get<i128>(iv->val);

          i128 max_v = 0;
          for (auto &v : set_t->vals) {
            i128 set_v = 0;
            if (std::holds_alternative<u64>(v.val))
              set_v = (i128)std::get<u64>(v.val);
            else
              set_v = (i128)std::get<i64>(v.val);
            
            if (set_v > max_v) max_v = set_v;
          }
          
          i128 allowed_max = (max_v << 1) - 1;
          if (target_val < 0 || target_val > allowed_max)
            return errors::CastBoundsError(now->pos(), "set cast");
        }
        
        if (target_type->isInteger() && target_type->is<types::PrimitiveType>() && val->is<exprs::IntegerLiteral>()) {
          auto iv = val->as<exprs::IntegerLiteral>();
          auto pt = target_type->as<types::PrimitiveType>();
          
          u128 max_val = 0;
          switch(pt->kind) {
            case types::PrimitiveEnum::I8:
            case types::PrimitiveEnum::U8: max_val = 0xFF; break;
            case types::PrimitiveEnum::I16:
            case types::PrimitiveEnum::U16: max_val = 0xFFFF; break;
            case types::PrimitiveEnum::I32:
            case types::PrimitiveEnum::U32: max_val = 0xFFFFFFFF; break;
            case types::PrimitiveEnum::I64:
            case types::PrimitiveEnum::U64: max_val = 0xFFFFFFFFFFFFFFFF; break;
            default: max_val = (u128)-1; break;
          }
          if (std::holds_alternative<u128>(iv->val)) {
            if (std::get<u128>(iv->val) > max_val)
              return errors::CastBoundsError(now->pos(), "int cast");
          }
          else {
            if ((u128)std::get<i128>(iv->val) > max_val)
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


  // Type
  fun Sema::sema_Type(types::Type *&now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    assert(now && "null parameter.");

    l_begin:
    if (now->is<types::PrimitiveType>())
      return {};
    ef (now->is<types::TypeParamType>())
      return {};

    ef (now->is<types::StructType>()) return sema_StructType(now, errpos);
    ef (now->is<types::IFaceType>())  return sema_IFaceType(now, errpos);
    ef (now->is<types::EnumType>())   return sema_EnumType(now, errpos);
    ef (now->is<types::SetType>())    return sema_SetType(now, errpos);

    ef (now->is<types::FuncType>()) {
      auto ftype = now->as<types::FuncType>();
      std::vector<types::FieldType> resolved_pars = ftype->pars;
      
      for (auto &X : resolved_pars)
        if_except(sema_Type(X.type, errpos));

      auto resolved_ret = ftype->ret;
      if_except(sema_Type(resolved_ret, errpos));

      now = types::Type::make_Func(ctx, resolved_pars, resolved_ret);
    }
    ef(now->is<types::PointerType>()) {
      auto sub = now->as<types::PointerType>()->sub;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_Pointer(ctx, sub);
    }
    ef(now->is<types::ReferenceType>()) {
      auto sub = now->as<types::ReferenceType>()->sub;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_Reference(ctx, sub);
    }
    ef(now->is<types::ZArrayType>()) {
      auto sub = now->as<types::ZArrayType>()->sub;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_ZArray(ctx, sub);
    }
    ef(now->is<types::PArrayType>()) {
      auto parray = now->as<types::PArrayType>();
      auto sub = parray->sub;
      u64 size = parray->size;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_PArray(ctx, sub, size);
    }

    ef(now->is<types::GenericType>()) {
      auto gen = now->as<types::GenericType>();
      if_except(sema_Type(gen->sub, errpos));
      for (auto &f : gen->fields) {
        if_except(sema_Type(f, errpos));
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
        
        if_except(sema_Type(inst_type, errpos));
        now = inst_type;
      }
    }
    ef(now->is<types::NickType>()) {
      auto ret = sema_NickType(now, errpos);

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

  fun Sema::sema_EnumType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    now->sema() = StageStatus::Checking;
    
    auto enumType = now->as<types::EnumType>();
    
    now->sema() = StageStatus::Checked;
    
    if (enumType->decl) {
      for (auto &F: enumType->decl->as<decls::EnumDecl>()->func) {
        if_except(sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }

    i128 min_val = 0, max_val = 0;
    if (!enumType->vals.empty()) {
      min_val = std::holds_alternative<u64>(enumType->vals[0].val) ? (i128)std::get<u64>(enumType->vals[0].val) : (i128)std::get<i64>(enumType->vals[0].val);
      max_val = min_val;
    }

    for (auto &v: enumType->vals) {
      i128 cv = std::holds_alternative<u64>(v.val) ? (i128)std::get<u64>(v.val) : (i128)std::get<i64>(v.val);
      if (cv < min_val) min_val = cv;
      if (cv > max_val) max_val = cv;
    }

    if (enumType->baseType) {
      word base_pos = enumType->baseTypePos ? enumType->baseTypePos : errpos;
      if_except(sema_Type(enumType->baseType, base_pos));
      if (!enumType->baseType->isInteger()) {
        return errors::InvalidBaseType(base_pos, std::string(now->typname()), "must be an integer");
      }
      
      bool is_signed = enumType->baseType->isSigned();
      u32 bits = enumType->baseType->intBit();
      
      i128 b_min = is_signed ? -((i128)1 << (bits - 1)) : 0;
      i128 b_max = is_signed ? ((i128)1 << (bits - 1)) - 1 : (bits == 128 ? (i128)-1 : ((i128)1 << bits) - 1);
      
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

  fun Sema::sema_SetType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    now->sema() = StageStatus::Checking;
    
    auto setType = now->as<types::SetType>();
    
    now->sema() = StageStatus::Checked;
    
    if (setType->decl) {
      for (auto &F: setType->decl->as<decls::SetDecl>()->func) {
        if_except(sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }

    i128 max_val = 0;
    if (!setType->vals.empty()) {
      max_val = std::holds_alternative<u64>(setType->vals[0].val) ? (i128)std::get<u64>(setType->vals[0].val) : (i128)std::get<i64>(setType->vals[0].val);
    }

    for (auto &v: setType->vals) {
      i128 cv = std::holds_alternative<u64>(v.val) ? (i128)std::get<u64>(v.val) : (i128)std::get<i64>(v.val);
      if (cv > max_val) max_val = cv;
    }

    if (setType->baseType) {
      word base_pos = setType->baseTypePos ? setType->baseTypePos : errpos;
      if_except(sema_Type(setType->baseType, base_pos));
      if (!setType->baseType->isInteger()) {
        return errors::InvalidBaseType(base_pos, std::string(now->typname()), "must be an integer");
      }
      
      bool is_signed = setType->baseType->isSigned();
      u32 bits = setType->baseType->intBit();
      
      i128 b_max = is_signed ? ((i128)1 << (bits - 1)) - 1 : (bits == 128 ? (i128)-1 : ((i128)1 << bits) - 1);
      
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

  fun Sema::sema_StructType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    ef (now->sema() == StageStatus::Checking) return {};

    now->sema() = StageStatus::Checking;

    auto strct = now->as<types::StructType>();

    std::vector<types::FieldType> flattened_vars;
    for (size_t i = 0; i < strct->baseTypes.size(); ++i) {
      auto &baseType = strct->baseTypes[i];
      word base_pos = strct->baseTypePos.size() > i ? strct->baseTypePos[i] : errpos;
      if_except(sema_Type(baseType, base_pos));
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
      if_except(sema_Type(X.type, errpos));
      flattened_vars.push_back(X);
    }

    strct->vars = flattened_vars;

    now->sema() = StageStatus::Checked;

    if (strct->decl) {
      for (auto &F: strct->decl->as<decls::StructDecl>()->func) {
        if_except(sema_Attributes(F));
        if_except(sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
      for (auto &C: strct->decl->as<decls::StructDecl>()->constructors) {
        if_except(sema_Attributes(C));
        if_except(sema_ConstructorDecl(C));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(C), C);
      }
      for (auto &D: strct->decl->as<decls::StructDecl>()->destructors) {
        if_except(sema_Attributes(D));
        if_except(sema_DestructorDecl(D));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(D), D);
      }
    }

    return {};
  }

  fun Sema::sema_IFaceType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->sema() == StageStatus::Checked) return {};
    ef (now->sema() == StageStatus::Checking) return {};
    
    now->sema() = StageStatus::Checking;
    auto iface = now->as<types::IFaceType>();

    for (size_t i = 0; i < iface->baseTypes.size(); ++i) {
      auto &baseType = iface->baseTypes[i];
      word base_pos = iface->baseTypePos.size() > i ? iface->baseTypePos[i] : errpos;
      if_except(sema_Type(baseType, base_pos));
      auto resolved_base = baseType;

      while (resolved_base->isReference())
        resolved_base = resolved_base->as<types::ReferenceType>()->sub;

      if (!resolved_base->is<types::IFaceType>())
        return errors::InvalidBaseType(base_pos, std::string(now->typname()), "must be an interface");
    }

    if (iface->decl) {
      for (auto &F: iface->decl->as<decls::IFaceDecl>()->func) {
        if_except(sema_Attributes(F));
        if_except(sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }

    now->sema() = StageStatus::Checked;
    return {};
  }

  fun Sema::sema_NickType(types::Type *now, word errpos) -> std::expected<types::Type*, uptr<diagnostic::message>> {
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

  fun Sema::sema_FuncType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>> {
    auto ftype = now->as<types::FuncType>();

    for (auto &X: ftype->pars)
      if_except(sema_Type(X.type, errpos));

    if_except(sema_Type(ftype->ret, errpos));
    return {};
  }

}
