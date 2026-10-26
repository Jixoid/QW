/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::num::NonZeroU32;

use lasso::{Key, Rodeo, Spur};
use qwc_serial_macro::{Decode, Encode};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
#[serial(map)]
pub struct Sid (NonZeroU32);


impl From<NonZeroU32> for Sid {
  fn from(value: NonZeroU32) -> Self { Self(value) }
}


#[derive(Clone)]
pub struct StrInterner {
  rodeo: Rodeo,

  sid_core: Sid,
  sid_types: Sid,
  
  sid_bool: Sid,
  
  sid_usize: Sid,
  sid_isize: Sid,

  sid_import: Sid,
  sid_export: Sid,
  sid_entry: Sid,
  sid_c: Sid,

  sid_self: Sid,

  sid_add: Sid,
  sid_sub: Sid,
  sid_mul: Sid,
  sid_div: Sid,
  sid_rem: Sid,

  sid_add_assign: Sid,
  sid_sub_assign: Sid,
  sid_mul_assign: Sid,
  sid_div_assign: Sid,
  sid_rem_assign: Sid,

  sid_neg: Sid,
  sid_not: Sid,
  sid_deref: Sid,
  
  sid_eq: Sid,
  sid_ne: Sid,
  sid_ordering: Sid,
  sid_less: Sid,
  sid_equal: Sid,
  sid_greater: Sid,

  sid_partial_ord: Sid,
  sid_ord: Sid,
  sid_partial_cmp: Sid,
  sid_cmp: Sid,
  sid_lt: Sid,
  sid_le: Sid,
  sid_gt: Sid,
  sid_ge: Sid,
}

impl StrInterner {
  
  #[inline]
  pub fn sid(&mut self, s: &str) -> Sid {
    let spur = self.rodeo.get_or_intern(s);
    Sid::from(spur.into_inner())
  }

  #[inline]
  pub fn get(&self, s: &str) -> Option<Sid> {
    self.rodeo.get(s).map(|s| Sid::from(s.into_inner()))
  }

  #[inline]
  pub fn str(&self, sid: Sid) -> &str {
    let spur = Spur::try_from_usize(sid.0.get() as usize -1).unwrap();
    self.rodeo.resolve(&spur)
  }


  pub fn sid_core(&self) -> Sid  { self.sid_core }
  pub fn sid_types(&self) -> Sid { self.sid_types }

  pub fn sid_bool(&self) -> Sid { self.sid_bool }
  
  pub fn sid_usize(&self) -> Sid { self.sid_usize }
  pub fn sid_isize(&self) -> Sid { self.sid_isize }

  pub fn sid_import(&self) -> Sid { self.sid_import }
  pub fn sid_export(&self) -> Sid { self.sid_export }
  pub fn sid_entry(&self) -> Sid  { self.sid_entry }
  pub fn sid_c(&self) -> Sid  { self.sid_c }
  pub fn sid_self(&self) -> Sid   { self.sid_self }

  pub fn sid_add(&self) -> Sid { self.sid_add }
  pub fn sid_sub(&self) -> Sid { self.sid_sub }
  pub fn sid_mul(&self) -> Sid { self.sid_mul }
  pub fn sid_div(&self) -> Sid { self.sid_div }
  pub fn sid_rem(&self) -> Sid { self.sid_rem }

  pub fn sid_add_assign(&self) -> Sid { self.sid_add_assign }
  pub fn sid_sub_assign(&self) -> Sid { self.sid_sub_assign }
  pub fn sid_mul_assign(&self) -> Sid { self.sid_mul_assign }
  pub fn sid_div_assign(&self) -> Sid { self.sid_div_assign }
  pub fn sid_rem_assign(&self) -> Sid { self.sid_rem_assign }

  pub fn sid_neg(&self) -> Sid { self.sid_neg }
  pub fn sid_not(&self) -> Sid { self.sid_not }
  pub fn sid_deref(&self) -> Sid { self.sid_deref }
  
  pub fn sid_eq(&self) -> Sid { self.sid_eq }
  pub fn sid_ne(&self) -> Sid { self.sid_ne }
  pub fn sid_ordering(&self) -> Sid { self.sid_ordering }
  pub fn sid_less(&self) -> Sid { self.sid_less }
  pub fn sid_equal(&self) -> Sid { self.sid_equal }
  pub fn sid_greater(&self) -> Sid { self.sid_greater }

  pub fn sid_partial_ord(&self) -> Sid { self.sid_partial_ord }
  pub fn sid_ord(&self) -> Sid { self.sid_ord }
  pub fn sid_partial_cmp(&self) -> Sid { self.sid_partial_cmp }
  pub fn sid_cmp(&self) -> Sid { self.sid_cmp }
  pub fn sid_lt(&self) -> Sid { self.sid_lt }
  pub fn sid_le(&self) -> Sid { self.sid_le }
  pub fn sid_gt(&self) -> Sid { self.sid_gt }
  pub fn sid_ge(&self) -> Sid { self.sid_ge }
}

impl Default for StrInterner {
  fn default() -> Self {
    let mut rodeo = Rodeo::default();

    Self {
      sid_core:  Sid::from(rodeo.get_or_intern_static("core").into_inner()),
      sid_types: Sid::from(rodeo.get_or_intern_static("types").into_inner()),
      
      sid_bool: Sid::from(rodeo.get_or_intern_static("bool").into_inner()),
      
      sid_usize: Sid::from(rodeo.get_or_intern_static("usize").into_inner()),
      sid_isize: Sid::from(rodeo.get_or_intern_static("isize").into_inner()),

      sid_import: Sid::from(rodeo.get_or_intern_static("import").into_inner()),
      sid_export: Sid::from(rodeo.get_or_intern_static("export").into_inner()),
      sid_entry:  Sid::from(rodeo.get_or_intern_static("entry").into_inner()),
      sid_c:      Sid::from(rodeo.get_or_intern_static("C").into_inner()),

      sid_self: Sid::from(rodeo.get_or_intern_static("self").into_inner()),
      
      sid_add: Sid::from(rodeo.get_or_intern_static("add").into_inner()),
      sid_sub: Sid::from(rodeo.get_or_intern_static("sub").into_inner()),
      sid_mul: Sid::from(rodeo.get_or_intern_static("mul").into_inner()),
      sid_div: Sid::from(rodeo.get_or_intern_static("div").into_inner()),
      sid_rem: Sid::from(rodeo.get_or_intern_static("rem").into_inner()),

      sid_add_assign: Sid::from(rodeo.get_or_intern_static("add_assign").into_inner()),
      sid_sub_assign: Sid::from(rodeo.get_or_intern_static("sub_assign").into_inner()),
      sid_mul_assign: Sid::from(rodeo.get_or_intern_static("mul_assign").into_inner()),
      sid_div_assign: Sid::from(rodeo.get_or_intern_static("div_assign").into_inner()),
      sid_rem_assign: Sid::from(rodeo.get_or_intern_static("rem_assign").into_inner()),

      sid_neg: Sid::from(rodeo.get_or_intern_static("neg").into_inner()),
      sid_not: Sid::from(rodeo.get_or_intern_static("not").into_inner()),
      sid_deref: Sid::from(rodeo.get_or_intern_static("deref").into_inner()),
      
      sid_eq: Sid::from(rodeo.get_or_intern_static("eq").into_inner()),
      sid_ne: Sid::from(rodeo.get_or_intern_static("ne").into_inner()),
      sid_ordering: Sid::from(rodeo.get_or_intern_static("Ordering").into_inner()),
      sid_less: Sid::from(rodeo.get_or_intern_static("Less").into_inner()),
      sid_equal: Sid::from(rodeo.get_or_intern_static("Equal").into_inner()),
      sid_greater: Sid::from(rodeo.get_or_intern_static("Greater").into_inner()),

      sid_partial_ord: Sid::from(rodeo.get_or_intern_static("PartialOrd").into_inner()),
      sid_ord: Sid::from(rodeo.get_or_intern_static("Ord").into_inner()),
      sid_partial_cmp: Sid::from(rodeo.get_or_intern_static("partial_cmp").into_inner()),
      sid_cmp: Sid::from(rodeo.get_or_intern_static("cmp").into_inner()),
      sid_lt: Sid::from(rodeo.get_or_intern_static("lt").into_inner()),
      sid_le: Sid::from(rodeo.get_or_intern_static("le").into_inner()),
      sid_gt: Sid::from(rodeo.get_or_intern_static("gt").into_inner()),
      sid_ge: Sid::from(rodeo.get_or_intern_static("ge").into_inner()),
      
      rodeo,
    }
  }
}
