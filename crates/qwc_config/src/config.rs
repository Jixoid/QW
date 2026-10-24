/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use rustc_hash::{FxHashMap, FxHashSet};

use crate::{Version, VerReq};


// Config
pub struct Config {
  pub name: String,
  pub vers: Version,

  pub feat: FxHashSet<String>,
  pub deps: FxHashMap<String, Deps>,
}


#[derive(Hash)]
pub enum GitSelect { Branch(String), Tag(String), Rev(String) }

pub enum RepoFrom {
  WSpace {
    name: String
  },
  Git {
    repo: String,
    select: Option<GitSelect>,
  },
  Remote {
    verreq: Option<VerReq>,
    remote: Option<String>,
  },
}

pub struct Deps {
  pub feat: Option<FxHashSet<String>>,
  pub from: RepoFrom,
}
