/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "qw/basis.hh"
#include "qw/control/context.hh"
#include "qw/control/scopemng.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/stmts.hh"
#include "qw/tree/types.hh"
#include <expected>
#include <string>
#include <vector>

#include <llvm/IR/BasicBlock.h>
#include <llvm/IR/Function.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>

#define ef else if



namespace qw
{

  struct MetaCGen;
  struct TypeCGen;
  struct DeclCGen;
  struct ExprCGen;
  struct StmtCGen;

  struct CGContext {
    public:
      CGContext(qw::module *mod, scopemng &SMng, llvm::IRBuilder<> &IR)
        : mod(mod), ctx(mod->ctx()), SMng(SMng), IR(IR)
      {}

    public:
      qw::module *mod{};
      qw::context *ctx{};
      scopemng &SMng;
      llvm::IRBuilder<> &IR;

    public:
      MetaCGen *meta{};
      TypeCGen *type{};
      DeclCGen *decl{};
      ExprCGen *expr{};
      StmtCGen *stmt{};
  };


  struct SubCGen {
    public:
      SubCGen(CGContext &cctx)
        : cctx(cctx)
        , mod(cctx.mod)
        , ctx(cctx.ctx)
        , SMng(cctx.SMng)
        , IR(cctx.IR)
      {}

    protected:
      CGContext &cctx;
      qw::module *mod{};
      qw::context *ctx{};
      scopemng &SMng;
      llvm::IRBuilder<> &IR;
  };


  struct MetaCGen: SubCGen {
    public:
      MetaCGen(CGContext &cctx): SubCGen(cctx) {}

    public:
      u64 m_counter_typerec{};
      std::vector<stmts::CodeBlock*> active_blocks;
  };

  struct TypeCGen: SubCGen {
    public:
      TypeCGen(CGContext &cctx): SubCGen(cctx) {}

    public:
      fun gen_Type(types::Type*&) -> std::expected<void, uptr<diagnostic::message>>;
      fun gen_StructType(types::Type*) -> std::expected<void, uptr<diagnostic::message>>;
      fun gen_IFaceType(types::Type*) -> std::expected<void, uptr<diagnostic::message>>;
      fun gen_FuncType(types::Type*) -> std::expected<void, uptr<diagnostic::message>>;
      
      fun gen_VMT(types::Type *recType) -> void;
  };

  struct DeclCGen: SubCGen {
    public:
      DeclCGen(CGContext &cctx): SubCGen(cctx) {}

    public:
      fun gen_FuncDecl(decls::Decl *now) -> void;
      fun gen_ConstructorDecl(decls::Decl *now) -> void;
      fun gen_DestructorDecl(decls::Decl *now) -> void;
      fun gen_VarDecl(decls::Decl *now) -> void;

      fun call_destructors(stmts::CodeBlock *target_block = nullptr) -> void;
  };

  struct ExprCGen: SubCGen {
    public:
      ExprCGen(CGContext &cctx): SubCGen(cctx) {}

    public:
      fun gen_Convert(types::Type *target_typ, exprs::Expr *val) -> llvm::Value*;
      fun gen_Expr(exprs::Expr *now) -> llvm::Value*;
  };

  struct StmtCGen: SubCGen {
    public:
      StmtCGen(CGContext &cctx): SubCGen(cctx) {}

    public:
      fun gen_CodeBlock(types::Type *expected_ret, stmts::Stmt *now) -> void;
      fun gen_IfStmt(types::Type *expected_ret, stmts::Stmt *now) -> void;
      fun gen_WhileStmt(types::Type *expected_ret, stmts::Stmt *now) -> void;
      fun gen_VarStmt(stmts::Stmt *now) -> void;
      fun gen_ReturnStmt(types::Type *expected_ret, stmts::Stmt *now) -> void;
      fun gen_UnsafeStmt(types::Type *expected_ret, stmts::Stmt *now) -> void;
  };



  class CodeGen
  {
    public:
      inline CodeGen(qw::module *mod, std::vector<std::string> ans = {})
        : mod(mod)
        , ctx(mod->ctx())
        , SMng(ctx, { &ctx->gst() }, { "" })
        , IR(*ctx->llvm())
        , cctx(mod, SMng, IR)
        , meta(cctx)
        , type(cctx)
        , decl(cctx)
        , expr(cctx)
        , stmt(cctx)
      {
        SMng.ans().insert(SMng.ans().end(), ans.begin(), ans.end());
        cctx.meta = &meta;
        cctx.type = &type;
        cctx.decl = &decl;
        cctx.expr = &expr;
        cctx.stmt = &stmt;
      }

    protected:
      qw::module *mod{};
      qw::context *ctx{};
      scopemng SMng;
      llvm::IRBuilder<> IR;
      CGContext cctx;

    protected:
      MetaCGen meta;
      TypeCGen type;
      DeclCGen decl;
      ExprCGen expr;
      StmtCGen stmt;

    public:
      static fun pass(qw::module *mod, std::vector<std::string> ans = {}) -> void {
        CodeGen CG(mod, ans);
        CG.gen_NameSpace(mod->nameSpace());
      }

      static fun pass_ns(qw::module *mod, decls::Decl *ns, std::vector<std::string> ans = {}) -> void {
        for (auto decl : ns->as<decls::NameSpaceDecl>()->decls) {
          if (decl->is<decls::FuncDecl>()) decl->as<decls::FuncDecl>()->llvm = nullptr;
          ef (decl->is<decls::NameSpaceDecl>()) {
            for (auto d : decl->as<decls::NameSpaceDecl>()->decls) {
              if (d->is<decls::FuncDecl>()) d->as<decls::FuncDecl>()->llvm = nullptr;
            }
          }
        }
        CodeGen CG(mod, ans);
        CG.gen_NameSpace(ns);
      }

    public:
      fun gen_NameSpace(decls::Decl *now) -> void;
  };

}
