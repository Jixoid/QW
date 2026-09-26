use crate::{CodedMsg, FmtMsg};

// Coded Messages
pub const MUTUALLY_CONTRADICTORY_DEFINITIONS:   CodedMsg = CodedMsg::new(0x0001, "mutually contradictory definitions");
pub const CANNOT_FIELD_ACCESS_AFTER_MEMBER:     CodedMsg = CodedMsg::new(0x0002, "cannot use `::` on a field access expression, use type name instead");
pub const UNKNOWN_ATTRIBUTE:                    CodedMsg = CodedMsg::new(0x0003, "unknown attribute");
pub const CANNOT_CONVERT_TO_INT:                CodedMsg = CodedMsg::new(0x0004, "cannot convert to int");
pub const COULD_NOT_FIND_MODULE_FILE:           CodedMsg = CodedMsg::new(0x0005, "could not find module file");
pub const CANNOT_FIND_X_IN_SCOPE:               CodedMsg = CodedMsg::new(0x0006, "cannot find `{}` in this scope");
pub const NOT_FOUND_IN_SCOPE:                   CodedMsg = CodedMsg::new(0x0007, "`{}` not found in `{}` scope");
pub const LOOP_BRANCHES_HAVE_INCOMPATIBLE_TYPE: CodedMsg = CodedMsg::new(0x0007, "loop branches have incompatible types");
pub const CANNOT_ASSIGN_IMMUTABLE:              CodedMsg = CodedMsg::new(0x0008, "cannot assign to immutable expression");
pub const CANNOT_ASSIGN_RVALUE:                 CodedMsg = CodedMsg::new(0x0009, "cannot assign to rvalue");
pub const VARIABLE_REQUIRES_INITIALIZER:        CodedMsg = CodedMsg::new(0x000a, "variable `{}` requires an initializer expression");
pub const MISMATCHED_TYPES:                     CodedMsg = CodedMsg::new(0x000b, "mismatched types: expected `{}`, found `{}`");


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
pub const DUPLICATE_ATTRIBUTE:              CodedMsg = CodedMsg::new_str("duplicate attribute");
pub const DUPLICATE_IDENTIFIER:             CodedMsg = CodedMsg::new_str("duplicate identifier");
pub const DID_YOU_MEAN:                     CodedMsg = CodedMsg::new_str("did you mean `{}`?");
pub const DST_TYPES_CANNOT_EXIST_IN_STRUCT: CodedMsg = CodedMsg::new_str("dst types cannot exist within a struct");
pub const DST_TYPES_CANNOT_EXIST_IN_TUPLE:  CodedMsg = CodedMsg::new_str("dst types cannot exist within a tuple");
pub const DST_TYPES_CANNOT_EXIST_IN_ARRAY:  CodedMsg = CodedMsg::new_str("dst types cannot exist within a array");
pub const DST_TYPES_CANNOT_EXIST_IN_SLICE:  CodedMsg = CodedMsg::new_str("dst types cannot exist within a slice");


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
