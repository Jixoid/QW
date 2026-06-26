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
  fun Sema::sema_NameSpace(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    SMng.ans().push_back(scopemng::mangling_abi_qw(now));

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      ctx->gst().add_ident(scopemng::mangling_abi_qw(X), X);
    }

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      if (X->is<decls::NameSpaceDecl>())
        if_error(sema_NameSpace(X)) else if (X->is<decls::FuncDecl>()) if_error(sema_FuncDecl(X)) else if (X->is<decls::TypeDecl>())
          if_error(sema_TypeDecl(X)) else diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());
    }

    SMng.ans().pop_back();
    return {};
  }

  fun Sema::sema_TypeDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    if_except(sema_Type(now->as<decls::TypeDecl>()->type, now->pos()));

    return {};
  }

  fun Sema::sema_FuncDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto fdecl         = now->as<decls::FuncDecl>();
    types::Type *ftype = fdecl->funcType;

    if_except(sema_FuncType(ftype, now->pos()));

    if (fdecl->body) {
      auto block          = fdecl->body->as<stmts::CodeBlock>();
      auto ftype_concrete = ftype->as<types::FuncType>();

      for (const auto &p: ftype_concrete->pars) {
        bool exists = false;
        for (auto v : block->vars) {
          if (v->as<stmts::CodeVar>()->name == p.name) {
            exists = true;
            break;
          }
        }
        if (!exists) {
          stmts::Stmt::make_CodeVar(ctx, fdecl->body, p.name, p.type, fdecl->body->pos());
        }
      }

      if_except(sema_CodeBlock(fdecl->body, ftype_concrete->ret));
    }

    return {};
  }

  fun Sema::sema_ConstructorDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto cdecl         = now->as<decls::ConstructorDecl>();
    types::Type *ctype = cdecl->funcType;

    if_except(sema_FuncType(ctype, now->pos()));

    auto ftype_concrete = ctype->as<types::FuncType>();
    
    // Check inits
    if (ftype_concrete->pars.size() > 0) {
      auto self_ref = ftype_concrete->pars[0].type->as<types::ReferenceType>();
      if (self_ref && self_ref->sub->is<types::RecordType>()) {
        auto recType = self_ref->sub->as<types::RecordType>();

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
            return errors::UnexpectedIdentifier(now->pos(), init_pair.first);
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

  fun Sema::sema_VarDecl(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    return sema_Type(now->as<decls::VarDecl>()->type, now->pos());
  }

  // Stat
  fun Sema::sema_CodeBlock(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto block = now->as<stmts::CodeBlock>();

    for (auto X: block->vars)
      if_error(sema_VarStmt(X));

    for (auto &X: block->codes) {
      if (X->is<stmts::ReturnStmt>()) {
        if_error(sema_ReturnStmt(X, expected_ret));
      }
      ef (X->is<stmts::ExprStmt>())
      {
        auto expr_stmt = X->as<stmts::ExprStmt>();
        if_error(sema_Expr(expr_stmt->expr));
      }
      ef (X->is<stmts::IfStmt>()) {
        if_error(sema_IfStmt(X, expected_ret));
      }
      ef (X->is<stmts::WhileStmt>()) {
        if_error(sema_WhileStmt(X, expected_ret));
      }
      ef (X->is<stmts::BreakStmt>()) {
      }
      ef (X->is<stmts::ContinueStmt>()) {
      }
      ef (X->is<stmts::CodeBlock>()) {
        if_error(sema_CodeBlock(X, expected_ret));
      }
      else
      {
        diagnostic::fatal(fatals::Internal_UnknownStmt().error()->msg());
      }
    }

    return {};
  }

  fun Sema::sema_VarStmt(stmts::Stmt *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto cvar = now->as<stmts::CodeVar>();
    if_except(sema_Type(cvar->targetType, now->pos()));
    return {};
  }

  fun Sema::sema_ReturnStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto Ret = now->as<stmts::ReturnStmt>();
    if_except(sema_Expr(Ret->expr));

    if_except(sema_Convert(expected_ret, Ret->expr, now->pos()));

    return {};
  }

  fun Sema::sema_IfStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto if_stmt = now->as<stmts::IfStmt>();
    if_except(sema_Expr(if_stmt->condition));

    auto cond_type = if_stmt->condition->targetType();
    while (cond_type->isReference()) {
      cond_type = cond_type->as<types::ReferenceType>()->sub;
    }

    if (!cond_type->isBool()) {
      return errors::NoMatchOperator(now->pos(), "if", std::string(if_stmt->condition->targetType()->typname()), "condition must be boolean");
    }

    if (if_stmt->then_block) {
      if_except(sema_CodeBlock(if_stmt->then_block, expected_ret));
    }
    
    if (if_stmt->else_block) {
      if (if_stmt->else_block->is<stmts::IfStmt>()) {
        if_except(sema_IfStmt(if_stmt->else_block, expected_ret));
      }
      else {
        if_except(sema_CodeBlock(if_stmt->else_block, expected_ret));
      }
    }

    return {};
  }

  fun Sema::sema_WhileStmt(stmts::Stmt *now, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto while_stmt = now->as<stmts::WhileStmt>();
    if_except(sema_Expr(while_stmt->condition));

    auto cond_type = while_stmt->condition->targetType();
    while (cond_type->isReference()) {
      cond_type = cond_type->as<types::ReferenceType>()->sub;
    }

    if (!cond_type->isBool()) {
      return errors::NoMatchOperator(now->pos(), "while", std::string(while_stmt->condition->targetType()->typname()), "condition must be boolean");
    }

    if (while_stmt->body) {
      if_except(sema_CodeBlock(while_stmt->body, expected_ret));
    }

    return {};
  }

  // Expr
  fun Sema::sema_Convert(types::Type *typ, exprs::Expr *val, word errpos) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto styp = val->targetType();

    l_re:
    if (typ == styp)
      return {};

    ef (styp->isReference()) {
      styp = styp->as<types::ReferenceType>()->sub;
      goto l_re;
    }

    ef (typ->isInteger() && styp->isInteger()) return {};
    ef (typ->isFloat() && styp->isFloat()) return {};

    ef (typ->isChar() && styp->isInteger()) return {};
    ef (typ->isInteger() && styp->isChar()) return {};

    return errors::NoMatchOperator(errpos, "=", std::string(typ->typname()), std::string(val->targetType()->typname()));
  }

  fun Sema::sema_Expr(exprs::Expr *&now) -> std::expected<void, uptr<diagnostic::message>>
  {
    l_begin:

    if (now->is<exprs::IntegerLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->intS32_t();
      return {};
    }
    ef (now->is<exprs::FloatingLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->flo32_t();
      return {};
    }
    ef (now->is<exprs::CharLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->char_t();
      return {};
    }
    ef (now->is<exprs::BoolLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->bool_t();
      return {};
    }
    ef (now->is<exprs::PtrLiteral>()) {
      if (!now->targetType())
        now->targetType() = ctx->ptr_t();
      return {};
    }
    ef (now->is<exprs::StringLiteral>()) {
      if (!now->targetType())
        now->targetType() = types::Type::make_Pointer(ctx, ctx->char_t());
      return {};
    }
    ef (now->is<exprs::VarExpr>()) {
      auto C = now->as<exprs::VarExpr>();

      if (!now->targetType()) {
        auto cvar         = C->var->as<stmts::CodeVar>();
        now->targetType() = types::Type::make_Reference(ctx, cvar->targetType);
      }

      return {};
    }
    ef (now->is<exprs::UnaryOp>()) {
      auto U = now->as<exprs::UnaryOp>();
      if_except(sema_Expr(U->o1));

      if (U->kind == exprs::UnaryOpEnum::AddrOf) {
        if (!U->o1->targetType()->isReference()) {
          return errors::NoMatchOperator(now->pos(), "@", std::string(U->o1->targetType()->typname()), "operand must be an lvalue");
        }
        auto base_type = U->o1->targetType()->as<types::ReferenceType>()->sub;
        now->targetType() = types::Type::make_Pointer(ctx, base_type);
      }
      ef (U->kind == exprs::UnaryOpEnum::Minus || U->kind == exprs::UnaryOpEnum::Plus) {
        auto t = U->o1->targetType();
        if (t->isReference()) t = t->as<types::ReferenceType>()->sub;
        if (!t->isInteger() && !t->isFloat()) {
          return errors::NoMatchOperator(now->pos(), U->kind == exprs::UnaryOpEnum::Minus ? "-" : "+", std::string(t->typname()), "operand must be numeric");
        }
        now->targetType() = t;
      }
      ef (U->kind == exprs::UnaryOpEnum::LNot) {
        auto t = U->o1->targetType();
        if (t->isReference()) t = t->as<types::ReferenceType>()->sub;
        if (t->typname() != ctx->bool_t()->typname()) {
          return errors::NoMatchOperator(now->pos(), "!", std::string(t->typname()), "operand must be boolean");
        }
        now->targetType() = t;
      }
      ef (U->kind == exprs::UnaryOpEnum::BitNot) {
        auto t = U->o1->targetType();
        if (t->isReference()) t = t->as<types::ReferenceType>()->sub;
        if (!t->isInteger() && !t->is<types::SetType>()) {
          return errors::NoMatchOperator(now->pos(), "~", std::string(t->typname()), "operand must be an integer or a set");
        }
        now->targetType() = t;
      }

      return {};
    }
    ef (now->is<exprs::BinaryOp>()) {
      auto C = now->as<exprs::BinaryOp>();

      bool is_compound_assign = (C->kind >= exprs::BinaryOpEnum::AddAssign && C->kind <= exprs::BinaryOpEnum::ShrAssign);

      if (C->kind == exprs::BinaryOpEnum::Assign || is_compound_assign) {
        if_except(sema_Expr(C->o1));
        if_except(sema_Expr(C->o2));

        if (!C->o1->targetType()->isReference()) {
          return errors::NoMatchOperator(
            now->pos(), is_compound_assign ? "compound assignment" : "=", std::string(C->o1->targetType()->typname()), "left-hand side of assignment must be an lvalue"
          );
        }

        auto lhs_concrete_type = C->o1->targetType()->as<types::ReferenceType>()->sub;

        if (is_compound_assign) {
          if (!lhs_concrete_type->isInteger() && !lhs_concrete_type->isFloat()) {
            return errors::NoMatchOperator(now->pos(), "compound assignment", std::string(lhs_concrete_type->typname()), "left-hand side must be numeric");
          }
          if (lhs_concrete_type->isFloat() && (C->kind >= exprs::BinaryOpEnum::RemAssign)) {
            return errors::NoMatchOperator(now->pos(), "compound assignment", std::string(lhs_concrete_type->typname()), "bitwise or remainder operator not allowed for float");
          }
          C->computationType = lhs_concrete_type;
        }

        if_except(sema_Convert(lhs_concrete_type, C->o2, now->pos()));

        now->targetType() = lhs_concrete_type;
        return {};
      }

      if_except(sema_Expr(C->o1));
      if_except(sema_Expr(C->o2));

      auto t1 = C->o1->targetType(), t2 = C->o2->targetType();

      l_re:
      if (t1->isInteger() && t2->isInteger()) {
        types::Type *target{};

        if (t1 != t2)
          target = (t1->intBit() != t2->intBit()) ? (t1->intBit() > t2->intBit() ? t1 : t2) : (!t1->isSigned() ? t1 : t2);
        else
          target = t1;

        if (C->kind == exprs::BinaryOpEnum::Eq || C->kind == exprs::BinaryOpEnum::NEq ||
            C->kind == exprs::BinaryOpEnum::Lt || C->kind == exprs::BinaryOpEnum::Gt ||
            C->kind == exprs::BinaryOpEnum::LEq || C->kind == exprs::BinaryOpEnum::GEq) {
          C->computationType = target;
          now->targetType() = ctx->bool_t();
        } else {
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
            C->kind == exprs::BinaryOpEnum::LEq || C->kind == exprs::BinaryOpEnum::GEq) {
          C->computationType = target;
          now->targetType() = ctx->bool_t();
        } else {
          now->targetType() = target;
        }
        return {};
      }

      if (t1->isReference() || t2->isReference()) {
        if (t1->isReference())
          t1 = t1->as<types::ReferenceType>()->sub;
        if (t2->isReference())
          t2 = t2->as<types::ReferenceType>()->sub;
        goto l_re;
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
          return {};
        }
        if (C->kind == exprs::BinaryOpEnum::BitOr || C->kind == exprs::BinaryOpEnum::BitAnd || C->kind == exprs::BinaryOpEnum::BitXor ||
            C->kind == exprs::BinaryOpEnum::BitOrAssign || C->kind == exprs::BinaryOpEnum::BitAndAssign || C->kind == exprs::BinaryOpEnum::BitXorAssign) {
          C->computationType = t1;
          now->targetType() = t1;
          return {};
        }
      }

      std::unordered_map<exprs::BinaryOpEnum, std::string> Ops = {
        { exprs::BinaryOpEnum::Add, "+" }, { exprs::BinaryOpEnum::Sub, "-" }, { exprs::BinaryOpEnum::Mul, "*" },
        { exprs::BinaryOpEnum::Div, "/" }, { exprs::BinaryOpEnum::Rem, "%" },
        { exprs::BinaryOpEnum::Eq, "==" }, { exprs::BinaryOpEnum::NEq, "!=" },
        { exprs::BinaryOpEnum::Lt, "<" }, { exprs::BinaryOpEnum::Gt, ">" },
        { exprs::BinaryOpEnum::LEq, "<=" }, { exprs::BinaryOpEnum::GEq, ">=" },
      };

      return errors::NoMatchOperator(
        now->pos(), Ops[C->kind], std::string(C->o1->targetType()->typname()), std::string(C->o2->targetType()->typname())
      );
    }
    ef (now->is<exprs::MemberOp>()) {
      auto M = now->as<exprs::MemberOp>();

      if (M->kind == exprs::MemberOpEnum::NameS && M->obj->is<exprs::NickExpr>()) {
         auto nick = M->obj->as<exprs::NickExpr>();
         auto ret = SMng.lookup(now->parent(), nick->unresolved);
         if (ret && ret->type() == IdentyEnum::Decl && static_cast<decls::Decl*>(ret)->is<decls::TypeDecl>()) {
            auto typeDecl = static_cast<decls::Decl*>(ret)->as<decls::TypeDecl>();
            if (typeDecl->type->is<types::EnumType>()) {
               auto enum_t = typeDecl->type->as<types::EnumType>();
               auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];
               for (auto &v : enum_t->vals) {
                  if (v.cons == field_name) {
                     now->vari() = exprs::IntegerLiteral{ v.val };
                     now->targetType() = typeDecl->type;
                     return {};
                  }
               }
               return errors::IdentifierNotFound(M->mem->pos(), field_name);
            } else if (typeDecl->type->is<types::SetType>()) {
               auto set_t = typeDecl->type->as<types::SetType>();
               auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];
               for (auto &v : set_t->vals) {
                  if (v.cons == field_name) {
                     now->vari() = exprs::IntegerLiteral{ v.val };
                     now->targetType() = typeDecl->type;
                     return {};
                  }
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

      if (!obj_type->is<types::RecordType>()) {
        return errors::NoMatchOperator(now->pos(), M->kind == exprs::MemberOpEnum::NameS ? "::" : ".", std::string(M->obj->targetType()->typname()), M->mem->as<exprs::NickExpr>()->unresolved[0]);
      }

      auto rec_type   = obj_type->as<types::RecordType>();
      auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];

      bool found{};
      for (size_t i = 0; i < rec_type->vars.size(); i++) {
        if (rec_type->vars[i].name == field_name) {
          now->targetType() = types::Type::make_Reference(ctx, rec_type->vars[i].type);
          found             = true;
          break;
        }
      }

      if (!found) {
        return errors::IdentifierNotFound(M->mem->pos(), field_name);
      }
      return {};
    }
    ef (now->is<exprs::PostfixOp>()) {
      auto M = now->as<exprs::PostfixOp>();
      if (M->kind == exprs::PostfixOpEnum::Deref) {
        if_except(sema_Expr(M->obj));
        auto t = M->obj->targetType();
        if (t->isReference()) t = t->as<types::ReferenceType>()->sub;

        if (!t->is<types::PointerType>()) {
          return errors::NoMatchOperator(now->pos(), "?", std::string(t->typname()), "operand must be a pointer");
        }

        auto base_type = t->as<types::PointerType>()->sub;
        now->targetType() = types::Type::make_Reference(ctx, base_type);
        return {};
      }
      ef (M->kind == exprs::PostfixOpEnum::Call) {

        bool is_sys = false;
        std::string sys_name;
        std::vector<types::Type*> sys_gargs;
        auto check_sys_nick = [&](exprs::Expr *E) {
          if (E->is<exprs::NickExpr>()) {
            auto nick = E->as<exprs::NickExpr>();
            if (nick->unresolved.size() == 2 && nick->unresolved[0] == "sys") {
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
        } else {
          check_sys_nick(M->obj);
        }
        
        if (is_sys) {
           return sema_SysIntrinsic(now, sys_name, sys_gargs, M->operands);
        }


        if (M->obj->is<exprs::MemberOp>()) {
          auto memOp = M->obj->as<exprs::MemberOp>();
          if_except(sema_Expr(memOp->obj));

          auto obj_type = memOp->obj->targetType();
          while (obj_type->isReference()) {
            obj_type = obj_type->as<types::ReferenceType>()->sub;
          }

          if (obj_type->is<types::RecordType>()) {
            auto rec_type    = obj_type->as<types::RecordType>();
            auto member_name = memOp->mem->as<exprs::NickExpr>()->unresolved[0];

            for (size_t i = 0; i < M->operands.size(); i++) {
              if_except(sema_Expr(M->operands[i]));
            }

            std::vector<types::Type *> arg_types;
            arg_types.push_back(memOp->obj->targetType());
            for (auto op : M->operands)
              arg_types.push_back(op->targetType());

            decls::Decl *found_func{};
            if (rec_type->decl) {
              for (auto &F : rec_type->decl->func) {
                if (F->name() == member_name) {
                  auto ftype = F->as<decls::FuncDecl>()->funcType->as<types::FuncType>();
                  if (ftype->pars.size() == arg_types.size()) {
                    bool match = true;
                    for (size_t i = 0; i < arg_types.size(); ++i) {
                      if (ftype->pars[i].type->typname() != arg_types[i]->typname()) {
                        match = false;
                        break;
                      }
                    }
                    if (match) {
                      found_func = F;
                      break;
                    }
                  }
                }
              }
            }

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
          if (M->operands.size() != ftype->pars.size()) {
            return errors::NoMatchOperator(now->pos(), "call", "function pointer", "arguments count mismatch");
          }
          for (size_t i = 0; i < M->operands.size(); ++i) {
            if_except(sema_Expr(M->operands[i]));
            if_except(sema_Convert(ftype->pars[i].type, M->operands[i], M->operands[i]->pos()));
          }
          now->targetType() = ftype->ret;
          return {};
        }

        return errors::NoMatchOperator(now->pos(), "call", std::string(M->obj->targetType()->typname()), "not a callable type");
      }
      ef (M->kind == exprs::PostfixOpEnum::Array) {
        if_except(sema_Expr(M->obj));

        auto obj_type = M->obj->targetType();
        while (obj_type->isReference()) {
          obj_type = obj_type->as<types::ReferenceType>()->sub;
        }

        if (M->operands.size() != 1) {
          return errors::NoMatchOperator(now->pos(), "[]", std::string(M->obj->targetType()->typname()), "dimension mismatch");
        }

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
        ef (obj_type->is<types::ZArrayType>()) { sub_type = obj_type->as<types::ZArrayType>()->sub; }
        else
        {
          return errors::NoMatchOperator(now->pos(), "[]", std::string(M->obj->targetType()->typname()), "not an array type");
        }

        now->targetType() = types::Type::make_Reference(ctx, sub_type);
        return {};
      }
      else
      {
        diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
        return {};
      }
    }
    ef(now->is<exprs::NickExpr>()) {
      auto ret = sema_NickExpr(now);

      if (ret.has_value()) {
        now = *ret;
        goto l_begin;
      }
      else
        return std::unexpected(std::move(ret.error()));
    }
    else {
      diagnostic::fatal(fatals::Internal_UnknownExpr().error()->msg());
      return {};
    }
  }

  fun Sema::sema_NickExpr(exprs::Expr *now) -> std::expected<exprs::Expr *, uptr<diagnostic::message>>
  {
    auto nick               = now->as<exprs::NickExpr>();
    const auto __join_human = [](std::vector<std::string> &ps) -> std::string {
      std::string ret = ps[0];
      for (size_t i = 1; i < ps.size(); i++)
        ret += "::" + ps[i];
      return ret;
    };

    auto ret = SMng.lookup(now->parent(), nick->unresolved);

    if (!ret)
      return errors::IdentifierNotFound(now->pos(), __join_human(nick->unresolved));

    if (ret->type() == IdentyEnum::Stmt && static_cast<stmts::Stmt *>(ret)->is<stmts::CodeVar>()) {
      auto stmt = static_cast<stmts::Stmt *>(ret);
      auto cvar = stmt->as<stmts::CodeVar>();

      auto var_expr          = exprs::Expr::make_VarExpr(ctx, now->parent(), stmt, now->pos());
      var_expr->targetType() = types::Type::make_Reference(ctx, cvar->targetType);

      return var_expr;
    }

    ef (auto expr = SMng.fetch_expr(ret)) { return expr; }

    else {
      return errors::IdentifierNExpr(now->pos(), __join_human(nick->unresolved));
    }
  }

  // Type
  fun Sema::sema_Type(types::Type *&now, word errpos) -> std::expected<void, uptr<diagnostic::message>>
  {
    assert(now && "null parameter.");

    l_begin:
    if (now->is<types::PrimitiveType>())
      return {};

    ef (now->is<types::RecordType>()) return sema_RecordType(now, errpos);
    ef (now->is<types::EnumType>())   return sema_EnumType(now, errpos);
    ef (now->is<types::SetType>())    return sema_SetType(now, errpos);

    ef (now->is<types::FuncType>()) {
      auto ftype                                  = now->as<types::FuncType>();
      std::vector<types::FieldType> resolved_pars = ftype->pars;
      for (auto &X : resolved_pars) {
        if_except(sema_Type(X.type, errpos));
      }
      auto resolved_ret = ftype->ret;
      if_except(sema_Type(resolved_ret, errpos));

      now = types::Type::make_Func(ctx, resolved_pars, resolved_ret);
      return {};
    }

    ef(now->is<types::PointerType>()) {
      auto sub = now->as<types::PointerType>()->sub;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_Pointer(ctx, sub);
      return {};
    }
    ef(now->is<types::ReferenceType>()) {
      auto sub = now->as<types::ReferenceType>()->sub;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_Reference(ctx, sub);
      return {};
    }
    ef(now->is<types::ZArrayType>()) {
      auto sub = now->as<types::ZArrayType>()->sub;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_ZArray(ctx, sub);
      return {};
    }
    ef(now->is<types::PArrayType>()) {
      auto parray = now->as<types::PArrayType>();
      auto sub    = parray->sub;
      u64 size    = parray->size;
      if_except(sema_Type(sub, errpos));
      now = types::Type::make_PArray(ctx, sub, size);
      return {};
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

    diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
    return {};
  }

  fun Sema::sema_EnumType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>>
  {
    if (now->sema() == StageStatus::Checked) return {};
    now->sema() = StageStatus::Checking;
    
    auto enumType = now->as<types::EnumType>();
    
    now->sema() = StageStatus::Checked;
    
    if (enumType->decl) {
      for (auto &F : enumType->decl->func) {
        if_except(sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }
    
    return {};
  }

  fun Sema::sema_SetType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>>
  {
    if (now->sema() == StageStatus::Checked) return {};
    now->sema() = StageStatus::Checking;
    
    auto setType = now->as<types::SetType>();
    
    now->sema() = StageStatus::Checked;
    
    if (setType->decl) {
      for (auto &F : setType->decl->func) {
        if_except(sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
    }

    return {};
  }

  fun Sema::sema_RecordType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>>
  {
    if (now->sema() == StageStatus::Checked)
      return {};
    ef(now->sema() == StageStatus::Checking) std::cerr << "!!!CHECKING" << std::endl;

    now->sema() = StageStatus::Checking;

    auto record = now->as<types::RecordType>();

    for (auto &X : record->vars)
      if_except(sema_Type(X.type, errpos));

    now->sema() = StageStatus::Checked;

    if (record->decl) {
      for (auto &F : record->decl->func) {
        if_except(sema_FuncDecl(F));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(F), F);
      }
      for (auto &C : record->decl->constructors) {
        if_except(sema_ConstructorDecl(C));
        ctx->gst().add_ident(scopemng::mangling_abi_qw(C), C);
      }
    }

    return {};
  }

  fun Sema::sema_NickType(types::Type *now, word errpos) -> std::expected<types::Type *, uptr<diagnostic::message>>
  {
    auto nick               = now->as<types::NickType>();
    const auto __join_human = [](std::vector<std::string> &ps) -> std::string {
      std::string ret = ps[0];
      for (int i = 1; i < ps.size(); i++)
        ret += "::" + ps[i];
      return ret;
    };

    auto ret = SMng.lookup(nullptr, nick->unresolved);

    if (!ret)
      return errors::IdentifierNotFound(errpos, __join_human(nick->unresolved));

    ef (auto typ = SMng.fetch_type(ret)) return typ;
    else
      return errors::IdentifierNType(errpos, __join_human(nick->unresolved));
  }

  fun Sema::sema_FuncType(types::Type *now, word errpos) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto ftype = now->as<types::FuncType>();
    for (auto &X : ftype->pars)
      if_except(sema_Type(X.type, errpos));
    if_except(sema_Type(ftype->ret, errpos));
    return {};
  }

  fun Sema::sema_SysIntrinsic(exprs::Expr *&now, const std::string &intrin, const std::vector<types::Type*> &gargs, const std::vector<exprs::Expr*> &args) -> std::expected<void, uptr<diagnostic::message>>
  {
    std::vector<types::Type*> resolved_gargs;
    for (auto t : gargs) {
      if_except(sema_Type(t, now->pos()));
      resolved_gargs.push_back(t);
    }

    if (intrin == "is_size") {
      if (resolved_gargs.size() != 1) return errors::UnexpectedIdentifier(now->pos(), "is_size expects 1 generic argument");
      now->vari() = exprs::IntegerLiteral{ (u128)8 };
      now->targetType() = ctx->intU64_t();
      return {};
    }
    
    bool res = false;
    
    if (intrin == "is_int" || intrin == "is_float" || intrin == "is_bool" || intrin == "is_char" || intrin == "is_void" || intrin == "is_ptr" || intrin == "is_signed" || intrin == "is_unsigned") {
      if (resolved_gargs.size() != 1) return errors::UnexpectedIdentifier(now->pos(), intrin + " expects 1 generic argument");
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
    ef (intrin == "is_pointer" || intrin == "is_reference" || intrin == "is_array" || intrin == "is_struct" || intrin == "is_function" || intrin == "is_enum") {
      if (resolved_gargs.size() != 1) return errors::UnexpectedIdentifier(now->pos(), intrin + " expects 1 generic argument");
      auto T = resolved_gargs[0];
      
      if (intrin == "is_pointer") res = T->is<types::PointerType>();
      ef (intrin == "is_reference") res = T->is<types::ReferenceType>();
      ef (intrin == "is_array") res = T->is<types::ZArrayType>() || T->is<types::PArrayType>();
      ef (intrin == "is_struct") res = T->is<types::RecordType>();
      ef (intrin == "is_function") res = T->is<types::FuncType>();
      ef (intrin == "is_enum") res = T->is<types::EnumType>();
      ef (intrin == "is_set") res = T->is<types::SetType>();
    }
    ef (intrin == "is_same") {
      if (resolved_gargs.size() != 2) return errors::UnexpectedIdentifier(now->pos(), intrin + " expects 2 generic arguments");
      res = (resolved_gargs[0]->typname() == resolved_gargs[1]->typname());
    }
    ef (intrin == "is_convertible") {
      if (resolved_gargs.size() != 2) return errors::UnexpectedIdentifier(now->pos(), intrin + " expects 2 generic arguments");
      auto dummy = exprs::Expr::make_IntegerLiteral(ctx, now, (i128)0, now->pos()); // Dummy
      dummy->targetType() = resolved_gargs[0];
      res = sema_Convert(resolved_gargs[1], dummy, now->pos()).has_value();
    }
    ef (intrin == "cast") {
      if (resolved_gargs.size() != 1) return errors::UnexpectedIdentifier(now->pos(), "cast expects 1 generic argument");
      if (args.size() != 1) return errors::UnexpectedIdentifier(now->pos(), "cast expects 1 argument");
      
      auto target_type = resolved_gargs[0];
      auto val = args[0];
      if_except(sema_Expr(val));
      
      auto src_type = val->targetType();
      while (src_type->isReference()) src_type = src_type->as<types::ReferenceType>()->sub;
      while (target_type->isReference()) target_type = target_type->as<types::ReferenceType>()->sub;
            if ((target_type->is<types::EnumType>() && src_type->isInteger()) ||
          (target_type->isInteger() && src_type->is<types::EnumType>()) ||
          (target_type->is<types::SetType>() && src_type->isInteger()) ||
          (target_type->isInteger() && src_type->is<types::SetType>())) {
          
        if (target_type->is<types::EnumType>() && val->is<exprs::IntegerLiteral>()) {
           auto enum_t = target_type->as<types::EnumType>();
           auto iv = val->as<exprs::IntegerLiteral>();
           
           i128 target_val = 0;
           if (std::holds_alternative<u128>(iv->val)) target_val = (i128)std::get<u128>(iv->val);
           else target_val = std::get<i128>(iv->val);

           bool found = false;
           for (auto &v : enum_t->vals) {
               i128 enum_v = 0;
               if (std::holds_alternative<u128>(v.val)) enum_v = (i128)std::get<u128>(v.val);
               else enum_v = std::get<i128>(v.val);

               if (target_val == enum_v) { found = true; break; }
           }
           if (!found) {
               return errors::UnexpectedIdentifier(now->pos(), "enum cast bounds check failed");
           }
        }

        if (target_type->is<types::SetType>() && val->is<exprs::IntegerLiteral>()) {
            auto set_t = target_type->as<types::SetType>();
            auto iv = val->as<exprs::IntegerLiteral>();
            
            i128 target_val = 0;
            if (std::holds_alternative<u128>(iv->val)) target_val = (i128)std::get<u128>(iv->val);
            else target_val = std::get<i128>(iv->val);

            i128 max_v = 0;
            for (auto &v : set_t->vals) {
                i128 set_v = 0;
                if (std::holds_alternative<u128>(v.val)) set_v = (i128)std::get<u128>(v.val);
                else set_v = std::get<i128>(v.val);
                if (set_v > max_v) max_v = set_v;
            }
            
            i128 allowed_max = (max_v << 1) - 1;
            if (target_val < 0 || target_val > allowed_max) {
                return errors::UnexpectedIdentifier(now->pos(), "set cast bounds check failed");
            }
         }
         
        if (target_type->isInteger() && target_type->is<types::PrimitiveType>() && val->is<exprs::IntegerLiteral>()) {
            auto iv = val->as<exprs::IntegerLiteral>();
            auto pt = target_type->as<types::PrimitiveType>();
            // Strict bit-width checking based on user prompt "Örneğin 257 değerindeki bir enum u8 e cast edilememeli"
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
                if (std::get<u128>(iv->val) > max_val) return errors::UnexpectedIdentifier(now->pos(), "enum to int cast bounds check failed");
            } else {
                if ((u128)std::get<i128>(iv->val) > max_val) return errors::UnexpectedIdentifier(now->pos(), "enum to int cast bounds check failed");
            }
        }
        
        now = exprs::Expr::make_UnaryOp(ctx, now->parent(), exprs::UnaryOpEnum::Plus, val, now->pos());
        now->targetType() = target_type;
        return {};
      }
      
      return errors::UnexpectedIdentifier(now->pos(), "sys::cast only supports enum <-> int for now");
    }
    else {
      return errors::UnexpectedIdentifier(now->pos(), intrin);
    }
    
    now->vari() = exprs::BoolLiteral{ res };
    now->targetType() = ctx->bool_t();
    
    return {};
  }

}
