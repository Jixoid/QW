/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include "qw/vfs/vfs.hh"
#include <filesystem>
#include <istream>
#include <string>
#include <string_view>
#include <unordered_map>
#include <utility>



namespace vfs
{
	std::unordered_map<std::string, std::pair<vfs_provider*, void*>> providers;
	std::shared_mutex providers_mtx;

	

  fun provider_reg(std::string protocol, vfs_provider *provider) -> void
  {
    void* data = provider->init();

		ulock lock(providers_mtx);
    providers[protocol] = {provider, data};
  }

	fun provider_del(std::string protocol) -> void
  {
		ulock lock(providers_mtx);
    if (auto X = providers.find(protocol); X != providers.end()) {
      X->second.first->fini(X->second.second);

      providers.erase(X);
    }
  }



  fun resolve_ro(std::string_view fpath) -> sptr<std::istream>
	{
		std::string_view payload;
		std::string protocol;

		auto splitPos = fpath.find("://", 1);


		if (splitPos != std::string_view::npos) [[likely]] {
			protocol = fpath.substr(0, splitPos);
			payload  = fpath.substr(splitPos+3);
		}

		else [[unlikely]] {
			protocol = "file";
			payload  = fpath;
		}


		slock lock(providers_mtx);
		if (auto X = vfs::providers.find(protocol); X != vfs::providers.end())
		{
			if (X->second.first->open_ro)
				return X->second.first->open_ro(X->second.second, payload);
			else
				throw std::filesystem::filesystem_error("Virtual file could not be opened", "open_ro", std::make_error_code(std::errc::protocol_not_supported));
		}
		else
			throw std::filesystem::filesystem_error("Virtual file could not be opened", protocol, std::make_error_code(std::errc::protocol_not_supported));
	}

	fun resolve_map(std::string_view fpath) -> sptr<mapped>
	{
		std::string_view payload;
		std::string protocol;

		auto splitPos = fpath.find("://", 1);


		if (splitPos != std::string_view::npos) [[likely]] {
			protocol = fpath.substr(0, splitPos);
			payload  = fpath.substr(splitPos+3);
		}

		else [[unlikely]] {
			protocol = "file";
			payload  = fpath;
		}


		slock lock(providers_mtx);
		if (auto X = vfs::providers.find(protocol); X != vfs::providers.end())
		{
			if (X->second.first->open_map)
				return X->second.first->open_map(X->second.second, payload);
			else
				throw std::filesystem::filesystem_error("Virtual file could not be opened", "open_map", std::make_error_code(std::errc::protocol_not_supported));
		}
		else
			throw std::filesystem::filesystem_error("Virtual file could not be opened", protocol, std::make_error_code(std::errc::protocol_not_supported));
	}

  fun resolve_rw(std::string_view fpath) -> sptr<std::iostream>
	{
		std::string_view payload;
		std::string protocol;

		auto splitPos = fpath.find("://", 1);


		if (splitPos != std::string_view::npos) [[likely]] {
			protocol = fpath.substr(0, splitPos);
			payload  = fpath.substr(splitPos+3);
		}

		else [[unlikely]] {
			protocol = "file";
			payload  = fpath;
		}

		slock lock(providers_mtx);
		if (auto X = vfs::providers.find(protocol); X != vfs::providers.end())
		{
			if (X->second.first->open_rw)
				return X->second.first->open_rw(X->second.second, payload);
			else
				throw std::filesystem::filesystem_error("Virtual file could not be opened", "open_rw", std::make_error_code(std::errc::protocol_not_supported));
		}
		else
			throw std::filesystem::filesystem_error("Virtual file could not be opened", protocol, std::make_error_code(std::errc::protocol_not_supported));
	}

}
