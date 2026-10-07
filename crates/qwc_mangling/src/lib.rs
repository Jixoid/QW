/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_string_interner::{Sid, StrInterner};


pub trait Mangler {
  fn new(sin: &StrInterner, mgr: &[Sid], now: Sid) -> String;
  fn new_impl(sin: &StrInterner, mgr: &[Sid], target_ty: Sid, iface: Option<Sid>, method: Sid) -> String;
  fn new_vmt(sin: &StrInterner, mgr: &[Sid], target_ty: Sid, iface: Sid) -> String;
}


pub struct ManglerQW;

impl Mangler for ManglerQW {
  fn new(sin: &StrInterner, mgr: &[Sid], now: Sid) -> String {
    let mut ret = String::from("qw_");

    for sid in mgr {
      let str = sin.str(*sid);
      ret += &format!("{}{}", str.len(), str);
    }

    let str = sin.str(now);
    ret += &format!("{}{}", str.len(), str);

    ret
  }

  fn new_impl(sin: &StrInterner, mgr: &[Sid], target_ty: Sid, iface: Option<Sid>, method: Sid) -> String {
    let mut ty_str = String::new();
    for sid in mgr {
      let s = sin.str(*sid);
      ty_str += &format!("{}{}", s.len(), s);
    }
    let s = sin.str(target_ty);
    ty_str += &format!("{}{}", s.len(), s);

    let mut method_str = String::new();
    if let Some(iface_sid) = iface {
      let s = sin.str(iface_sid);
      method_str += &format!("{}{}", s.len(), s);
    }
    let s = sin.str(method);
    method_str += &format!("{}{}", s.len(), s);

    format!("qw_impl_{}_{}", ty_str, method_str)
  }

  fn new_vmt(sin: &StrInterner, mgr: &[Sid], target_ty: Sid, iface: Sid) -> String {
    let mut ty_str = String::new();
    for sid in mgr {
      let s = sin.str(*sid);
      ty_str += &format!("{}{}", s.len(), s);
    }
    let s = sin.str(target_ty);
    ty_str += &format!("{}{}", s.len(), s);

    let iface_s = sin.str(iface);
    let iface_str = format!("{}{}", iface_s.len(), iface_s);

    format!("qw_vmt_{}_{}", ty_str, iface_str)
  }
}
