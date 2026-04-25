/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Typs.hh"
#include "Exprs.hh"
#include "Basis.hh"
#include "Context.hh"
#include "Decls.hh"
#include "Diagnostic.hh"
#include "ScopeManager.hh"
#include "Types.hh"
#include <expected>
#include <iostream>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Value.h>

using namespace std;



namespace qw
{

  class pass_1
  {
    private:
      inline pass_1(qw::module *mod, vector<string> ans = {}): mod(mod), ctx(mod->ctx()), SMng(ctx, {&ctx->gst()}, {""}) {
        SMng.ans().insert(SMng.ans().end(), ans.begin(), ans.end());
      }

    private:
      qw::module *mod{};
      qw::context *ctx{};
      diagnostic::qSummary sum;
      scopeManager SMng;
    

    public:
      inline static fun pass(qw::module *mod, vector<string> ans = {}) {
        pass_1 Visiter(mod, ans);

        auto E = Visiter.check_NameSpace(mod->nameSpace());

        if (!E.has_value()) {
          Visiter.sum.add(E.error().get());
          cerr << E.error();
        }

        return std::move(Visiter.sum);
      }
      
      
    private:
      fun if_error(expected<void, uptr<diagnostic::qMessage>> msg) -> void;

    private:
      fun check_NameSpace(decls::qNameSpaceDecl*) -> expected<void, uptr<diagnostic::qMessage>>;

      fun check_FuncDecl(decls::qFuncDecl*) -> expected<void, uptr<diagnostic::qMessage>>;
      fun check_VarDecl(decls::qVarDecl*) -> expected<void, uptr<diagnostic::qMessage>>;

      fun check_CodeBlock(llvm::IRBuilder<>&, types::qFuncType*, stmts::qCodeBlock*) -> expected<void, uptr<diagnostic::qMessage>>;
      fun check_VarStmt(llvm::IRBuilder<>&, stmts::qCodeVar*) -> expected<void, uptr<diagnostic::qMessage>>;
      fun check_ReturnStmt(llvm::IRBuilder<>&, types::qFuncType*, stmts::qReturn*) -> expected<void, uptr<diagnostic::qMessage>>;

      fun check_Convert(llvm::IRBuilder<>&, types::qType*, exprs::qExpr*, word errpos) -> expected<llvm::Value*, uptr<diagnostic::qMessage>>;

      fun check_Expr(llvm::IRBuilder<>&, exprs::qExpr*&) -> expected<void, uptr<diagnostic::qMessage>>;
      fun check_NickExpr(exprs::qNickExpr*) -> expected<exprs::qExpr*, uptr<diagnostic::qMessage>>;

      fun check_Type(types::qType*&) -> expected<void, uptr<diagnostic::qMessage>>;
      fun check_RecordType(types::qRecordType*) -> expected<void, uptr<diagnostic::qMessage>>;
      fun check_NickType(types::qNickType*) -> expected<types::qType*, uptr<diagnostic::qMessage>>;
      fun check_FuncType(types::qFuncType*) -> expected<void, uptr<diagnostic::qMessage>>;

      fun check_FieldMember(types::qMemberFieldType*) -> expected<void, uptr<diagnostic::qMessage>>;
  };

}
