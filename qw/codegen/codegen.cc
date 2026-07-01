/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/codegen/codegen.hh"
#include "qw/basis.hh"
#include "qw/control/scopemng.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"

#include <llvm/IR/BasicBlock.h>
#include <llvm/IR/Constants.h>
#include <llvm/IR/DerivedTypes.h>
#include <llvm/IR/Function.h>
#include <llvm/IR/GlobalValue.h>
#include <llvm/IR/Type.h>
#include <llvm/IR/Value.h>
#include <string>

#define ef else if

#define if_error(X)                                                                                                                                  \
  {                                                                                                                                                  \
    auto E = X;                                                                                                                                      \
    if (!E.has_value()) {                                                                                                                            \
      if (E.error()->type() == qw::diagnostic::qMsgType::mtFatal)                                                                                    \
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
  fun CodeGen::gen_NameSpace(decls::Decl *now) -> void
  {
    SMng.ans().push_back(scopemng::mangling_abi_qw(now));

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      if (X->is<decls::NameSpaceDecl>()) gen_NameSpace(X);
      ef (X->is<decls::FuncDecl>())      gen_FuncDecl(X);
      ef (X->is<decls::VarDecl>())       gen_VarDecl(X);

      ef (X->is<decls::TypeDecl>()) {
        auto tdecl = X->as<decls::TypeDecl>();
        if (tdecl->type && tdecl->type->is<types::StructType>()) {
          auto ret = gen_Type(tdecl->type);
          auto rec = tdecl->type->as<types::StructType>();
          
          if (rec->decl) {
            for (auto &F: rec->decl->func) gen_FuncDecl(F);
            for (auto &C: rec->decl->constructors) gen_ConstructorDecl(C);
          }
          
        }
        ef (tdecl->type && tdecl->type->is<types::EnumType>()) {
          auto enum_t = tdecl->type->as<types::EnumType>();
          if (enum_t->decl) {
            for (auto &F: enum_t->decl->func) gen_FuncDecl(F);
          }
        }
      }
    }

    SMng.ans().pop_back();
  }

  fun CodeGen::gen_FuncDecl(decls::Decl *now) -> void
  {
    auto fdecl         = now->as<decls::FuncDecl>();
    types::Type *ftype = fdecl->funcType;

    auto res = gen_Type(ftype);
    // if_except(res);

    fdecl->llvm = llvm::Function::Create(
      llvm::cast<llvm::FunctionType>(ftype->llvm()), llvm::GlobalValue::ExternalLinkage, scopemng::mangling_abi_qw(now), mod->llvm()
    );

    if (fdecl->body) {
      auto BB = llvm::BasicBlock::Create(*ctx->llvm(), "entry", fdecl->llvm);
      IR.SetInsertPoint(BB);

      gen_CodeBlock(ftype->as<types::FuncType>()->ret, fdecl->body);
    }
  }

  fun CodeGen::gen_ConstructorDecl(decls::Decl *now) -> void
  {
    auto cdecl         = now->as<decls::ConstructorDecl>();
    types::Type *ftype = cdecl->funcType;

    auto res = gen_Type(ftype);
    // if_except(res);

    cdecl->llvm = llvm::Function::Create(
      llvm::cast<llvm::FunctionType>(ftype->llvm()), llvm::GlobalValue::ExternalLinkage, scopemng::mangling_abi_qw(now), mod->llvm()
    );

    if (cdecl->body) {
      auto BB = llvm::BasicBlock::Create(*ctx->llvm(), "entry", cdecl->llvm);
      IR.SetInsertPoint(BB);

      // Gen inits
      auto ftype_concrete = ftype->as<types::FuncType>();
      if (ftype_concrete->pars.size() > 0) {
        auto self_ref = ftype_concrete->pars[0].type->as<types::ReferenceType>();
        if (self_ref && self_ref->sub->is<types::StructType>()) {
          auto recType = self_ref->sub->as<types::StructType>();
          
          auto self_val = cdecl->llvm->arg_begin();
          
          for (auto &init_pair: cdecl->inits) {
            u32 idx = 0;
            for (auto &v: recType->vars) {
              if (v.name == init_pair.first) break;
              idx++;
            }
            
            auto val = gen_Expr(init_pair.second);
            val = gen_Convert(recType->vars[idx].type, init_pair.second);

            auto gep = IR.CreateStructGEP(self_ref->sub->llvm(), &*self_val, idx);
            IR.CreateStore(val, gep);
          }
        }
      }

      gen_CodeBlock(ftype_concrete->ret, cdecl->body);

      if (!IR.GetInsertBlock()->getTerminator()) {
        IR.CreateRetVoid();
      }
    }
  }

  fun CodeGen::gen_VarDecl(decls::Decl *now) -> void
  {
    // Gelecekte Global Değişken (GlobalVariable) eklendiğinde burası doldurulacak.
  }

  // Stat
  fun CodeGen::gen_CodeBlock(types::Type *expected_ret, stmts::Stmt *now) -> void
  {
    auto block = now->as<stmts::CodeBlock>();

    for (auto X : block->vars)
      gen_VarStmt(X);

    // Pars
    if (now->parent() && now->parent()->type() == IdentyEnum::Decl) {
      if (static_cast<decls::Decl*>(now->parent())->is<decls::FuncDecl>()) {
        auto fdecl = static_cast<decls::Decl*>(now->parent())->as<decls::FuncDecl>();
        auto ftype = fdecl->funcType->as<types::FuncType>();

        auto arg_it = fdecl->llvm->arg_begin();
        for (const auto &p: ftype->pars) {
          for (auto X: block->vars) {
            auto cvar = X->as<stmts::CodeVar>();
            if (cvar->name == p.name) {
              llvm::Value *arg_val = &*arg_it;
              IR.CreateStore(arg_val, cvar->llvm);
              break;
            }
          }
          arg_it++;
        }
      }
      ef (static_cast<decls::Decl*>(now->parent())->is<decls::ConstructorDecl>()) {
        auto cdecl = static_cast<decls::Decl*>(now->parent())->as<decls::ConstructorDecl>();
        auto ftype = cdecl->funcType->as<types::FuncType>();

        auto arg_it = cdecl->llvm->arg_begin();
        for (const auto &p: ftype->pars) {
          for (auto X: block->vars) {
            auto cvar = X->as<stmts::CodeVar>();
            if (cvar->name == p.name) {
              llvm::Value *arg_val = &*arg_it;
              IR.CreateStore(arg_val, cvar->llvm);
              break;
            }
          }
          arg_it++;
        }
      }
    }

    // Codes
    for (auto &X: block->codes) {
      if (X->is<stmts::ReturnStmt>()) {
        gen_ReturnStmt(expected_ret, X);
        break;
      }
      ef (X->is<stmts::ExprStmt>()) {
        auto expr_stmt = X->as<stmts::ExprStmt>();
        gen_Expr(expr_stmt->expr);
      }
      ef (X->is<stmts::IfStmt>()) {
        gen_IfStmt(expected_ret, X);
      }
      ef (X->is<stmts::WhileStmt>()) {
        gen_WhileStmt(expected_ret, X);
      }
      ef (X->is<stmts::BreakStmt>()) {
        if (!loop_stack.empty()) {
          IR.CreateBr(loop_stack.back().second);
        }
        break;
      }
      ef (X->is<stmts::ContinueStmt>()) {
        if (!loop_stack.empty()) {
          IR.CreateBr(loop_stack.back().first);
        }
        break;
      }
      ef (X->is<stmts::CodeBlock>()) {
        gen_CodeBlock(expected_ret, X);
      }
    }
  }

  fun CodeGen::gen_IfStmt(types::Type *expected_ret, stmts::Stmt *now) -> void
  {
    auto if_stmt = now->as<stmts::IfStmt>();
    
    llvm::Function *TheFunction = IR.GetInsertBlock()->getParent();
    llvm::BasicBlock *ThenBB = llvm::BasicBlock::Create(*ctx->llvm(), "", TheFunction);
    llvm::BasicBlock *ElseBB = llvm::BasicBlock::Create(*ctx->llvm());
    llvm::BasicBlock *MergeBB = llvm::BasicBlock::Create(*ctx->llvm());

    llvm::Value *CondV = gen_Convert(ctx->bool_t(), if_stmt->condition);
    IR.CreateCondBr(CondV, ThenBB, if_stmt->else_block ? ElseBB : MergeBB);

    IR.SetInsertPoint(ThenBB);
    if (if_stmt->then_block) {
      gen_CodeBlock(expected_ret, if_stmt->then_block);
    }
    if (!IR.GetInsertBlock()->getTerminator())
      IR.CreateBr(MergeBB);

    if (if_stmt->else_block) {
      TheFunction->insert(TheFunction->end(), ElseBB);
      IR.SetInsertPoint(ElseBB);
      
      if (if_stmt->else_block->is<stmts::IfStmt>()) {
        gen_IfStmt(expected_ret, if_stmt->else_block);
      }
      else {
        gen_CodeBlock(expected_ret, if_stmt->else_block);
      }
      
      if (!IR.GetInsertBlock()->getTerminator())
        IR.CreateBr(MergeBB);
    }
    else {
      delete ElseBB;
    }

    TheFunction->insert(TheFunction->end(), MergeBB);
    IR.SetInsertPoint(MergeBB);
  }

  fun CodeGen::gen_WhileStmt(types::Type *expected_ret, stmts::Stmt *now) -> void
  {
    auto while_stmt = now->as<stmts::WhileStmt>();

    llvm::Function *TheFunction = IR.GetInsertBlock()->getParent();
    llvm::BasicBlock *CondBB = llvm::BasicBlock::Create(*ctx->llvm(), "", TheFunction);
    llvm::BasicBlock *LoopBB = llvm::BasicBlock::Create(*ctx->llvm());
    llvm::BasicBlock *MergeBB = llvm::BasicBlock::Create(*ctx->llvm());

    IR.CreateBr(CondBB);
    IR.SetInsertPoint(CondBB);

    llvm::Value *CondV = gen_Convert(ctx->bool_t(), while_stmt->condition);
    IR.CreateCondBr(CondV, LoopBB, MergeBB);

    TheFunction->insert(TheFunction->end(), LoopBB);
    IR.SetInsertPoint(LoopBB);

    loop_stack.push_back({CondBB, MergeBB});

    if (while_stmt->body) {
      gen_CodeBlock(expected_ret, while_stmt->body);
    }

    loop_stack.pop_back();

    if (!IR.GetInsertBlock()->getTerminator())
      IR.CreateBr(CondBB);

    TheFunction->insert(TheFunction->end(), MergeBB);
    IR.SetInsertPoint(MergeBB);
  }

  fun CodeGen::gen_VarStmt(stmts::Stmt *now) -> void
  {
    auto cvar  = now->as<stmts::CodeVar>();
    auto res   = gen_Type(cvar->targetType);
    cvar->llvm = IR.CreateAlloca(cvar->targetType->llvm());
  }

  fun CodeGen::gen_ReturnStmt(types::Type *expected_ret, stmts::Stmt *now) -> void
  {
    auto ret = now->as<stmts::ReturnStmt>();

    llvm::Value *val = gen_Convert(expected_ret, ret->expr);
    IR.CreateRet(val);
  }

  // Expr
  fun CodeGen::gen_Convert(types::Type *typ, exprs::Expr *val) -> llvm::Value*
  {
    llvm::Value *src  = gen_Expr(val);
    types::Type *styp = val->targetType();

    l_re:
    if (typ == styp)
      return src;

    ef (styp->isReference()) {
      styp = styp->as<types::ReferenceType>()->sub;
      src  = IR.CreateLoad(styp->llvm(), src);
      goto l_re;
    }

    ef (typ->isInteger() && styp->isInteger()) { return IR.CreateIntCast(src, typ->llvm(), typ->isSigned() && styp->isSigned()); }
    ef (typ->is<types::EnumType>() && styp->isInteger()) { return IR.CreateIntCast(src, typ->llvm(), false); }
    ef (typ->isInteger() && styp->is<types::EnumType>()) { return IR.CreateIntCast(src, typ->llvm(), typ->isSigned()); }
    ef (typ->is<types::EnumType>() && styp->is<types::EnumType>()) { return IR.CreateIntCast(src, typ->llvm(), false); }
    ef (typ->isChar() && styp->isInteger()) { return IR.CreateIntCast(src, typ->llvm(), false); }
    ef (typ->isInteger() && styp->isChar()) { return IR.CreateIntCast(src, typ->llvm(), false); }

    return src;
  }

  fun CodeGen::gen_Expr(exprs::Expr *now) -> llvm::Value*
  {
    if (now->is<exprs::IntegerLiteral>()) {
      auto lit       = now->as<exprs::IntegerLiteral>();
      types::Type *t = now->targetType() ? now->targetType() : ctx->intS32_t(); // Fallback

      if (std::holds_alternative<u128>(lit->val))
        return llvm::ConstantInt::get(t->llvm(), std::get<u128>(lit->val), false);
      else
        return llvm::ConstantInt::get(t->llvm(), std::get<i128>(lit->val), true);
    }
    ef (now->is<exprs::FloatingLiteral>()) {
      auto lit       = now->as<exprs::FloatingLiteral>();
      types::Type *t = now->targetType() ? now->targetType() : ctx->flo32_t();
      return llvm::ConstantFP::get(t->llvm(), lit->val);
    }
    ef (now->is<exprs::CharLiteral>()) {
      auto lit = now->as<exprs::CharLiteral>();
      return llvm::ConstantInt::get(ctx->char_t()->llvm(), lit->val, false);
    }
    ef (now->is<exprs::BoolLiteral>()) {
      auto lit = now->as<exprs::BoolLiteral>();
      return llvm::ConstantInt::get(ctx->bool_t()->llvm(), lit->val, false);
    }
    ef (now->is<exprs::PtrLiteral>()) {
      types::Type *t = now->targetType() ? now->targetType() : ctx->ptr_t();
      return llvm::ConstantPointerNull::get(llvm::cast<llvm::PointerType>(t->llvm()));
    }
    ef (now->is<exprs::StringLiteral>()) {
      auto lit = now->as<exprs::StringLiteral>();
      now->llvm() = IR.CreateGlobalString(lit->val, "", 0, mod->llvm());
      return now->llvm();
    }
    ef (now->is<exprs::VarExpr>()) {
      auto vexpr = now->as<exprs::VarExpr>();
      auto cvar  = vexpr->var->as<stmts::CodeVar>();

      if (cvar->targetType && cvar->targetType->isReference()) {
        return IR.CreateLoad(llvm::PointerType::getUnqual(*ctx->llvm()), cvar->llvm);
      }

      return cvar->llvm;
    }
    ef (now->is<exprs::ValExpr>()) {
      return now->llvm();
    }
    ef (now->is<exprs::UnaryOp>()) {
      auto U = now->as<exprs::UnaryOp>();
      if (U->kind == exprs::UnaryOpEnum::AddrOf) {
        return gen_Expr(U->o1);
      }
      else if (U->kind == exprs::UnaryOpEnum::Minus) {
        auto v = gen_Convert(now->targetType(), U->o1);
        return now->targetType()->isFloat() ? IR.CreateFNeg(v) : IR.CreateNeg(v);
      }
      else if (U->kind == exprs::UnaryOpEnum::Plus) {
        return gen_Convert(now->targetType(), U->o1);
      }
      else if (U->kind == exprs::UnaryOpEnum::LNot) {
        auto v = gen_Convert(now->targetType(), U->o1);
        return IR.CreateNot(v);
      }
      else if (U->kind == exprs::UnaryOpEnum::BitNot) {
        auto v = gen_Convert(now->targetType(), U->o1);
        return IR.CreateNot(v);
      }
      return nullptr;
    }
    ef (now->is<exprs::BinaryOp>()) {
      auto C = now->as<exprs::BinaryOp>();

      bool is_compound_assign = (C->kind >= exprs::BinaryOpEnum::AddAssign && C->kind <= exprs::BinaryOpEnum::ShrAssign);

      if (C->kind == exprs::BinaryOpEnum::Assign || is_compound_assign) {
        llvm::Value *ptr = gen_Expr(C->o1);

        auto lhs_type = C->o1->targetType();
        while (lhs_type->isReference()) {
          lhs_type = lhs_type->as<types::ReferenceType>()->sub;
        }

        llvm::Value *val2 = gen_Convert(lhs_type, C->o2);
        
        if (is_compound_assign) {
           llvm::Value *val1 = IR.CreateLoad(lhs_type->llvm(), ptr);
           bool is_float = lhs_type->isFloat();
           bool is_signed = lhs_type->isSigned();

           switch (C->kind) {
             case exprs::BinaryOpEnum::AddAssign: val2 = is_float ? IR.CreateFAdd(val1, val2) : IR.CreateAdd(val1, val2); break;
             case exprs::BinaryOpEnum::SubAssign: val2 = is_float ? IR.CreateFSub(val1, val2) : IR.CreateSub(val1, val2); break;
             case exprs::BinaryOpEnum::MulAssign: val2 = is_float ? IR.CreateFMul(val1, val2) : IR.CreateMul(val1, val2); break;
             case exprs::BinaryOpEnum::DivAssign:
               if (is_float) val2 = IR.CreateFDiv(val1, val2);
               else val2 = is_signed ? IR.CreateSDiv(val1, val2) : IR.CreateUDiv(val1, val2);
               break;
             case exprs::BinaryOpEnum::RemAssign:
               if (is_float) val2 = IR.CreateFRem(val1, val2);
               else val2 = is_signed ? IR.CreateSRem(val1, val2) : IR.CreateURem(val1, val2);
               break;
             case exprs::BinaryOpEnum::BitAndAssign: val2 = IR.CreateAnd(val1, val2); break;
             case exprs::BinaryOpEnum::BitOrAssign:  val2 = IR.CreateOr(val1, val2); break;
             case exprs::BinaryOpEnum::BitXorAssign: val2 = IR.CreateXor(val1, val2); break;
             case exprs::BinaryOpEnum::ShlAssign:    val2 = IR.CreateShl(val1, val2); break;
             case exprs::BinaryOpEnum::ShrAssign:    val2 = is_signed ? IR.CreateAShr(val1, val2) : IR.CreateLShr(val1, val2); break;
             default: break;
           }
        }

        IR.CreateStore(val2, ptr);
        return val2;
      }

      types::Type *computationType = C->computationType ? C->computationType : now->targetType();
      llvm::Value *v1 = gen_Convert(computationType, C->o1);
      llvm::Value *v2 = gen_Convert(computationType, C->o2);

      bool is_float = computationType->isFloat();
      bool is_signed = computationType->isSigned();

      switch (C->kind) {
        case exprs::BinaryOpEnum::Add: return is_float ? IR.CreateFAdd(v1, v2) : IR.CreateAdd(v1, v2);
        case exprs::BinaryOpEnum::Sub: return is_float ? IR.CreateFSub(v1, v2) : IR.CreateSub(v1, v2);
        case exprs::BinaryOpEnum::Mul: return is_float ? IR.CreateFMul(v1, v2) : IR.CreateMul(v1, v2);

        case exprs::BinaryOpEnum::Div:
          if (is_float) return IR.CreateFDiv(v1, v2);
          return is_signed ? IR.CreateSDiv(v1, v2) : IR.CreateUDiv(v1, v2);

        case exprs::BinaryOpEnum::Rem:
          if (is_float) return IR.CreateFRem(v1, v2);
          return is_signed ? IR.CreateSRem(v1, v2) : IR.CreateURem(v1, v2);

        case exprs::BinaryOpEnum::BitAnd: return IR.CreateAnd(v1, v2);
        case exprs::BinaryOpEnum::BitOr:  return IR.CreateOr(v1, v2);
        case exprs::BinaryOpEnum::BitXor: return IR.CreateXor(v1, v2);
        case exprs::BinaryOpEnum::Shl:    return IR.CreateShl(v1, v2);
        case exprs::BinaryOpEnum::LShr:   return IR.CreateLShr(v1, v2);
        case exprs::BinaryOpEnum::AShr:   return IR.CreateAShr(v1, v2);

        case exprs::BinaryOpEnum::Eq:
          return is_float ? IR.CreateFCmpOEQ(v1, v2) : IR.CreateICmpEQ(v1, v2);
        case exprs::BinaryOpEnum::NEq:
          return is_float ? IR.CreateFCmpONE(v1, v2) : IR.CreateICmpNE(v1, v2);
        case exprs::BinaryOpEnum::Lt:
          return is_float ? IR.CreateFCmpOLT(v1, v2) : (is_signed ? IR.CreateICmpSLT(v1, v2) : IR.CreateICmpULT(v1, v2));
        case exprs::BinaryOpEnum::Gt:
          return is_float ? IR.CreateFCmpOGT(v1, v2) : (is_signed ? IR.CreateICmpSGT(v1, v2) : IR.CreateICmpUGT(v1, v2));
        case exprs::BinaryOpEnum::LEq:
          return is_float ? IR.CreateFCmpOLE(v1, v2) : (is_signed ? IR.CreateICmpSLE(v1, v2) : IR.CreateICmpULE(v1, v2));
        case exprs::BinaryOpEnum::GEq:
          return is_float ? IR.CreateFCmpOGE(v1, v2) : (is_signed ? IR.CreateICmpSGE(v1, v2) : IR.CreateICmpUGE(v1, v2));

        default: diagnostic::fatal("CodeGen: Unimplemented BinaryOp!"); return nullptr;
      }
    }
    ef (now->is<exprs::PostfixOp>()) {
      auto M = now->as<exprs::PostfixOp>();
      if (M->kind == exprs::PostfixOpEnum::Deref) {
        types::Type *ptrType = M->obj->targetType();
        if (ptrType->isReference()) ptrType = ptrType->as<types::ReferenceType>()->sub;
        return gen_Convert(ptrType, M->obj);
      }
      ef (M->kind == exprs::PostfixOpEnum::Call) {
        llvm::Function *calleeFn = nullptr;

        if (M->obj->is<exprs::MemberOp>()) {
          auto memOp    = M->obj->as<exprs::MemberOp>();
          auto obj_type = memOp->obj->targetType();
          while (obj_type->isReference()) {
            obj_type = obj_type->as<types::ReferenceType>()->sub;
          }
          auto rec_type    = obj_type->as<types::StructType>();
          auto member_name = memOp->mem->as<exprs::NickExpr>()->unresolved[0];

          std::vector<types::Type *> arg_types;
          for (auto op : M->operands)
            arg_types.push_back(op->targetType());

          if (rec_type->decl) {
            for (auto &F : rec_type->decl->func) {
              if (F->name() == member_name) {
                auto ftype = F->as<decls::FuncDecl>()->funcType->as<types::FuncType>();
                if (ftype->pars.size() == arg_types.size()) {
                  bool match = true;
                  for (size_t i = 0; i < arg_types.size(); i++)
                    if (ftype->pars[i].type->typname() != arg_types[i]->typname()) {
                      match = false;
                      break;
                    }
                    
                  if (match) {
                    calleeFn = F->as<decls::FuncDecl>()->llvm;
                    break;
                  }
                }
              }
            }
          }
        }
        ef (M->obj->is<exprs::NickExpr>()) {
          auto nick = M->obj->as<exprs::NickExpr>();
          
          std::vector<types::Type *> arg_types;
          for (auto op : M->operands)
            arg_types.push_back(op->targetType());
            
          auto ret  = SMng.lookup(now->parent(), nick->unresolved, &arg_types);
          if (ret && ret->type() == IdentyEnum::Decl && static_cast<decls::Decl *>(ret)->is<decls::FuncDecl>()) {
            auto fdecl = static_cast<decls::Decl *>(ret)->as<decls::FuncDecl>();
            if (!fdecl->llvm) {
                auto _r = gen_Type(fdecl->funcType);
                fdecl->llvm = llvm::Function::Create(
                    llvm::cast<llvm::FunctionType>(fdecl->funcType->llvm()), llvm::GlobalValue::ExternalLinkage, scopemng::mangling_abi_qw(ret), mod->llvm()
                );
            }
            calleeFn = fdecl->llvm;
          }
        }

        if (calleeFn) {
          std::vector<llvm::Value *> args;
          auto ftype = M->obj->targetType()->as<types::FuncType>();
          for (size_t i = 0; i < M->operands.size(); ++i) {
            args.push_back(gen_Convert(ftype->pars[i].type, M->operands[i]));
          }
          return IR.CreateCall(calleeFn, args);
        }

        llvm::Value *calleeVal = gen_Expr(M->obj);
        auto ftype             = M->obj->targetType()->as<types::FuncType>();
        std::vector<llvm::Value *> args;
        for (size_t i = 0; i < M->operands.size(); ++i) {
          args.push_back(gen_Convert(ftype->pars[i].type, M->operands[i]));
        }
        return IR.CreateCall(llvm::cast<llvm::FunctionType>(M->obj->targetType()->llvm()), calleeVal, args);
      }
      ef (M->kind == exprs::PostfixOpEnum::Array) {
        llvm::Value *obj_ptr = gen_Expr(M->obj);
        auto obj_type        = M->obj->targetType();

        while (obj_type->isReference()) {
          obj_type = obj_type->as<types::ReferenceType>()->sub;
        }

        llvm::Value *idx = gen_Expr(M->operands[0]);

        if (obj_type->is<types::PArrayType>()) {
          llvm::Value *zero = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), 0);
          return IR.CreateInBoundsGEP(obj_type->llvm(), obj_ptr, {zero, idx});
        }
        ef (obj_type->is<types::ZArrayType>()) {
          llvm::Value *elem_ptr = IR.CreateLoad(obj_type->llvm(), obj_ptr);
          auto sub_type         = obj_type->as<types::ZArrayType>()->sub;
          return IR.CreateInBoundsGEP(sub_type->llvm(), elem_ptr, idx);
        }
        else {
          diagnostic::fatal("CodeGen: Unknown Expr Type!");
          return nullptr;
        }
      }
      else {
        diagnostic::fatal("CodeGen: Unknown Expr Type!");
        return nullptr;
      }
    }
    ef (now->is<exprs::MemberOp>()) {
      auto M = now->as<exprs::MemberOp>();

      llvm::Value *obj_ptr = gen_Expr(M->obj);
      auto obj_type        = M->obj->targetType();

      while (obj_type->isReference()) {
        obj_type = obj_type->as<types::ReferenceType>()->sub;
      }

      auto ret = gen_Type(obj_type);

      auto rec_type   = obj_type->as<types::StructType>();
      auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];

      u32 index{};
      for (size_t i = 0; i < rec_type->vars.size(); ++i) {
        if (rec_type->vars[i].name == field_name) {
          index = i;
          break;
        }
      }

      llvm::Value *zero = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), 0);
      llvm::Value *idx  = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), index);

      return IR.CreateInBoundsGEP(obj_type->llvm(), obj_ptr, {zero, idx});
    }
    else {
      diagnostic::fatal("CodeGen: Unknown Expr Type!");
      return nullptr;
    }
  }

  // Type
  fun CodeGen::gen_Type(types::Type *&now) -> std::expected<void, uptr<diagnostic::message>>
  {
    l_begin:
    if (now->is<types::PrimitiveType>())
      return {};

    ef (now->is<types::StructType>()) return gen_StructType(now);
    ef (now->is<types::FuncType>())   return gen_FuncType(now);
    ef (now->is<types::PArrayType>()) {
      auto parray = now->as<types::PArrayType>();
      if_except(gen_Type(parray->sub));
      now->llvm() = llvm::ArrayType::get(parray->sub->llvm(), parray->size);
      return {};
    }
    ef (now->is<types::EnumType>() || now->is<types::SetType>()) return {};
    ef (now->is<types::PointerType>() || now->is<types::ReferenceType>() || now->is<types::ZArrayType>()) return {};

    diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
    return {};
  }

  fun CodeGen::gen_StructType(types::Type *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    if (now->cgen() == StageStatus::Checked) return {};
    ef (now->cgen() == StageStatus::Checking) std::cerr << "!!!CHECKING" << std::endl;

    now->cgen() = StageStatus::Checking;

    auto strct = now->as<types::StructType>();
    for (auto &X: strct->vars)
      if_except(gen_Type(X.type));

    std::vector<llvm::Type*> LTyps;
    for (auto &v: strct->vars)
      LTyps.push_back(v.type->llvm());

    now->llvm() = llvm::StructType::create(*ctx->llvm(), LTyps, "typerec" + std::to_string(m_counter_typerec++));
    now->cgen() = StageStatus::Checked;

    return {};
  }

  fun CodeGen::gen_FuncType(types::Type *now) -> std::expected<void, uptr<diagnostic::message>>
  {
    auto ftype = now->as<types::FuncType>();
    for (auto &X: ftype->pars)
      if_except(gen_Type(X.type));
    if_except(gen_Type(ftype->ret));

    std::vector<llvm::Type *> llvm_params;
    llvm_params.reserve(ftype->pars.size());
    for (auto [s, t, v]: ftype->pars) {
      llvm_params.push_back(t->llvm());
    }
    now->llvm() = llvm::FunctionType::get(ftype->ret->llvm(), llvm_params, false);

    return {};
  }

}
