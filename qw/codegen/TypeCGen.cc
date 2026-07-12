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




  fun TypeCGen::gen_Type(types::Type *&now) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->is<types::PrimitiveType>())
      return {};

    ef (now->is<types::StructType>()) return cctx.type->gen_StructType(now);
    ef (now->is<types::IFaceType>())  return cctx.type->gen_IFaceType(now);
    ef (now->is<types::FuncType>())   return cctx.type->gen_FuncType(now);
    ef (now->is<types::PArrayType>()) {
      auto parray = now->as<types::PArrayType>();
      if_except(cctx.type->gen_Type(parray->sub));
      now->llvm() = llvm::ArrayType::get(parray->sub->llvm(), parray->size);
      return {};
    }
    ef (now->is<types::EnumType>() || now->is<types::SetType>()) {
      if (now->cgen() == StageStatus::Checked) return {};
      now->cgen() = StageStatus::Checked;

      if (now->is<types::EnumType>()) {
        auto enum_t = now->as<types::EnumType>();
        if (enum_t->decl) {
          for (auto &F: enum_t->decl->as<decls::EnumDecl>()->func) cctx.decl->gen_FuncDecl(F);
        }
      }
      return {};
    }
    ef (now->is<types::PointerType>() || now->is<types::ReferenceType>() || now->is<types::ZArrayType>()) return {};

    diagnostic::fatal(fatals::Internal_UnknownType().error()->msg());
    return {};
  }

  fun TypeCGen::gen_StructType(types::Type *now) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->cgen() == StageStatus::Checked) return {};
    now->cgen() = StageStatus::Checking;
    
    now->cgen() = StageStatus::Checking;

    auto strct = now->as<types::StructType>();
    for (auto &X: strct->vars)
      if_except(cctx.type->gen_Type(X.type));

    std::vector<llvm::Type*> LTyps;


    for (auto &v: strct->vars)
      LTyps.push_back(v.type->llvm());

    now->llvm() = llvm::StructType::create(*ctx->llvm(), LTyps, "typerec" + std::to_string(cctx.meta->m_counter_typerec++));
    now->cgen() = StageStatus::Checked;

    if (strct->decl) {
      auto old_insert_point = IR.GetInsertBlock();
      auto sdecl = strct->decl->as<decls::StructDecl>();
      for (auto &F: sdecl->func) cctx.decl->gen_FuncDecl(F);
      cctx.type->gen_VMT(now);
      for (auto &C: sdecl->constructors) cctx.decl->gen_ConstructorDecl(C);
      for (auto &D: sdecl->destructors) cctx.decl->gen_DestructorDecl(D);
      if (old_insert_point) IR.SetInsertPoint(old_insert_point);
    }

    return {};
  }

  fun TypeCGen::gen_VMT(types::Type *recType) -> void {
    auto rec = recType->as<types::StructType>();
    bool has_iface = false;

    for (auto &bt: rec->baseTypes) {
      auto resolved = bt;
      while (resolved->isReference()) resolved = resolved->as<types::ReferenceType>()->sub;
      if (resolved->is<types::IFaceType>()) {
        has_iface = true;
        break;
      }
    }

    if (!has_iface) return;

    std::string mangled = scopemng::mangle_type(recType);
    if (!mangled.empty() && mangled.front() == 'N' && mangled.back() == 'Z') {
      mangled = mangled.substr(1, mangled.size() - 2);
    }
    std::string vmt_name = "_qw_" + mangled + "@vmt";
    std::string real_vmt_name = vmt_name + "_data";

    std::vector<llvm::Constant*> header;
    std::vector<llvm::Constant*> body;
    auto i8_ptr_ty = llvm::PointerType::getUnqual(*ctx->llvm());
    auto word_ty = llvm::IntegerType::get(*ctx->llvm(), (u32)ctx->progBits() * 8);

    auto dl = mod->llvm()->getDataLayout();
    uint64_t size = dl.getTypeAllocSize(recType->llvm());
    uint64_t align = dl.getABITypeAlign(recType->llvm()).value();

    // Num bases (placeholder for now)
    header.push_back(llvm::ConstantExpr::getIntToPtr(llvm::ConstantInt::get(word_ty, 0), i8_ptr_ty)); 
    // Size
    header.push_back(llvm::ConstantExpr::getIntToPtr(llvm::ConstantInt::get(word_ty, size), i8_ptr_ty));
    // Align
    header.push_back(llvm::ConstantExpr::getIntToPtr(llvm::ConstantInt::get(word_ty, align), i8_ptr_ty));
    
    // Fini (Destructor)
    llvm::Constant *fini_ptr = llvm::ConstantPointerNull::get(i8_ptr_ty);
    if (!rec->decl->as<decls::StructDecl>()->destructors.empty()) {
        auto d_decl = rec->decl->as<decls::StructDecl>()->destructors[0]->as<decls::DestructorDecl>();
        if (d_decl->llvm) fini_ptr = d_decl->llvm;
    }
    header.push_back(fini_ptr);
    
    // Type ID (RTTI)
    header.push_back(llvm::ConstantExpr::getIntToPtr(llvm::ConstantInt::get(word_ty, 0), i8_ptr_ty)); 
    
    // Positive Offsets (Body)
    for (auto &bt: rec->baseTypes) {
      auto resolved = bt;
      while (resolved->isReference()) resolved = resolved->as<types::ReferenceType>()->sub;
      if (resolved->is<types::IFaceType>()) {
        auto iface = resolved->as<types::IFaceType>();
        if (iface->decl) {
          for (auto &iface_func: iface->decl->as<decls::IFaceDecl>()->func) {
            llvm::Function *impl_func = nullptr;
            for (auto &rec_func: rec->decl->as<decls::StructDecl>()->func) {
              if (rec_func->name() == iface_func->name()) {
                impl_func = rec_func->as<decls::FuncDecl>()->llvm;
                break;
              }
            }
            if (impl_func) body.push_back(impl_func);
            else body.push_back(llvm::ConstantPointerNull::get(i8_ptr_ty));
          }
        }
      }
    }

    std::vector<llvm::Constant*> all_entries;
    all_entries.insert(all_entries.end(), header.begin(), header.end());
    all_entries.insert(all_entries.end(), body.begin(), body.end());

    llvm::ArrayType *vmt_type = llvm::ArrayType::get(i8_ptr_ty, all_entries.size());
    llvm::Constant *vmt_init = llvm::ConstantArray::get(vmt_type, all_entries);
    
    auto global_vmt = new llvm::GlobalVariable(*mod->llvm(), vmt_type, true, llvm::GlobalValue::ExternalLinkage, vmt_init, real_vmt_name);
    
    // Create an alias pointing to the anchor (index 0 of body)
    // Removed because ORC JIT fails to correctly infer the size of the alias, causing execution errors.
    // cctx.expr->gen_Convert now directly applies the GEP to vmt_data.
  }

  fun TypeCGen::gen_FuncType(types::Type *now) -> std::expected<void, uptr<diagnostic::message>> {
    auto ftype = now->as<types::FuncType>();
    for (auto &X: ftype->pars)
      if_except(cctx.type->gen_Type(X.type));
    
    if_except(cctx.type->gen_Type(ftype->ret));

    std::vector<llvm::Type *> llvm_params;
    llvm_params.reserve(ftype->pars.size());
    for (auto [s, t, v]: ftype->pars) {
      llvm_params.push_back(t->llvm());
    }
    now->llvm() = llvm::FunctionType::get(ftype->ret->llvm(), llvm_params, false);

    return {};
  }

  fun TypeCGen::gen_IFaceType(types::Type *now) -> std::expected<void, uptr<diagnostic::message>> {
    if (now->cgen() == StageStatus::Checked) return {};
    now->cgen() = StageStatus::Checking;
    
    auto ptr_ty = llvm::PointerType::getUnqual(*ctx->llvm());
    now->llvm() = llvm::StructType::get(*ctx->llvm(), {ptr_ty, ptr_ty});
    now->cgen() = StageStatus::Checked;

    return {};
  }

}
