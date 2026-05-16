/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#pragma once

#include "qw/basis.hh"
#include <cstddef>
#include <iostream>
#include <istream>
#include <string_view>



namespace vfs
{

	struct mapped
	{
		protected:
			mapped() = default;

		public:
			virtual ~mapped() = default;

		
		protected:
			void* m_data{};
			u0 m_size;

		public:
			inline fun data() { return m_data; }
			inline fun size() { return m_size; }

			inline fun view() { return std::string_view{(char*)data(), size()}; }
			inline fun span() { return std::span<char>{(char*)data(), size()}; }

		public:
			inline operator ::data() { return {data(), size()}; }
	};



	/**
	 * @brief Virtual file system provider interface.
	 */
	struct vfs_provider
	{
		/**
		 * @brief Initializes the VFS provider.
		 * @return A pointer to the internal state of the provider.
		 */
		fun (*init)() -> void* {};

		/**
		 * @brief Finalizes and cleans up the VFS provider.
		 * @param state A pointer to the internal state returned by init().
		 */
		fun (*fini)(void* state) -> void {};

		/**
		 * @brief Opens a file in read-only mode.
		 * @param state A pointer to the internal state of the provider.
		 * @param fpath The path of the file to open.
		 * @return A shared pointer to an input stream, or null if it fails.
		 */
		fun (*open_ro)(void* state, std::string_view fpath) -> sptr<std::istream> {};


		fun (*open_map)(void* state, std::string_view fpath) -> sptr<mapped> {};

		/**
		 * @brief Opens a file in read-write mode.
		 * @param state A pointer to the internal state of the provider.
		 * @param fpath The path of the file to open.
		 * @return A shared pointer to an input/output stream, or null if it fails.
		 */
		fun (*open_rw)(void* state, std::string_view fpath) -> sptr<std::iostream> {};
	};



	/**
	 * @brief Registers a VFS provider for a specific protocol.
	 * @param protocol The protocol identifier.
	 * @param provider Pointer to the provider structure.
	 */
	fun provider_reg(std::string protocol, vfs_provider* provider) -> void;

	/**
	 * @brief Deletes/unregisters a VFS provider for a specific protocol.
	 * @param protocol The protocol identifier to remove.
	 */
	fun provider_del(std::string protocol) -> void;


	/**
	 * @brief Resolves a file path and opens it in read-only mode.
	 * @param fpath The path of the file to resolve.
	 * @return A shared pointer to an input stream.
	 */
	fun resolve_ro(std::string_view fpath) -> sptr<std::istream>;


	/**
	 * @brief Resolves a file path and opens it in read-only mode.
	 * @param fpath The path of the file to resolve.
	 * @return A shared pointer to a mapped.
	 */
	fun resolve_map(std::string_view fpath) -> sptr<mapped>;


	/**
	 * @brief Resolves a file path and opens it in read-write mode.
	 * @param fpath The path of the file to resolve.
	 * @return A shared pointer to an input/output stream.
	 */
	fun resolve_rw(std::string_view fpath) -> sptr<std::iostream>;




	/**
	 * @brief RAII wrapper for a VFS provider.
	 * 
	 * Automatically registers a VFS provider upon creation and unregisters/deletes it
	 * upon destruction. Ensures safe and automatic resource lifecycle management.
	 */
	struct vfs_raii
	{
		public:
			/**
			 * @brief Constructs the RAII wrapper and registers the provider.
			 * 
			 * @param protocol The protocol identifier to register (e.g., "file").
			 * @param prov Pointer to the newly allocated provider to take ownership of.
			 */
			vfs_raii(std::string protocol, vfs_provider *prov)
				: m_protocol(protocol)
				, m_prov(prov)
			{
				provider_reg(protocol, prov);
			}
			
			/**
			 * @brief Destructs the RAII wrapper, unregisters, and deletes the provider.
			 */
			~vfs_raii() {
				provider_del(m_protocol);
				delete m_prov;
			}

		
		private:
			std::string m_protocol{};
			vfs_provider *m_prov{};
	};

}




/**
 * @brief User-defined literal for resolving a file path in read-only mode.
 * @param fpath The file path std::string literal.
 * @param len The length of the std::string literal.
 * @return A shared pointer to an input stream.
 */
inline fun operator""_vfs_ro (const char *fpath, std::size_t len) -> sptr<std::istream>
{
	return vfs::resolve_ro(std::string_view(fpath, len));
}


inline fun operator""_vfs_map (const char *fpath, std::size_t len) -> sptr<vfs::mapped>
{
	return vfs::resolve_map(std::string_view(fpath, len));
}

/**
 * @brief User-defined literal for resolving a file path in read-write mode.
 * @param fpath The file path std::string literal.
 * @param len The length of the std::string literal.
 * @return A shared pointer to an input/output stream.
 */
inline fun operator""_vfs_rw (const char *fpath, std::size_t len) -> sptr<std::iostream>
{
	return vfs::resolve_rw(std::string_view(fpath, len));
}
