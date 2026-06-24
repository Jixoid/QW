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
#include <iostream>
#include <string>
#include <vector>

#define ef else if



namespace qw
{

  class Sema
  {
    private:
      inline Sema(qw::module *mod, std::vector<std::string> ans = {})
        : mod(mod), ctx(mod->ctx()), SMng(ctx, {&ctx->gst()}, {""}) 
      {
        SMng.ans().insert(SMng.ans().end(), ans.begin(), ans.end());
      }

    private:
      qw::module *mod{};
      qw::context *ctx{};
      diagnostic::summary sum;
      scopemng SMng;

    public:
      static fun pass(qw::module *mod, std::vector<std::string> ans = {}) -> diagnostic::summary {
        Sema Visiter(mod, ans);

        auto E = Visiter.sema_NameSpace(mod->nameSpace());

        if (!E.has_value()) {
          Visiter.sum.add(E.error().get());
          std::cerr << E.error();
        }

        return std::move(Visiter.sum);
      }

    private:
      fun if_error(std::expected<void, uptr<diagnostic::message>> msg) -> void;

    private:
      // Decl
      fun sema_NameSpace(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_TypeDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_FuncDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_VarDecl(decls::Decl*) -> std::expected<void, uptr<diagnostic::message>>;

      // Stat
      fun sema_CodeBlock(stmts::Stmt*, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_VarStmt(stmts::Stmt*) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_ReturnStmt(stmts::Stmt*, types::Type *expected_ret) -> std::expected<void, uptr<diagnostic::message>>;

      // Expr
      fun sema_Convert(types::Type *target_typ, exprs::Expr *val, word errpos) -> std::expected<void, uptr<diagnostic::message>>;

      fun sema_Expr(exprs::Expr*&) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_NickExpr(exprs::Expr*) -> std::expected<exprs::Expr*, uptr<diagnostic::message>>;

      // Type
      fun sema_Type(types::Type*&, word errpos) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_RecordType(types::Type*, word errpos) -> std::expected<void, uptr<diagnostic::message>>;
      fun sema_NickType(types::Type*, word errpos) -> std::expected<types::Type*, uptr<diagnostic::message>>;
      fun sema_FuncType(types::Type*, word errpos) -> std::expected<void, uptr<diagnostic::message>>;
  };

}
