/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "qw/basis.hh"
#include "qw/pretype.hh"
#include "qw/control/context.hh"
#include "qw/control/scopemng.hh"
#include "qw/tree/types.hh"
#include "qw/tree/exprs.hh"
#include "qw/tree/decls.hh"
#include "qw/diagnostic/diagnostic.hh"
#include <expected>
#include <iostream>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Value.h>



namespace qw
{

  class check
  {
    private:
      inline check(qw::module *mod, std::vector<std::string> ans = {}): mod(mod), ctx(mod->ctx()), SMng(ctx, {&ctx->gst()}, {""}) {
        SMng.ans().insert(SMng.ans().end(), ans.begin(), ans.end());
      }

    private:
      qw::module *mod{};
      qw::context *ctx{};
      diagnostic::qSummary sum;
      scopemng SMng;
    

    public:
      inline static fun pass(qw::module *mod, std::vector<std::string> ans = {}) {
        check Visiter(mod, ans);

        auto E = Visiter.check_NameSpace(mod->nameSpace());

        if (!E.has_value()) {
          Visiter.sum.add(E.error().get());
          std::cerr << E.error();
        }

        return std::move(Visiter.sum);
      }
      
      
    private:
      fun if_error(std::expected<void, uptr<diagnostic::qMessage>> msg) -> void;

    private:
      fun check_NameSpace(decls::NameSpace*) -> std::expected<void, uptr<diagnostic::qMessage>>;

      fun check_FuncDecl(decls::Func*) -> std::expected<void, uptr<diagnostic::qMessage>>;
      fun check_VarDecl(decls::Var*) -> std::expected<void, uptr<diagnostic::qMessage>>;

      fun check_CodeBlock(llvm::IRBuilder<>&, types::Func*, stmts::CodeBlock*) -> std::expected<void, uptr<diagnostic::qMessage>>;
      fun check_VarStmt(llvm::IRBuilder<>&, stmts::CodeVar*) -> std::expected<void, uptr<diagnostic::qMessage>>;
      fun check_ReturnStmt(llvm::IRBuilder<>&, types::Func*, stmts::Return*) -> std::expected<void, uptr<diagnostic::qMessage>>;

      fun check_Convert(llvm::IRBuilder<>&, types::Type*, exprs::Expr*, word errpos) -> std::expected<llvm::Value*, uptr<diagnostic::qMessage>>;

      fun check_Expr(llvm::IRBuilder<>&, exprs::Expr*&) -> std::expected<void, uptr<diagnostic::qMessage>>;
      fun check_NickExpr(exprs::NickExpr*) -> std::expected<exprs::Expr*, uptr<diagnostic::qMessage>>;

      fun check_Type(types::Type*&) -> std::expected<void, uptr<diagnostic::qMessage>>;
      fun check_RecordType(types::Record*) -> std::expected<void, uptr<diagnostic::qMessage>>;
      fun check_NickType(types::Nick*) -> std::expected<types::Type*, uptr<diagnostic::qMessage>>;
      fun check_FuncType(types::Func*) -> std::expected<void, uptr<diagnostic::qMessage>>;

      fun check_FieldMember(types::MemberField*) -> std::expected<void, uptr<diagnostic::qMessage>>;
  };

}
