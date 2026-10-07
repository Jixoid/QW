/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
  /// Byte offsets where each line starts (0-indexed).
  line_starts: Vec<u32>,
  /// Total length of the text in bytes.
  text_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
  /// 0-indexed line number
  pub line: u32,
  /// 0-indexed character offset (UTF-8 character count from start of line)
  pub col: u32,
}

impl LineIndex {
  pub fn new(text: &str) -> Self {
    let mut line_starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
      if b == b'\n' {
        line_starts.push((i + 1) as u32);
      }
    }

    Self {
      line_starts,
      text_len: text.len(),
    }
  }

  pub fn line_count(&self) -> usize {
    self.line_starts.len()
  }

  pub fn offset_to_line_col(&self, text: &str, offset: usize) -> Option<LineCol> {
    if offset > self.text_len {
      return None;
    }

    // Binary search to find the line
    let line_idx = match self.line_starts.binary_search(&(offset as u32)) {
      Ok(idx) => idx,
      Err(idx) => idx.saturating_sub(1),
    };

    let line_start = self.line_starts[line_idx] as usize;
    let slice = &text[line_start..offset];
    let col = slice.chars().count() as u32;

    Some(LineCol {
      line: line_idx as u32,
      col,
    })
  }

  pub fn line_col_to_offset(&self, text: &str, line_col: LineCol) -> Option<usize> {
    let line_idx = line_col.line as usize;
    if line_idx >= self.line_starts.len() {
      return None;
    }

    let line_start = self.line_starts[line_idx] as usize;
    let next_line_start = self
      .line_starts
      .get(line_idx + 1)
      .copied()
      .map(|o| o as usize)
      .unwrap_or(self.text_len);

    let line_str = &text[line_start..next_line_start];
    let mut byte_offset = line_start;

    for (i, c) in line_str.chars().enumerate() {
      if (i as u32) >= line_col.col { break }
      
      byte_offset += c.len_utf8();
    }

    Some(byte_offset)
  }
}
