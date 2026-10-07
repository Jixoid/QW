/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{fs, path::Path};

mod arena;

pub use arena::Arena;


pub struct File {
  fpath: String,
  map: Vec<u8>,
  fid: u16,
}

impl File {
  pub fn fpath(&self) -> &str { &self.fpath }
  pub fn map(&self) -> &[u8] { &self.map }
  pub fn fid(&self) -> u16 { self.fid }
}


pub struct Files {
  arena: Vec<File>
}

impl Files {

  pub fn new() -> Self {
    Self{ arena: Vec::new() }
  }


  pub fn add<'a>(&'a mut self, fpath: &Path) -> u16 {
    let map = fs::read(fpath).unwrap();
    self.add_buffer(fpath, map)
  }

  pub fn add_buffer(&mut self, fpath: &Path, mut map: Vec<u8>) -> u16 {
    map.resize(map.len() + 8, 0);

    let fi = File{
      fid: u16::try_from(self.arena.len()).unwrap(),
      fpath: fpath.to_str().unwrap().to_string(),
      map,
    };

    self.arena.push(fi);
    (self.arena.len()-1) as u16
  }

  pub fn get(&self, fid: u16) -> &File {
    &self.arena[fid as usize]
  }

}
