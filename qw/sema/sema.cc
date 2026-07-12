/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "qw/sema/sema.hh"
#include "qw/basis.hh"
#include "qw/control/scopemng.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/msgs.hh"
#include "qw/pretype.hh"
#include "qw/tree/decls.hh"
#include <cassert>
#include <expected>
#include <memory>
#include <string>
#include <vector>

#define if_error(X)                                                                                                                                  \
  {                                                                                                                                                  \
    auto E = X;                                                                                                                                      \
    if (!E.has_value()) {                                                                                                                            \
      if (E.error()->type() == qw::diagnostic::MsgType::Fatal)                                                                                       \
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

  fun Sema::sema_NameSpace(decls::Decl *now) -> std::expected<void, uptr<diagnostic::message>> {
    SMng.ans().push_back(scopemng::mangling_abi_qw(now));

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      auto name = scopemng::mangling_abi_qw(X);
      auto existing = ctx->gst().find(name);
      if (!existing.has_value())
        ctx->gst().add_ident(name, X);
    }

    for (auto &X : now->as<decls::NameSpaceDecl>()->decls) {
      if_except(sctx.decl->sema_Attributes(X));
      if (X->is<decls::NameSpaceDecl>()) if_error(sema_NameSpace(X))
      ef (X->is<decls::FuncDecl>()) if_error(sctx.decl->sema_FuncDecl(X))
      ef (X->is<decls::TypeDecl>()) if_error(sctx.decl->sema_TypeDecl(X))
      ef (X->is<decls::VarDecl>()) if_error(sctx.decl->sema_VarDecl(X))
      ef (X->is<decls::AliasDecl>()) {}
      else
        diagnostic::fatal(fatals::Internal_UnknownDecl().error()->msg());
    }

    SMng.ans().pop_back();
    return {};
  }

}
