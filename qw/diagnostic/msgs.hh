/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "qw/basis.hh"
#include "qw/diagnostic/diagnostic.hh"
#include "qw/diagnostic/i18n.hh"
#include <expected>

#define ef else if

#define REPN_0(m)
#define REPN_1(m) , m##1
#define REPN_2(m) REPN_1(m), m##2
#define REPN_3(m) REPN_2(m), m##3

#define REPN(n, m) REPN_##n(m)

#define REP_0(m)
#define REP_1(m) m##1,
#define REP_2(m) REP_1(m) m##2,
#define REP_3(m) REP_2(m) m##3,

#define REP(n, m) REP_##n(m)

#define NewMsg(Name, Type, Msg, N)                                                                                                                   \
  inline fun Name(word w REPN(N, std::string p)) { return std::unexpected(diagnostic::Type ::make(w, Msg, { REP(N, p) })); }

#define NewMsgF(Name, Msg, N)                                                                                                                        \
  inline fun Name(REPN(N, std::string p)) { return std::unexpected(diagnostic::fatal::make(Msg, { REP(N, p) })); }



namespace qw::fatals
{
  NewMsgF(FileEndedButContextNotFinished, _("file ended but context not finished."), 0);

  // Internal
  NewMsgF(Internal_UnknownDecl, _("<red>internal<gray>:<reset> unknown decl."), 0);
  NewMsgF(Internal_UnknownType, _("<red>internal<gray>:<reset> unknown type."), 0);
  NewMsgF(Internal_UnknownStmt, _("<red>internal<gray>:<reset> unknown stmt."), 0);
  NewMsgF(Internal_UnknownExpr, _("<red>internal<gray>:<reset> unknown expr."), 0);
}

namespace qw::errors
{
  NewMsg(ExpectedIdentifierBut, error, _("expected ‘<blue>{1}<reset>‘, but found ‘<blue>{0}<reset>‘."), 2);
  NewMsg(ExpectedIdentifierBut2, error, _("expected ‘<blue>{1}<reset>‘ or ‘<blue>{2}<reset>‘, but found ‘<blue>{0}<reset>‘."), 3);
  NewMsg(ExpectedAWord, error, _("expected a word, but found ‘<blue>{}<reset>‘."), 1);

  NewMsg(UnknownKeyword, error, _("unknown keyword ‘<blue>{}<reset>‘."), 1);

  NewMsg(NoMatchOperator, error, _("no match for ‘<blue>operator <yellow>{0}<reset>’ (‘<blue>{1}<reset>‘ <yellow>{0}<reset> ‘<blue>{2}<reset>’)."), 3);

  NewMsg(EmptyCharacterConstant, error, _("empty character constant ‘‘."), 0);
  NewMsg(CharacterConstantTooLong, error, _("character constant too long ‘<blue>{}<reset>‘."), 1);
  NewMsg(CantConvertInteger, error, _("‘<blue>{}<reset>‘ could not be converted to int."), 1);

  NewMsg(UnexpectedIdentifier, error, _("un expected identifier ‘<blue>{}<reset>‘."), 1);

  NewMsg(IdentifierNotFound, error, _("identifier not found ‘<blue>{}<reset>‘."), 1);
  NewMsg(IdentifierNType, error, _("identifier is not a type ‘<blue>{}<reset>‘."), 1);
  NewMsg(IdentifierNExpr, error, _("identifier is not a expr ‘<blue>{}<reset>‘."), 1);

  NewMsg(OnlyOneVariableCanBeInitialized, error, _("only one variable can be initialized."), 1);
  NewMsg(VisibilitySettingNApplicableInContext, error, _("‘<blue>{}<reset>‘ visibility setting is not applicable in this context."), 1);
}

#undef NewMsg
#undef NewMsgF

#undef REP
#undef REP_0
#undef REP_1
#undef REP_2
#undef REP_3

#undef REPN
#undef REPN_0
#undef REPN_1
#undef REPN_2
#undef REPN_3
