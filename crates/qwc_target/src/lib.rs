/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use bitflags::bitflags;


pub enum Arch {
  X86_64,
  I686,
  ARM,
  AArch64,
  RiscV32,
  RiscV64,
  SpirV32,
  SpirV64,
}

pub enum Vendor {
  Unknown,
  Apple,
}

pub enum Sys {
  Vulkan,
  OpenCL,
  Linux,
  Window,
  Android,
  Darwin,
  IOS,
}

pub enum Abi {
  Unknown,
  GNU,
  Android,
}


pub struct Triple (pub Arch, pub Vendor, pub Sys, pub Abi);



pub fn get_dialect(triple: Triple) -> DialectOption {
  use Arch::*;
  
  match triple {
    Triple(X86_64 | I686 | ARM | AArch64 | RiscV32 | RiscV64, ..) => DialectOption::empty(),

    Triple(SpirV32 | SpirV64, ..) =>
      DialectOption::SPECIFIED_ADDRESS_SPACES |
      DialectOption::DENY_RECURSIVE_TYPES |
      DialectOption::DENY_RECURSIVE_CALLS |
      DialectOption::DENY_DYNAMIC_DISPATCH |
      DialectOption::DENY_FN_POINTERS,
  }
}



bitflags! {
  #[derive(Clone, Copy)]
  pub struct DialectOption: u32 {
    /// Enforces the specification of Address Spaces.
    /// 
    /// e.g., ON = `&:0 i32` OFF = `&i32`
    const SPECIFIED_ADDRESS_SPACES = 1 << 0;

    /// It prevents recursive definitions of types.
    /// 
    /// ```qw
    /// struct Node {
    ///   next: &Node;
    /// }
    /// ```
    const DENY_RECURSIVE_TYPES = 1 << 1;

    /// It prevents recursive calls to functions.
    /// 
    /// ```qw
    /// fun fib(i: i32) -> i32 {
    ///   if i == 0 { ret 0; }
    /// 
    ///   i + fib(i-1)
    /// }
    /// ```
    const DENY_RECURSIVE_CALLS = 1 << 2;

    /// It prevents dynamic types and structures.
    /// 
    /// e.g., `iface` `class`
    const DENY_DYNAMIC_DISPATCH = 1 << 3;

    /// Prevents function pointers.
    /// 
    /// e.g., `fun(...) -> T`
    /// 
    /// The 'Fun(...) -> T' trait is still usable.
    const DENY_FN_POINTERS = 1 << 4;

    /// Prevents all pointers.
    /// 
    /// e.g., `^T` `&T`
    const DENY_POINTERS = 1 << 5;
  }
}
