/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qw/basis.hh"
#include "qw/vfs/vfs.hh"



namespace vfs::__file
{
	/**
	 * @brief Gets a RAII wrapper for the "file" VFS provider.
	 * 
	 * This registers the provider upon initialization and automatically unregisters 
	 * and cleans up the provider when the returned object goes out of scope.
	 * 
	 * @return A RAII object managing the "file" provider lifecycle.
	 */
	fun get() -> vfs_raii;
}
