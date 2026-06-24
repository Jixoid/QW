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

  class CodeGen
  {
    private:
      CodeGen(qw::module *mod, std::vector<std::string> ans = {})
        : mod(mod)
        , ctx(mod->ctx())
        , SMng(ctx, { &ctx->gst() }, { "" })
        , IR(*ctx->llvm())
      {
        SMng.ans().insert(SMng.ans().end(), ans.begin(), ans.end());
      }

    private:
      qw::module *mod{};
      qw::context *ctx{};
      scopemng SMng;

      llvm::IRBuilder<> IR;

    public:
      static fun pass(qw::module *mod, std::vector<std::string> ans = {}) -> void {
        CodeGen Generator(mod, ans);
        Generator.gen_NameSpace(mod->nameSpace());
      }

      u64 m_counter_typerec{};

    private:
      // Decl
      fun gen_NameSpace(decls::Decl *now) -> void;
      fun gen_FuncDecl(decls::Decl *now) -> void;
      fun gen_VarDecl(decls::Decl *now) -> void;

      // Stat
      fun gen_CodeBlock(types::Type *expected_ret, stmts::Stmt *now) -> void;
      fun gen_VarStmt(stmts::Stmt *now) -> void;
      fun gen_ReturnStmt(types::Type *expected_ret, stmts::Stmt *now) -> void;

      // Expr
      fun gen_Convert(types::Type *target_typ, exprs::Expr *val) -> llvm::Value*;

      fun gen_Expr(exprs::Expr *now) -> llvm::Value*;

      // Type
      fun gen_Type(types::Type*&) -> std::expected<void, uptr<diagnostic::message>>;
      fun gen_RecordType(types::Type*) -> std::expected<void, uptr<diagnostic::message>>;
      fun gen_FuncType(types::Type*) -> std::expected<void, uptr<diagnostic::message>>;
  };

}
