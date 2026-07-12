/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "qw/basis.hh"
#include "qw/diagnostic/diagnostic.hh"



namespace qw
{

  struct IFrontend
  {
    public:
      virtual ~IFrontend() = default;
      
    public:
      virtual fun process() -> diagnostic::summary = 0;
      virtual fun get_root() -> decls::Decl* = 0;
  };


  enum struct StageKind { Unknown, Sema, Codegen, Optimization };

  struct IStage
  {
    public:
      virtual ~IStage() = default;
      
    public:
      virtual fun kind() const noexcept -> StageKind = 0;
      virtual fun process(decls::Decl *root) -> diagnostic::summary = 0;
  };

}
