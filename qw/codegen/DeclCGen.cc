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
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
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



  fun DeclCGen::gen_FuncDecl(decls::Decl *now) -> void {
    auto fdecl = now->as<decls::FuncDecl>();
    types::Type *ftype = fdecl->funcType;

    auto res = cctx.type->gen_Type(ftype);

    if (!fdecl->llvm) {
      fdecl->llvm = llvm::Function::Create(
        llvm::cast<llvm::FunctionType>(ftype->llvm()),
        llvm::GlobalValue::ExternalLinkage,
        get_symbol_name(now), mod->llvm()
      );
      setup_function_attrs(now, fdecl->llvm);
    }

    if (fdecl->body) {
      auto BB = llvm::BasicBlock::Create(*ctx->llvm(), "entry", fdecl->llvm);
      IR.SetInsertPoint(BB);

      cctx.stmt->gen_CodeBlock(ftype->as<types::FuncType>()->ret, fdecl->body);
      
      if (!IR.GetInsertBlock()->getTerminator()) {
        if (
          ftype->as<types::FuncType>()->ret->is<types::PrimitiveType>() && 
          ftype->as<types::FuncType>()->ret->as<types::PrimitiveType>()->kind == types::PrimitiveEnum::Void
        ){
          IR.CreateRetVoid();
        }
        else
          IR.CreateUnreachable();
      }
    }
  }

  fun DeclCGen::gen_ConstructorDecl(decls::Decl *now) -> void {
    auto cdecl = now->as<decls::ConstructorDecl>();
    types::Type *ftype = cdecl->funcType;

    auto res = cctx.type->gen_Type(ftype);

      if (!cdecl->llvm) {
        cdecl->llvm = llvm::Function::Create(
          llvm::cast<llvm::FunctionType>(cdecl->funcType->llvm()),
          llvm::GlobalValue::ExternalLinkage,
          get_symbol_name(now), mod->llvm()
        );
        setup_function_attrs(now, cdecl->llvm);
      }

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
            
            auto val = cctx.expr->gen_Expr(init_pair.second);
            val = cctx.expr->gen_Convert(recType->vars[idx].type, init_pair.second);

            auto gep = IR.CreateStructGEP(self_ref->sub->llvm(), &*self_val, idx);
            IR.CreateStore(val, gep);
          }
        }
      }

      cctx.stmt->gen_CodeBlock(ftype_concrete->ret, cdecl->body);

      if (!IR.GetInsertBlock()->getTerminator())
        IR.CreateRetVoid();
    }
  }

  fun DeclCGen::gen_DestructorDecl(decls::Decl *now) -> void {
    auto cdecl = now->as<decls::DestructorDecl>();
    types::Type *ftype = cdecl->funcType;

    auto res = cctx.type->gen_Type(ftype);

    cdecl->llvm = llvm::Function::Create(
      llvm::cast<llvm::FunctionType>(ftype->llvm()),
      llvm::GlobalValue::ExternalLinkage,
      get_symbol_name(now), mod->llvm()
    );
    setup_function_attrs(now, cdecl->llvm);

    if (cdecl->body) {
      auto BB = llvm::BasicBlock::Create(*ctx->llvm(), "entry", cdecl->llvm);
      IR.SetInsertPoint(BB);

      auto ftype_concrete = ftype->as<types::FuncType>();
      cctx.stmt->gen_CodeBlock(ftype_concrete->ret, cdecl->body);

      if (!IR.GetInsertBlock()->getTerminator())
        IR.CreateRetVoid();
    }
  }

  fun DeclCGen::call_destructors(stmts::CodeBlock *target_block) -> void {
    for (auto it = cctx.meta->active_blocks.rbegin(); it != cctx.meta->active_blocks.rend(); ++it) {
      auto block = *it;
      
      for (auto var_it = block->vars.rbegin(); var_it != block->vars.rend(); ++var_it) {
        auto cvar = (*var_it)->as<stmts::CodeVar>();
        auto target_type = cvar->targetType;
        if (target_type->is<types::StructType>()) {
          auto rec = target_type->as<types::StructType>();
          if (rec->decl) {
            auto sdecl = rec->decl->as<decls::StructDecl>();
            for (auto &D : sdecl->destructors) {
              auto ddecl = D->as<decls::DestructorDecl>();
              auto ftype = ddecl->funcType->as<types::FuncType>();
              if (ftype->pars.size() == 1) {
                if (!ddecl->llvm) {
                  types::Type *btype = ddecl->funcType;
                  auto _r = cctx.type->gen_Type(btype);
                  ddecl->llvm = llvm::Function::Create(
                    llvm::cast<llvm::FunctionType>(btype->llvm()),
                    llvm::GlobalValue::ExternalLinkage,
                    get_symbol_name(D), mod->llvm()
                  );
                  setup_function_attrs(D, ddecl->llvm);
                }
                std::vector<llvm::Value*> args;
                args.push_back(cvar->llvm);
                IR.CreateCall(llvm::cast<llvm::FunctionType>(ddecl->funcType->llvm()), ddecl->llvm, args);
                break;
              }
            }
          }
        }
      }

      if (block == target_block) break;
    }
  }

  fun DeclCGen::gen_VarDecl(decls::Decl *now) -> void {
    auto var = now->as<decls::VarDecl>();
    if (!var->type->llvm()) {
        auto res = cctx.type->gen_Type(var->type);
        if (!res) diagnostic::fatal("Failed to generate type for global variable");
    }
    auto ty = var->type->llvm();

    llvm::Constant *init = nullptr;
    if (var->initer) {
      if (var->initer->is<exprs::IntegerLiteral>() ||
          var->initer->is<exprs::FloatingLiteral>() ||
          var->initer->is<exprs::CharLiteral>() ||
          var->initer->is<exprs::BoolLiteral>()) {
         auto val = cctx.expr->gen_Expr(var->initer);
         init = llvm::dyn_cast<llvm::Constant>(val);
      } else {
         diagnostic::fatal("Global variable initialization currently only supports simple literals!");
      }
    }

    if (!init) {
      init = llvm::Constant::getNullValue(ty);
    }

    var->llvm = new llvm::GlobalVariable(
      *mod->llvm(),
      ty,
      false,
      llvm::GlobalValue::ExternalLinkage,
      init,
      get_symbol_name(now)
    );
    setup_global_attrs(now, llvm::cast<llvm::GlobalVariable>(var->llvm));
  }

}
