/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{CodedMsg, FmtMsg};

// Coded Messages
pub const MUTUALLY_CONTRADICTORY_DEFINITIONS:               CodedMsg = CodedMsg::new(0x0001, "mutually contradictory definitions");
pub const CANNOT_FIELD_ACCESS_AFTER_MEMBER:                 CodedMsg = CodedMsg::new(0x0002, "cannot use `::` on a field access expression, use type name instead");
pub const UNKNOWN_ATTRIBUTE:                                CodedMsg = CodedMsg::new(0x0003, "unknown attribute: `{}`");
pub const CANNOT_CONVERT_TO_INT:                            CodedMsg = CodedMsg::new(0x0004, "cannot convert to int");
pub const COULD_NOT_FIND_MODULE_FILE:                       CodedMsg = CodedMsg::new(0x0005, "could not find module file");
pub const CANNOT_FIND_X_IN_SCOPE:                           CodedMsg = CodedMsg::new(0x0006, "cannot find `{}` in this scope");
pub const NOT_FOUND_IN_SCOPE:                               CodedMsg = CodedMsg::new(0x0007, "`{}` not found in `{}` scope");
pub const LOOP_BRANCHES_HAVE_INCOMPATIBLE_TYPE:             CodedMsg = CodedMsg::new(0x0007, "loop branches have incompatible types");
pub const CANNOT_ASSIGN_IMMUTABLE:                          CodedMsg = CodedMsg::new(0x0008, "cannot assign to immutable expression");
pub const CANNOT_ASSIGN_RVALUE:                             CodedMsg = CodedMsg::new(0x0009, "cannot assign to rvalue");
pub const VARIABLE_REQUIRES_INITIALIZER:                    CodedMsg = CodedMsg::new(0x000a, "variable `{}` requires an initializer expression");
pub const MISMATCHED_TYPES:                                 CodedMsg = CodedMsg::new(0x000b, "mismatched types: expected `{}`, found `{}`");
pub const DST_TYPES_CANNOT_EXIST_IN_X:                      CodedMsg = CodedMsg::new(0x000c, "DST types cannot exist within a {}");
pub const META_TYPES_CANNOT_EXIST_IN_X:                     CodedMsg = CodedMsg::new(0x000c, "meta types cannot exist within a {}");
pub const FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED:   CodedMsg = CodedMsg::new(0x000d, "function takes {} arguments but {} arguments were supplied");
pub const ONLY_TYPES_CAN_BE_INITIALIZED_IN_THIS_WAY:        CodedMsg = CodedMsg::new(0x000e, "only types can be initialized in this way");
pub const FIELD_X_SPECIFIED_MORE_THAN_ONCE:                 CodedMsg = CodedMsg::new(0x000f, "field `{}` specified more than once");
pub const HAS_NO_FIELD_NAMED_X:                             CodedMsg = CodedMsg::new(0x0010, "has no field named `{}`");
pub const MISSING_FIELDS_X_ININITIALIZER:                   CodedMsg = CodedMsg::new(0x0011, "missing fields {} in initializer");
pub const NOT_ALL_IFACE_ITEMS_IMPLEMENTED:                  CodedMsg = CodedMsg::new(0x0012, "not all iface items implemented, missing: {}");
pub const METHOD_NOT_A_MEMBER_OF_IFACE:                     CodedMsg = CodedMsg::new(0x0013, "method `{}` is not a member of iface `{}`");
pub const INCOMPATIBLE_IFACE_METHOD_TYPE:                   CodedMsg = CodedMsg::new(0x0014, "method `{}` has an incompatible type for iface `{}`");
pub const X_NOT_IMPLEMENTED_FOR_TYPE:                       CodedMsg = CodedMsg::new(0x0016, "the {} `{}` is not implemented for `{}`");
pub const CANNOT_CAST_X_TO_Y:                               CodedMsg = CodedMsg::new(0x0017, "cannot cast `{}` to `{}`");
pub const SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT:  CodedMsg = CodedMsg::new(0x0018, "`Self` type is only allowed in associated context");
pub const SELF_PARAMETER_IS_ONLY_ALLOWED_IN_ASSOCIATED_FUN: CodedMsg = CodedMsg::new(0x0018, "self parameter is only allowed in associated function");
pub const DUPLICATE_ATTRIBUTE:                              CodedMsg = CodedMsg::new(0x0019, "duplicate attribute");
pub const ENTRY_FUNCTION_MUST_BE_PUBLIC:                    CodedMsg = CodedMsg::new(0x001a, "entry function must be public");
pub const INCOMPATIBLE_METHOD:                              CodedMsg = CodedMsg::new(0x001b, "method has an incompatible for `{}`");
pub const USE_ASSOCIATED_FUNCTION_SYNTAX_INSTEAD:           CodedMsg = CodedMsg::new(0x001c, "use associated function syntax instead: `{}`");
pub const UNREACHABLE_STATEMENT:                            CodedMsg = CodedMsg::new(0x001d, "unreachable statement");
pub const TYPE_NOT_SPECIFIED:                               CodedMsg = CodedMsg::new(0x001e, "type not specified");


// Non Coded Messages
pub const FILE_FINISHED:                    CodedMsg = CodedMsg::new_str("file finished");
pub const EXPECTED_IDENTIFIER:              CodedMsg = CodedMsg::new_str("expected identifier");
pub const EXPECTED_IDENTIFIER_AFTER:        CodedMsg = CodedMsg::new_str("expected identifier after: `{}`");
pub const EXPECTED_BUT_FOUND:               CodedMsg = CodedMsg::new_str("expected `{}`, but found `{}`");
pub const EXPECTED_BUT_FOUND2:              CodedMsg = CodedMsg::new_str("expected `{}` or `{}`, but found `{}`");
pub const EXPECTED_BUT_FOUND3:              CodedMsg = CodedMsg::new_str("expected `{}`, `{}` or `{}`, but found `{}`");
pub const EXPECTED_BUT_FOUND4:              CodedMsg = CodedMsg::new_str("expected `{}`, `{}`, `{}` or `{}`, but found `{}`");
pub const UNKNOWN_KEYWORD:                  CodedMsg = CodedMsg::new_str("unknown keyword");
pub const UNKNOWN_PATTERN:                  CodedMsg = CodedMsg::new_str("unknown pattern");
pub const UNKNOWN_USE_STARTER:              CodedMsg = CodedMsg::new_str("unknown use starter");
pub const UNKNOWN_USE_SEGMENT:              CodedMsg = CodedMsg::new_str("unknown use segment");
pub const VISIBILITY_AFTER_ATTRIBUTE:       CodedMsg = CodedMsg::new_str("a visibility modifier cannot appear after the attributes");
pub const DUPLICATE_IDENTIFIER:             CodedMsg = CodedMsg::new_str("duplicate identifier");
pub const DID_YOU_MEAN:                     CodedMsg = CodedMsg::new_str("did you mean `{}`?");
pub const CANNOT_USE_GENERIC_WITHOUT_SPECIALIZATION: CodedMsg = CodedMsg::new_str("cannot use generic type without specialization");


// Label
pub const CANNOT_ASSIGN_TO_THIS_EXPRESSION: FmtMsg = FmtMsg::new("cannot assign to this expression");
pub const FIRST_DEFINITION_HERE:            FmtMsg = FmtMsg::new("first definition is here");
pub const DEFINED_HERE:                     FmtMsg = FmtMsg::new("defined here");
pub const X_DEFINED_HERE:                   FmtMsg = FmtMsg::new("`{}` defined here");
pub const CONFLICTING_DEFINITION:           FmtMsg = FmtMsg::new("conflicting definition");
pub const ONLY_ONE_DEFINITION_REMAIN:       FmtMsg = FmtMsg::new("only one definition may remain");
pub const NOT_FOUND_IN:                     FmtMsg = FmtMsg::new("not found in `{}`");
pub const NOT_FOUND_IN_OR2:                 FmtMsg = FmtMsg::new("not found in `{}` or `{}`");
pub const YOU_SAID:                         FmtMsg = FmtMsg::new("you said `{}`");
pub const EXPECTED_X:                       FmtMsg = FmtMsg::new("expected `{}`");
pub const X_ARGUMENTS_ARE_MISSING:          FmtMsg = FmtMsg::new("{} arguments are missing");
pub const MISSING_IN_IMPLEMENTATION:        FmtMsg = FmtMsg::new("missing {} in implementation");
pub const NOT_A_MEMBER_OF_IFACE:            FmtMsg = FmtMsg::new("not a member of iface `{}`");
pub const CANNOT_CAST:                      FmtMsg = FmtMsg::new("cannot cast");
pub const UNEXPECTED_PARAMETER:             FmtMsg = FmtMsg::new("unexpected parameter:  `{}`");
pub const ARGUMENT_X_IS_MISSING:            FmtMsg = FmtMsg::new("argument `{}` is missing");
