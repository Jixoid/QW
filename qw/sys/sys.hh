/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "qw/control/context.hh"
#include "qw/tree/decls.hh"



namespace qw::sys
{
  fun build_sys_module(qw::context *ctx) -> decls::Decl*;
}
