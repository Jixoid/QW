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



  fun CodeGen::gen_NameSpace(decls::Decl *now) -> void {
    SMng.ans().push_back(scopemng::mangling_abi_qw(now));

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      if (X->is<decls::NameSpaceDecl>()) gen_NameSpace(X);
      ef (X->is<decls::FuncDecl>())      cctx.decl->gen_FuncDecl(X);
      ef (X->is<decls::VarDecl>())       cctx.decl->gen_VarDecl(X);
      ef (X->is<decls::TypeDecl>()) {
        if (static_cast<decls::Decl*>(X)->is_generic()) continue;
        auto tdecl = X->as<decls::TypeDecl>();
        if (tdecl->type && (tdecl->type->is<types::StructType>() || tdecl->type->is<types::EnumType>())) {
          auto ret = cctx.type->gen_Type(tdecl->type);
        }
      }
    }

    SMng.ans().pop_back();
  }

  fun CodeGen::gen_GlobalCtorsDtors() -> void {
    auto emit_array = [&](bool is_ctor, const char* array_name) {
      std::vector<decls::FuncDecl*> list;
      for (auto m : ctx->modules()) {
        auto& mod_list = is_ctor ? m->global_ctors() : m->global_dtors();
        for (auto fdecl : mod_list) {
          if (fdecl->llvm && fdecl->llvm->getParent() == mod->llvm()) {
            list.push_back(fdecl);
          }
        }
      }

      if (list.empty()) return;

      auto i32_t = llvm::Type::getInt32Ty(*ctx->llvm());
      auto ptr_t = llvm::PointerType::getUnqual(*ctx->llvm());
      std::vector<llvm::Type*> struct_types = { i32_t, ptr_t, ptr_t };
      auto struct_t = llvm::StructType::get(*ctx->llvm(), struct_types);

      std::vector<llvm::Constant*> init_list;
      for (auto fdecl : list) {
        auto priority = llvm::ConstantInt::get(i32_t, 65535);
        auto null_ptr = llvm::ConstantPointerNull::get(llvm::PointerType::getUnqual(*ctx->llvm()));
        
        std::vector<llvm::Constant*> fields = { priority, llvm::cast<llvm::Constant>(fdecl->llvm), null_ptr };
        auto struct_val = llvm::ConstantStruct::get(struct_t, fields);
        init_list.push_back(struct_val);
      }

      auto array_t = llvm::ArrayType::get(struct_t, init_list.size());
      auto array_val = llvm::ConstantArray::get(array_t, init_list);

      new llvm::GlobalVariable(
        *mod->llvm(),
        array_t,
        false,
        llvm::GlobalValue::AppendingLinkage,
        array_val,
        array_name
      );
    };

    emit_array(true, "llvm.global_ctors");
    emit_array(false, "llvm.global_dtors");
  }

}
