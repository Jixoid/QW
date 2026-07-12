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



  fun ExprCGen::gen_Convert(types::Type *typ, exprs::Expr *val) -> llvm::Value*  {
    llvm::Value *src  = cctx.expr->gen_Expr(val);
    types::Type *styp = val->targetType();

    re:
    if (typ == styp) return src;

    ef (styp->isReference()) {
      styp = styp->as<types::ReferenceType>()->sub;
      src  = IR.CreateLoad(styp->llvm(), src);
      goto re;
    }

    ef (typ->isInteger() && styp->isInteger()) { return IR.CreateIntCast(src, typ->llvm(), typ->isSigned() && styp->isSigned()); }
    ef (typ->is<types::EnumType>() && styp->isInteger()) { return IR.CreateIntCast(src, typ->llvm(), false); }
    ef (typ->isInteger() && styp->is<types::EnumType>()) { return IR.CreateIntCast(src, typ->llvm(), typ->isSigned()); }
    ef (typ->is<types::EnumType>() && styp->is<types::EnumType>()) { return IR.CreateIntCast(src, typ->llvm(), false); }
    ef (typ->isChar() && styp->isInteger()) { return IR.CreateIntCast(src, typ->llvm(), false); }
    ef (typ->isInteger() && styp->isChar()) { return IR.CreateIntCast(src, typ->llvm(), false); }
    ef ((typ->isPointer() || typ == ctx->ptr_t() || typ == ctx->null_t()) && styp->isInteger()) { return IR.CreateIntToPtr(src, typ->llvm()); }
    ef (typ->isInteger() && (styp->isPointer() || styp == ctx->ptr_t() || styp == ctx->null_t())) { return IR.CreatePtrToInt(src, typ->llvm()); }
    ef ((typ->isPointer() || typ == ctx->ptr_t() || typ == ctx->null_t()) && (styp->isPointer() || styp == ctx->ptr_t() || styp == ctx->null_t())) { return IR.CreatePointerCast(src, typ->llvm()); }

    ef (typ->is<types::IFaceType>() && styp->is<types::StructType>()) {
      auto iface_type = typ->as<types::IFaceType>();
      auto ptr_ty = llvm::PointerType::getUnqual(*ctx->llvm());
      
      std::string mangled = scopemng::mangle_type(styp);
      if (!mangled.empty() && mangled.front() == 'N' && mangled.back() == 'Z') {
        mangled = mangled.substr(1, mangled.size() - 2);
      }
      std::string vmt_name = "_qw_" + mangled + "@vmt_data";
      auto vmt_gv = mod->llvm()->getNamedGlobal(vmt_name);
      llvm::Value *vmt_ptr = llvm::ConstantPointerNull::get(ptr_ty);
      if (vmt_gv) {
          size_t header_size = 5;
          auto anchor_idx = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), header_size);
          auto zero = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), 0);
          std::vector<llvm::Value*> indices = {zero, anchor_idx};
          vmt_ptr = IR.CreateInBoundsGEP(vmt_gv->getValueType(), vmt_gv, indices);
      }
      
      llvm::Value *struct_ptr = IR.CreateAlloca(styp->llvm());
      IR.CreateStore(src, struct_ptr);
      
      llvm::Value *fat_ptr = llvm::UndefValue::get(typ->llvm());
      fat_ptr = IR.CreateInsertValue(fat_ptr, struct_ptr, 0);
      fat_ptr = IR.CreateInsertValue(fat_ptr, vmt_ptr, 1);
      
      return fat_ptr;
    }
    
    ef (typ->is<types::IFaceType>() && styp->is<types::IFaceType>()) {
      auto target_iface_type = typ->as<types::IFaceType>();
      auto ptr_ty = llvm::PointerType::getUnqual(*ctx->llvm());
      llvm::Value *data_ptr = IR.CreateExtractValue(src, 0);
      llvm::Value *old_vmt = IR.CreateExtractValue(src, 1);
      
      llvm::Value *fat_ptr = llvm::UndefValue::get(typ->llvm());
      fat_ptr = IR.CreateInsertValue(fat_ptr, data_ptr, 0);
      fat_ptr = IR.CreateInsertValue(fat_ptr, old_vmt, 1);
      
      return fat_ptr;
    }

    return src;
  }

  fun ExprCGen::gen_Expr(exprs::Expr *now) -> llvm::Value* {
    if (now->is<exprs::IntegerLiteral>()) {
      auto lit       = now->as<exprs::IntegerLiteral>();
      types::Type *t = now->targetType() ? now->targetType() : ctx->intS32_t();

      if (std::holds_alternative<u64>(lit->val))
        return llvm::ConstantInt::get(t->llvm(), std::get<u64>(lit->val), false);
      else
        return llvm::ConstantInt::get(t->llvm(), std::get<i64>(lit->val), true);
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

      if (vexpr->var->type() == IdentyEnum::Stmt) {
        auto cvar  = static_cast<stmts::Stmt*>(vexpr->var)->as<stmts::CodeVar>();
        if (cvar->targetType && cvar->targetType->isReference())
          return IR.CreateLoad(llvm::PointerType::getUnqual(*ctx->llvm()), cvar->llvm);
        
        return cvar->llvm;
      }
      ef (vexpr->var->type() == IdentyEnum::Decl) {
        auto vdecl = static_cast<decls::Decl*>(vexpr->var)->as<decls::VarDecl>();
        return vdecl->llvm;
      }
      return nullptr;
    }
    ef (now->is<exprs::ValExpr>()) {
      return now->llvm();
    }
    ef (now->is<exprs::UnaryOp>()) {
      auto U = now->as<exprs::UnaryOp>();
      if (U->kind == exprs::UnaryOpEnum::AddrOf) {
        return cctx.expr->gen_Expr(U->o1);
      }
      ef (U->kind == exprs::UnaryOpEnum::Minus) {
        auto v = cctx.expr->gen_Convert(now->targetType(), U->o1);
        return now->targetType()->isFloat() ? IR.CreateFNeg(v) : IR.CreateNeg(v);
      }
      ef (U->kind == exprs::UnaryOpEnum::Plus) {
        return cctx.expr->gen_Convert(now->targetType(), U->o1);
      }
      ef (U->kind == exprs::UnaryOpEnum::LNot) {
        auto v = cctx.expr->gen_Convert(now->targetType(), U->o1);
        return IR.CreateNot(v);
      }
      ef (U->kind == exprs::UnaryOpEnum::BitNot) {
        auto v = cctx.expr->gen_Convert(now->targetType(), U->o1);
        return IR.CreateNot(v);
      }
      return nullptr;
    }
    ef (now->is<exprs::BinaryOp>()) {
      auto C = now->as<exprs::BinaryOp>();

      bool is_compound_assign = (C->kind >= exprs::BinaryOpEnum::AddAssign && C->kind <= exprs::BinaryOpEnum::ShrAssign);

      if (C->kind == exprs::BinaryOpEnum::Assign || is_compound_assign) {
        llvm::Value *ptr = cctx.expr->gen_Expr(C->o1);

        auto lhs_type = C->o1->targetType();
        while (lhs_type->isReference()) {
          lhs_type = lhs_type->as<types::ReferenceType>()->sub;
        }

        llvm::Value *val2 = cctx.expr->gen_Convert(lhs_type, C->o2);
        
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
      llvm::Value *v1 = cctx.expr->gen_Convert(computationType, C->o1);
      llvm::Value *v2 = cctx.expr->gen_Convert(computationType, C->o2);

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
        return cctx.expr->gen_Convert(ptrType, M->obj);
      }
      ef (M->kind == exprs::PostfixOpEnum::Call) {
        llvm::Function *calleeFn = nullptr;
        
        if (M->obj->is<exprs::MemberOp>()) {
          auto memOp    = M->obj->as<exprs::MemberOp>();
          auto obj_type = memOp->obj->targetType();
          while (obj_type->isReference()) {
            obj_type = obj_type->as<types::ReferenceType>()->sub;
          }
          if (obj_type->is<types::StructType>()) {
            auto rec_type    = obj_type->as<types::StructType>();
            auto member_name = memOp->mem->as<exprs::NickExpr>()->unresolved[0];

            std::vector<types::Type *> arg_types;
            for (auto op : M->operands)
              arg_types.push_back(op->targetType());

            if (rec_type->decl) {
              for (auto &F : rec_type->decl->as<decls::StructDecl>()->func) {
                if (F->name() == member_name) {
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
                      
                    if (match) {
                      calleeFn = F->as<decls::FuncDecl>()->llvm;
                      break;
                    }
                  }
                }
              }
            }
          }
          ef (obj_type->is<types::IFaceType>()) {
            auto iface_type = obj_type->as<types::IFaceType>();
            auto member_name = memOp->mem->as<exprs::NickExpr>()->unresolved[0];
            
            u32 method_index = 0;
            for (size_t i = 0; i < iface_type->decl->as<decls::IFaceDecl>()->func.size(); ++i) {
              if (iface_type->decl->as<decls::IFaceDecl>()->func[i]->name() == member_name) {
                method_index = i;
                break;
              }
            }
            
            llvm::Value *obj_ptr = cctx.expr->gen_Expr(memOp->obj); // This is pointer to fat pointer {ptr, ptr}
            
            llvm::Type *ptr_ty = llvm::PointerType::getUnqual(*ctx->llvm());
            
            llvm::Value *data_ptr_ptr = IR.CreateStructGEP(obj_type->llvm(), obj_ptr, 0);
            llvm::Value *data_ptr = IR.CreateLoad(ptr_ty, data_ptr_ptr);
            
            llvm::Value *vmt_ptr_ptr = IR.CreateStructGEP(obj_type->llvm(), obj_ptr, 1);
            llvm::Value *vmt_ptr = IR.CreateLoad(ptr_ty, vmt_ptr_ptr);
            
            llvm::Value *idx  = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), method_index);
            llvm::Value *method_ptr_ptr = IR.CreateInBoundsGEP(ptr_ty, vmt_ptr, idx);
            llvm::Value *method_ptr = IR.CreateLoad(ptr_ty, method_ptr_ptr);
            
            auto ftype = M->obj->targetType()->as<types::FuncType>();
            auto _r = cctx.type->gen_Type(M->obj->targetType());
            std::vector<llvm::Value *> args;
            
            if (M->operands.size() > 0) {
              args.push_back(data_ptr); // The 'self' parameter is data_ptr, not the fat pointer
              for (size_t i = 1; i < M->operands.size(); ++i) {
                args.push_back(cctx.expr->gen_Convert(ftype->pars[i].type, M->operands[i]));
              }
            }
            
            return IR.CreateCall(llvm::cast<llvm::FunctionType>(M->obj->targetType()->llvm()), method_ptr, args);
          }
        }
        ef (M->obj->is<exprs::NickExpr>()) {
          auto nick = M->obj->as<exprs::NickExpr>();
          
          std::vector<types::Type*> arg_types;
          for (auto op : M->operands) arg_types.push_back(op->targetType());
          auto ret  = SMng.lookup(now->parent(), nick->unresolved, &arg_types);
          if (!ret) {
            ret = SMng.lookup(now->parent(), nick->unresolved, nullptr);
          }
          if (ret && ret->type() == IdentyEnum::Decl && static_cast<decls::Decl *>(ret)->is<decls::FuncDecl>()) {
            auto ret_decl = static_cast<decls::Decl *>(ret);
            if (ret_decl->name() == "syscall" && ret_decl->parent() == ctx->sys_api.sys_ns) {
              std::vector<llvm::Value *> args;
              auto ftype = M->obj->targetType()->as<types::FuncType>();
              for (size_t i = 0; i < M->operands.size(); ++i) {
                args.push_back(cctx.expr->gen_Convert(ftype->pars[i].type, M->operands[i]));
              }
              
              llvm::Type *int64Ty = llvm::Type::getInt64Ty(*ctx->llvm());
              std::vector<llvm::Type*> argTys(args.size(), int64Ty);
              auto fTy = llvm::FunctionType::get(int64Ty, argTys, false);
              
              std::string constraints = "={ax},{ax}";
              if (args.size() > 1) constraints += ",{di}";
              if (args.size() > 2) constraints += ",{si}";
              if (args.size() > 3) constraints += ",{dx}";
              if (args.size() > 4) constraints += ",{r10}";
              if (args.size() > 5) constraints += ",{r8}";
              if (args.size() > 6) constraints += ",{r9}";
              constraints += ",~{rcx},~{r11},~{memory},~{dirflag},~{fpsr},~{flags}";
              
              auto inlineAsm = llvm::InlineAsm::get(fTy, "syscall", constraints, true);
              return IR.CreateCall(fTy, inlineAsm, args);
            }
            
            auto fdecl = ret_decl->as<decls::FuncDecl>();
            if (!fdecl->llvm) {
                auto _r = cctx.type->gen_Type(fdecl->funcType);
                auto ret_decl = static_cast<decls::Decl *>(ret);
                fdecl->llvm = llvm::Function::Create(
                  llvm::cast<llvm::FunctionType>(fdecl->funcType->llvm()), llvm::GlobalValue::ExternalLinkage, get_symbol_name(ret_decl), mod->llvm()
                );
                setup_function_attrs(ret_decl, fdecl->llvm);
            }
            calleeFn = fdecl->llvm;
          }
        }

        if (calleeFn) {
          std::vector<llvm::Value *> args;
          auto ftype = M->obj->targetType()->as<types::FuncType>();
          for (size_t i = 0; i < M->operands.size(); ++i) {
            args.push_back(cctx.expr->gen_Convert(ftype->pars[i].type, M->operands[i]));
          }
          return IR.CreateCall(calleeFn, args);
        }

        llvm::Value *calleeVal = cctx.expr->gen_Expr(M->obj);
        if (!calleeVal) diagnostic::fatal("CodeGen: calleeVal is null for " + std::string(M->obj->targetType()->typname()));
        auto ftype             = M->obj->targetType()->as<types::FuncType>();
        std::vector<llvm::Value *> args;
        for (size_t i = 0; i < M->operands.size(); ++i) {
          args.push_back(cctx.expr->gen_Convert(ftype->pars[i].type, M->operands[i]));
        }
        return IR.CreateCall(llvm::cast<llvm::FunctionType>(M->obj->targetType()->llvm()), calleeVal, args);
      }
      ef (M->kind == exprs::PostfixOpEnum::Array) {
        llvm::Value *obj_ptr = cctx.expr->gen_Expr(M->obj);
        auto obj_type        = M->obj->targetType();

        while (obj_type->isReference()) {
          obj_type = obj_type->as<types::ReferenceType>()->sub;
        }

        llvm::Value *idx = cctx.expr->gen_Expr(M->operands[0]);

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
          diagnostic::fatal("CodeGen: Unknown Expr Type! (MemberOp)");
          return nullptr;
        }
      }
      else {
        diagnostic::fatal("CodeGen: Unknown Expr Type! (PostfixOp Array)");
        return nullptr;
      }
    }
    ef (now->is<exprs::MemberOp>()) {
      auto M = now->as<exprs::MemberOp>();

      llvm::Value *obj_ptr = cctx.expr->gen_Expr(M->obj);
      auto obj_type        = M->obj->targetType();

      while (obj_type->isReference()) {
        obj_type = obj_type->as<types::ReferenceType>()->sub;
      }

      auto ret = cctx.type->gen_Type(obj_type);

      auto rec_type   = obj_type->as<types::StructType>();
      auto field_name = M->mem->as<exprs::NickExpr>()->unresolved[0];

      u32 index{};
      for (size_t i = 0; i < rec_type->vars.size(); ++i) {
        if (rec_type->vars[i].name == field_name) {
          index = i;
          break;
        }
      }

      bool has_iface = false;
      for (auto &bt: rec_type->baseTypes) {
        auto resolved = bt;
        while (resolved->isReference()) resolved = resolved->as<types::ReferenceType>()->sub;
        if (resolved->is<types::IFaceType>()) {
          has_iface = true;
          break;
        }
      }
      
      if (has_iface) index++;

      llvm::Value *zero = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), 0);
      llvm::Value *idx  = llvm::ConstantInt::get(llvm::Type::getInt32Ty(*ctx->llvm()), index);

      return IR.CreateInBoundsGEP(obj_type->llvm(), obj_ptr, {zero, idx});
    }
    else {
      if (now->is<exprs::NickExpr>()) {
        auto nick = now->as<exprs::NickExpr>();
        std::string name = nick->unresolved[0];
        for (size_t i = 1; i < nick->unresolved.size(); i++) name += "::" + nick->unresolved[i];
        diagnostic::fatal("CodeGen: Unknown Expr Type! NickExpr: " + name);
      }
      else if (now->is<exprs::GenericOp>()) diagnostic::fatal("CodeGen: Unknown Expr Type! GenericOp");
      else if (now->is<exprs::StringLiteral>()) diagnostic::fatal("CodeGen: Unknown Expr Type! StringLiteral");
      else if (now->is<exprs::IntegerLiteral>()) diagnostic::fatal("CodeGen: Unknown Expr Type! IntegerLiteral");
      else if (now->is<exprs::FloatingLiteral>()) diagnostic::fatal("CodeGen: Unknown Expr Type! FloatingLiteral");
      else if (now->is<exprs::CharLiteral>()) diagnostic::fatal("CodeGen: Unknown Expr Type! CharLiteral");
      else if (now->is<exprs::BoolLiteral>()) diagnostic::fatal("CodeGen: Unknown Expr Type! BoolLiteral");
      else if (now->is<exprs::PtrLiteral>()) diagnostic::fatal("CodeGen: Unknown Expr Type! PtrLiteral");
      else diagnostic::fatal("CodeGen: Unknown Expr Type! Something Else");
    }
  }
  
}
