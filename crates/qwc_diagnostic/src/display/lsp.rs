/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use lsp_types::{Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Position, Range, Url};
use qwc_arena::Files;

use crate::{Level, Message, Summary};


impl Level {
  pub fn to_lsp_severity(self) -> DiagnosticSeverity {
    match self {
      Level::Fatal | Level::Error => DiagnosticSeverity::ERROR,
      Level::Warn => DiagnosticSeverity::WARNING,
      Level::Hint => DiagnosticSeverity::HINT,
    }
  }
}

impl Message {
  /// Converts this message to an LSP Diagnostic for the specified target file ID.
  /// Returns `None` if this message does not apply to `target_fid`.
  pub fn to_lsp(&self, far: &Files, target_fid: u16) -> Option<Diagnostic> {
    let primary_label = self.labels.first();

    // Check if message belongs to target_fid
    if let Some(lbl) = primary_label {
      if lbl.span.fid != target_fid {
        return None;
      }
    }

    let range = match primary_label {
      Some(lbl) => {
        let interval = lbl.span.interval(far);
        Range {
          start: Position {
            line: interval.start[0].get().saturating_sub(1),
            character: interval.start[1].get().saturating_sub(1),
          },
          end: Position {
            line: interval.end[0].get().saturating_sub(1),
            character: interval.end[1].get().saturating_sub(1),
          },
        }
      }
      None => Range::default(),
    };

    let mut message_text = self.message.msg.clone();
    if let Some(lbl) = primary_label {
      if let Some(lbl_msg) = &lbl.message {
        if !lbl_msg.is_empty() {
          message_text.push_str(": ");
          message_text.push_str(lbl_msg);
        }
      }
    }

    if !self.notes.is_empty() {
      for note in &self.notes {
        message_text.push_str("\nNote: ");
        message_text.push_str(note);
      }
    }

    if !self.suggestions.is_empty() {
      for sugg in &self.suggestions {
        message_text.push_str("\nHelp: ");
        message_text.push_str(&sugg.message.msg);
        if !sugg.replacement.is_empty() {
          message_text.push_str(&format!(" (try `{}`)", sugg.replacement));
        }
      }
    }

    let related_information = if self.labels.len() > 1 || !self.suggestions.is_empty() {
      let mut rel = Vec::new();
      for sec in self.labels.iter().skip(1) {
        let file = far.get(sec.span.fid);
        if let Ok(uri) = Url::from_file_path(file.fpath()) {
          let sec_int = sec.span.interval(far);
          let sec_range = Range {
            start: Position {
              line: sec_int.start[0].get().saturating_sub(1),
              character: sec_int.start[1].get().saturating_sub(1),
            },
            end: Position {
              line: sec_int.end[0].get().saturating_sub(1),
              character: sec_int.end[1].get().saturating_sub(1),
            },
          };
          rel.push(DiagnosticRelatedInformation {
            location: Location {
              uri,
              range: sec_range,
            },
            message: sec.message.clone().unwrap_or_else(|| "referenced here".to_string()),
          });
        }
      }
      for sugg in &self.suggestions {
        for sec in sugg.labels.iter().skip(1) {
          let file = far.get(sec.span.fid);
          if let Ok(uri) = Url::from_file_path(file.fpath()) {
            let sec_int = sec.span.interval(far);
            let sec_range = Range {
              start: Position {
                line: sec_int.start[0].get().saturating_sub(1),
                character: sec_int.start[1].get().saturating_sub(1),
              },
              end: Position {
                line: sec_int.end[0].get().saturating_sub(1),
                character: sec_int.end[1].get().saturating_sub(1),
              },
            };
            rel.push(DiagnosticRelatedInformation {
              location: Location {
                uri,
                range: sec_range,
              },
              message: sec.message.clone().unwrap_or_else(|| "defined here".to_string()),
            });
          }
        }
      }
      if rel.is_empty() {
        None
      } else {
        Some(rel)
      }
    } else {
      None
    };

    let sugg_data = if !self.suggestions.is_empty() {
      let mut fixes = Vec::new();
      for sugg in &self.suggestions {
        let target_label = sugg.labels.first().or(primary_label);
        if let Some(target_lbl) = target_label {
          if target_lbl.span.fid == target_fid {
            let interval = target_lbl.span.interval(far);
            let range = Range {
              start: Position {
                line: interval.start[0].get().saturating_sub(1),
                character: interval.start[1].get().saturating_sub(1),
              },
              end: Position {
                line: interval.end[0].get().saturating_sub(1),
                character: interval.end[1].get().saturating_sub(1),
              },
            };
            fixes.push(serde_json::json!({
              "title": sugg.message.msg.clone(),
              "replacement": sugg.replacement.clone(),
              "range": range,
            }));
          }
        }
      }
      if fixes.is_empty() {
        None
      } else {
        Some(serde_json::json!(fixes))
      }
    } else {
      None
    };

    let code = self.message.code.map(|c| NumberOrString::Number(c as i32));

    Some(Diagnostic {
      range,
      severity: Some(self.level.to_lsp_severity()),
      code,
      code_description: None,
      source: Some("qw".to_string()),
      message: message_text,
      related_information,
      tags: None,
      data: sugg_data,
    })
  }
}

impl Summary {
  /// Converts all diagnostic messages in this summary for a given file into LSP Diagnostics.
  pub fn to_lsp(&self, far: &Files, fid: u16) -> Vec<Diagnostic> {
    self
      .msgs()
      .iter()
      .filter_map(|msg| msg.to_lsp(far, fid))
      .collect()
  }
}
