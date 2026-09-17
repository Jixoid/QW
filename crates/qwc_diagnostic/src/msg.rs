use crate::CodedMsg;

// Coded Messages
pub const MUTUALLY_CONTRADICTORY_DEFINITIONS: CodedMsg = CodedMsg::new(0x0001, "mutually contradictory definitions");
pub const CANNOT_FIELD_ACCESS_AFTER_MEMBER:   CodedMsg = CodedMsg::new(0x0002, "cannot use `::` on a field access expression, use type name instead");
pub const UNKNOWN_ATTRIBUTE:                  CodedMsg = CodedMsg::new(0x0003, "unknown attribute");
pub const CANNOT_CONVERT_TO_INT:              CodedMsg = CodedMsg::new(0x0004, "cannot convert to int");


// Non Coded Messages
pub const FILE_FINISHED:              CodedMsg = CodedMsg::new_str("file finished");
pub const EXPECTED_IDENTIFIER:        CodedMsg = CodedMsg::new_str("expected identifier");
pub const EXPECTED_IDENTIFIER_AFTER:  CodedMsg = CodedMsg::new_str("expected identifier after: `{}`");
pub const EXPECTED_BUT_FOUND:         CodedMsg = CodedMsg::new_str("expected `{}`, but found `{}`");
pub const UNKNOWN_KEYWORD:            CodedMsg = CodedMsg::new_str("unknown keyword");
pub const UNKNOWN_PATTERN:            CodedMsg = CodedMsg::new_str("unknown pattern");
pub const UNKNOWN_USE_STARTER:        CodedMsg = CodedMsg::new_str("unknown use starter");
pub const UNKNOWN_USE_SEGMENT:        CodedMsg = CodedMsg::new_str("unknown use segment");
pub const VISIBILITY_AFTER_ATTRIBUTE: CodedMsg = CodedMsg::new_str("a visibility modifier cannot appear after the attributes");
pub const DUPLICATE_ATTRIBUTE:        CodedMsg = CodedMsg::new_str("duplicate attribute");
pub const DUPLICATE_IDENTIFIER:       CodedMsg = CodedMsg::new_str("duplicate identifier");
pub const UNRESOLVED_IDENTIFIER:      CodedMsg = CodedMsg::new_str("unresolved identifier");
pub const NOTFOUND_IN_SCOPE:          CodedMsg = CodedMsg::new_str("`{}` not found in `{}` scope");


// Label
pub const FIRST_DEFINITION_HERE:      &str = "first definition is here";
pub const CONFLICTING_DEFINITION:     &str = "conflicting definition";
pub const ONLY_ONE_DEFINITION_REMAIN: &str = "only one definition may remain";
