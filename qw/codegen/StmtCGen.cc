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
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"

#include <llvm/IR/BasicBlock.h>
#include <llvm/IR/Constants.h>
#include <llvm/IR/DerivedTypes.h>
#include <llvm/IR/InlineAsm.h>
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

  static fun get_symbol_name(decls::Decl* now) -> std::string {
    for (auto& attr : now->const_attrs()) {
      if (attr.name == "symbol") {
        if (attr.value == "bare") return std::string(now->name());
        ef (attr.value == "qw") return scopemng::mangling_abi_qw(now);
      }
    }
    return scopemng::mangling_abi_qw(now);
  }

  static fun setup_global_attrs(decls::Decl* now, llvm::GlobalVariable* gvar) -> void {
    for (auto& attr : now->const_attrs()) {
      if (attr.name == "weak") {
        gvar->setLinkage(llvm::GlobalValue::WeakAnyLinkage);
      }
      ef (attr.name == "thread_local") {
        gvar->setThreadLocal(true);
      }
    }
  }

  static fun setup_function_attrs(decls::Decl* now, llvm::Function* func) -> void {
    for (auto& attr : now->const_attrs()) {
      if (attr.name == "weak") {
        func->setLinkage(llvm::GlobalValue::WeakAnyLinkage);
      }
      ef (attr.name == "calling") {
        if (attr.value == "fast")  func->setCallingConv(llvm::CallingConv::Fast);
        ef (attr.value == "cdecl") func->setCallingConv(llvm::CallingConv::C);
        ef (attr.value == "cold")  func->setCallingConv(llvm::CallingConv::Cold);
      }
    }
  }



  fun StmtCGen::gen_CodeBlock(types::Type *expected_ret, stmts::Stmt *now) -> void {
    auto block = now->as<stmts::CodeBlock>();
    cctx.meta->active_blocks.push_back(block);

    for (auto X : block->vars)
      cctx.stmt->gen_VarStmt(X);

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
      ef (static_cast<decls::Decl*>(now->parent())->is<decls::DestructorDecl>()) {
        auto cdecl = static_cast<decls::Decl*>(now->parent())->as<decls::DestructorDecl>();
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
        cctx.stmt->gen_ReturnStmt(expected_ret, X);
        break;
      }
      ef (X->is<stmts::ExprStmt>()) {
        auto expr_stmt = X->as<stmts::ExprStmt>();
        cctx.expr->gen_Expr(expr_stmt->expr);
      }
      ef (X->is<stmts::IfStmt>())    cctx.stmt->gen_IfStmt(expected_ret, X);
      ef (X->is<stmts::WhileStmt>()) cctx.stmt->gen_WhileStmt(expected_ret, X);
      ef (X->is<stmts::CodeBlock>()) cctx.stmt->gen_CodeBlock(expected_ret, X);
      ef (X->is<stmts::UnsafeStmt>()) cctx.stmt->gen_UnsafeStmt(expected_ret, X);
      ef (X->is<stmts::BreakStmt>()) {
        auto break_stmt = X->as<stmts::BreakStmt>();
        if (break_stmt->loop) {
          cctx.decl->call_destructors(break_stmt->loop->as<stmts::WhileStmt>()->body->as<stmts::CodeBlock>());
          IR.CreateBr(break_stmt->loop->as<stmts::WhileStmt>()->end_block);
        }
        
        break;
      }
      ef (X->is<stmts::ContinueStmt>()) {
        auto continue_stmt = X->as<stmts::ContinueStmt>();
        if (continue_stmt->loop) {
          cctx.decl->call_destructors(continue_stmt->loop->as<stmts::WhileStmt>()->body->as<stmts::CodeBlock>());
          IR.CreateBr(continue_stmt->loop->as<stmts::WhileStmt>()->cond_block);
        }
        
        break;
      }
    }

    if (!IR.GetInsertBlock()->getTerminator()) {
      cctx.decl->call_destructors(block);
    }
    cctx.meta->active_blocks.pop_back();
  }

  fun StmtCGen::gen_IfStmt(types::Type *expected_ret, stmts::Stmt *now) -> void {
    auto if_stmt = now->as<stmts::IfStmt>();
    
    llvm::Function *TheFunction = IR.GetInsertBlock()->getParent();
    llvm::BasicBlock *ThenBB = llvm::BasicBlock::Create(*ctx->llvm(), "", TheFunction);
    llvm::BasicBlock *ElseBB = llvm::BasicBlock::Create(*ctx->llvm());
    llvm::BasicBlock *MergeBB = llvm::BasicBlock::Create(*ctx->llvm());

    llvm::Value *CondV = cctx.expr->gen_Convert(ctx->bool_t(), if_stmt->condition);
    IR.CreateCondBr(CondV, ThenBB, if_stmt->else_block ? ElseBB : MergeBB);

    IR.SetInsertPoint(ThenBB);
    
    if (if_stmt->then_block)
      cctx.stmt->gen_CodeBlock(expected_ret, if_stmt->then_block);
    
    if (!IR.GetInsertBlock()->getTerminator())
      IR.CreateBr(MergeBB);

    if (if_stmt->else_block) {
      TheFunction->insert(TheFunction->end(), ElseBB);
      IR.SetInsertPoint(ElseBB);
      
      if (if_stmt->else_block->is<stmts::IfStmt>())
        cctx.stmt->gen_IfStmt(expected_ret, if_stmt->else_block);
      else
        cctx.stmt->gen_CodeBlock(expected_ret, if_stmt->else_block);
      
      if (!IR.GetInsertBlock()->getTerminator())
        IR.CreateBr(MergeBB);
    }
    else delete ElseBB;

    TheFunction->insert(TheFunction->end(), MergeBB);
    IR.SetInsertPoint(MergeBB);
  }

  fun StmtCGen::gen_WhileStmt(types::Type *expected_ret, stmts::Stmt *now) -> void {
    auto while_stmt = now->as<stmts::WhileStmt>();

    llvm::Function *TheFunction = IR.GetInsertBlock()->getParent();
    llvm::BasicBlock *CondBB = llvm::BasicBlock::Create(*ctx->llvm(), "", TheFunction);
    llvm::BasicBlock *LoopBB = llvm::BasicBlock::Create(*ctx->llvm());
    llvm::BasicBlock *MergeBB = llvm::BasicBlock::Create(*ctx->llvm());

    IR.CreateBr(CondBB);
    IR.SetInsertPoint(CondBB);

    llvm::Value *CondV = cctx.expr->gen_Convert(ctx->bool_t(), while_stmt->condition);
    IR.CreateCondBr(CondV, LoopBB, MergeBB);

    TheFunction->insert(TheFunction->end(), LoopBB);
    IR.SetInsertPoint(LoopBB);

    while_stmt->cond_block = CondBB;
    while_stmt->end_block = MergeBB;

    if (while_stmt->body)
      cctx.stmt->gen_CodeBlock(expected_ret, while_stmt->body);

    if (!IR.GetInsertBlock()->getTerminator())
      IR.CreateBr(CondBB);

    TheFunction->insert(TheFunction->end(), MergeBB);
    IR.SetInsertPoint(MergeBB);
  }

  fun StmtCGen::gen_UnsafeStmt(types::Type *expected_ret, stmts::Stmt *now) -> void {
    auto u_stmt = now->as<stmts::UnsafeStmt>();
    auto X = u_stmt->stmt;

    if (X->is<stmts::ReturnStmt>()) {
      cctx.stmt->gen_ReturnStmt(expected_ret, X);
    }
    ef (X->is<stmts::ExprStmt>()) {
      auto expr_stmt = X->as<stmts::ExprStmt>();
      cctx.expr->gen_Expr(expr_stmt->expr);
    }
    ef (X->is<stmts::IfStmt>())    cctx.stmt->gen_IfStmt(expected_ret, X);
    ef (X->is<stmts::WhileStmt>()) cctx.stmt->gen_WhileStmt(expected_ret, X);
    ef (X->is<stmts::CodeBlock>()) cctx.stmt->gen_CodeBlock(expected_ret, X);
    ef (X->is<stmts::BreakStmt>()) {
      auto break_stmt = X->as<stmts::BreakStmt>();
      if (break_stmt->loop) {
        cctx.decl->call_destructors(break_stmt->loop->as<stmts::WhileStmt>()->body->as<stmts::CodeBlock>());
        IR.CreateBr(break_stmt->loop->as<stmts::WhileStmt>()->end_block);
      }
    }
    ef (X->is<stmts::ContinueStmt>()) {
      auto continue_stmt = X->as<stmts::ContinueStmt>();
      if (continue_stmt->loop) {
        cctx.decl->call_destructors(continue_stmt->loop->as<stmts::WhileStmt>()->body->as<stmts::CodeBlock>());
        IR.CreateBr(continue_stmt->loop->as<stmts::WhileStmt>()->cond_block);
      }
    }
    ef (X->is<stmts::UnsafeStmt>()) {
      cctx.stmt->gen_UnsafeStmt(expected_ret, X);
    }
  }

  fun StmtCGen::gen_VarStmt(stmts::Stmt *now) -> void {
    auto cvar = now->as<stmts::CodeVar>();
    if (!cvar->targetType) {
      std::cerr << "CRITICAL: targetType is null for CodeVar " << cvar->name << "\n";
      abort();
    }
    auto res   = cctx.type->gen_Type(cvar->targetType);
    cvar->llvm = IR.CreateAlloca(cvar->targetType->llvm());

    if (!cvar->has_init_expr) {
      auto target_type = cvar->targetType;
      if (target_type->is<types::StructType>()) {
        auto rec = target_type->as<types::StructType>();
        if (rec->decl) {
          auto sdecl = rec->decl->as<decls::StructDecl>();
          for (auto &C : sdecl->constructors) {
            auto cdecl = C->as<decls::ConstructorDecl>();
            auto ftype = cdecl->funcType->as<types::FuncType>();
            if (ftype->pars.size() == 1) {
              if (!cdecl->llvm) {
                types::Type *btype = cdecl->funcType;
                auto _r = cctx.type->gen_Type(btype);
                cdecl->llvm = llvm::Function::Create(
                  llvm::cast<llvm::FunctionType>(btype->llvm()),
                  llvm::GlobalValue::ExternalLinkage,
                  get_symbol_name(C), mod->llvm()
                );
                setup_function_attrs(C, cdecl->llvm);
              }
              std::vector<llvm::Value*> args;
              args.push_back(cvar->llvm);
              IR.CreateCall(llvm::cast<llvm::FunctionType>(cdecl->funcType->llvm()), cdecl->llvm, args);
              break;
            }
          }
        }
      }
    }
  }

  fun StmtCGen::gen_ReturnStmt(types::Type *expected_ret, stmts::Stmt *now) -> void {
    auto ret = now->as<stmts::ReturnStmt>();
    llvm::Value *val = nullptr;
    
    if (ret->expr)
      val = cctx.expr->gen_Convert(expected_ret, ret->expr);
    
    cctx.decl->call_destructors(nullptr);
    if (val) IR.CreateRet(val);
    else IR.CreateRetVoid();
  }

}
