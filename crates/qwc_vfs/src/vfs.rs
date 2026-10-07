/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use rustc_hash::FxHashMap;

use crate::file::{FileId, VfsFile};


#[derive(Debug, Default, Clone)]
pub struct Vfs {
  files: Vec<VfsFile>,
  path_to_id: FxHashMap<PathBuf, FileId>,
}

impl Vfs {
  pub fn new() -> Self {
    Self {
      files: Vec::new(),
      path_to_id: FxHashMap::default(),
    }
  }

  pub fn normalize_path(path: &Path) -> PathBuf {
    match path.canonicalize() {
      Ok(canonical) => canonical,
      Err(_) => {
        if path.is_absolute() {
          path.to_path_buf()
        } else {
          std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
        }
      }
    }
  }

  pub fn set_overlay(&mut self, path: impl AsRef<Path>, content: Vec<u8>, version: i32) -> FileId {
    let normalized = Self::normalize_path(path.as_ref());

    if let Some(&file_id) = self.path_to_id.get(&normalized) {
      let file = &mut self.files[file_id.0 as usize];
      *file = VfsFile::new(file_id, normalized, content, version, true);
      file_id
    } else {
      let file_id = FileId(self.files.len() as u32);
      let file = VfsFile::new(file_id, normalized.clone(), content, version, true);
      self.files.push(file);
      self.path_to_id.insert(normalized, file_id);
      file_id
    }
  }

  pub fn remove_overlay(&mut self, path: impl AsRef<Path>) -> Option<FileId> {
    let normalized = Self::normalize_path(path.as_ref());
    let file_id = *self.path_to_id.get(&normalized)?;

    if normalized.exists() {
      if let Ok(disk_content) = fs::read(&normalized) {
        self.files[file_id.0 as usize] =
          VfsFile::new(file_id, normalized, disk_content, 0, false);
      }
    }

    Some(file_id)
  }

  pub fn read(&mut self, path: impl AsRef<Path>) -> io::Result<&VfsFile> {
    let normalized = Self::normalize_path(path.as_ref());

    if let Some(&file_id) = self.path_to_id.get(&normalized) {
      return Ok(&self.files[file_id.0 as usize]);
    }

    let disk_content = fs::read(&normalized)?;
    let file_id = FileId(self.files.len() as u32);
    let file = VfsFile::new(file_id, normalized.clone(), disk_content, 0, false);
    self.files.push(file);
    self.path_to_id.insert(normalized, file_id);

    Ok(&self.files[file_id.0 as usize])
  }

  pub fn get(&self, id: FileId) -> Option<&VfsFile> {
    self.files.get(id.0 as usize)
  }

  pub fn get_by_path(&self, path: impl AsRef<Path>) -> Option<&VfsFile> {
    let normalized = Self::normalize_path(path.as_ref());
    let file_id = self.path_to_id.get(&normalized)?;
    self.get(*file_id)
  }

  pub fn file_id(&self, path: impl AsRef<Path>) -> Option<FileId> {
    let normalized = Self::normalize_path(path.as_ref());
    self.path_to_id.get(&normalized).copied()
  }

  pub fn exists(&self, path: impl AsRef<Path>) -> bool {
    let normalized = Self::normalize_path(path.as_ref());
    self.path_to_id.contains_key(&normalized) || normalized.exists()
  }

  pub fn len(&self) -> usize {
    self.files.len()
  }

  pub fn is_empty(&self) -> bool {
    self.files.is_empty()
  }

  pub fn iter(&self) -> impl Iterator<Item = &VfsFile> {
    self.files.iter()
  }

  pub fn clear(&mut self) {
    self.files.clear();
    self.path_to_id.clear();
  }
}
