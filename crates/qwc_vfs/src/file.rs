/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::{Path, PathBuf};
use crate::line_index::LineIndex;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub u32);

impl FileId {
  pub const ROOT: FileId = FileId(0);

  #[inline]
  pub fn as_u16(self) -> u16 {
    self.0 as u16
  }

  #[inline]
  pub fn raw(self) -> u32 {
    self.0
  }
}

impl From<u32> for FileId {
  fn from(val: u32) -> Self {
    FileId(val)
  }
}

impl From<u16> for FileId {
  fn from(val: u16) -> Self {
    FileId(val as u32)
  }
}


#[derive(Debug, Clone)]
pub struct VfsFile {
  id: FileId,
  path: PathBuf,
  content: Vec<u8>,
  version: i32,
  is_overlay: bool,
}

impl VfsFile {
  pub fn new(id: FileId, path: PathBuf, content: Vec<u8>, version: i32, is_overlay: bool) -> Self {
    Self {
      id,
      path,
      content,
      version,
      is_overlay,
    }
  }

  #[inline]
  pub fn id(&self) -> FileId {
    self.id
  }

  #[inline]
  pub fn path(&self) -> &Path {
    &self.path
  }

  #[inline]
  pub fn content(&self) -> &[u8] {
    &self.content
  }

  pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(&self.content)
  }

  #[inline]
  pub fn version(&self) -> i32 {
    self.version
  }

  #[inline]
  pub fn is_overlay(&self) -> bool {
    self.is_overlay
  }

  pub fn line_index(&self) -> Option<LineIndex> {
    self.as_str().ok().map(LineIndex::new)
  }
}
