/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <filesystem>
#include <iostream>
#include <istream>
#include <string>
#include <string_view>
#include <spanstream>
#include "qw/basis.hh"
#include "qw/vfs/vfs.hh"
#include "qw/vfs/vfs_embed.hh"

#if defined(_WIN32)
  #ifndef WIN32_LEAN_AND_MEAN
  #define WIN32_LEAN_AND_MEAN
  #endif
  #include <windows.h>
#else
  #include <dlfcn.h>
#endif



namespace vfs::__embed
{

	struct mapped__embed: mapped
	{
		public:
			mapped__embed(void* data, u0 size)
			{
				m_data = data;
				m_size = size;
			}

	};



	fun get() -> vfs_raii
	{
		return vfs_raii("embed", new vfs_provider{
			.init =[]() -> void* {return nil; },
			.fini = [](void* _Data) {},

			.open_ro = [](void*, std::string_view _fpath) -> sptr<std::istream>
			{
				std::string fpath(_fpath);
				for (char &C: fpath)
					if (C == '.' | C == '/' | C == '-')
						C = '_';

				void *Sym_Beg{}, *Sym_End{};

				// Platforma özgü sembol yükleme
				#if defined(_WIN32)
						HMODULE hMod = GetModuleHandle(NULL);
					Sym_Beg = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_start").c_str());
					Sym_End = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_end").c_str());
				#else
					Sym_Beg = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_start").c_str());
					Sym_End = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_end").c_str());
				#endif

				if (!Sym_Beg || !Sym_End)
					throw std::filesystem::filesystem_error("The embedded file could not be opened.", _fpath, std::make_error_code(std::errc::no_such_file_or_directory));

				auto Ret = new std::ispanstream(std::span<char>((char*)Sym_Beg, (u0)(Sym_End) - (u0)(Sym_Beg)));
				return sptr<std::istream>(Ret);
			},

			.open_map = [](void*, std::string_view _fpath) -> sptr<mapped>
			{
				std::string fpath(_fpath);
				for (char &C: fpath)
					if (C == '.' | C == '/' | C == '-')
						C = '_';

				void *Sym_Beg{}, *Sym_End{};

				// Platforma özgü sembol yükleme
				#if defined(_WIN32)
						HMODULE hMod = GetModuleHandle(NULL);
					Sym_Beg = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_start").c_str());
					Sym_End = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_end").c_str());
				#else
					Sym_Beg = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_start").c_str());
					Sym_End = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_end").c_str());
				#endif

				if (!Sym_Beg || !Sym_End)
					throw std::filesystem::filesystem_error("The embedded file could not be mapped.", _fpath, std::make_error_code(std::errc::no_such_file_or_directory));

				return make_sptr<mapped__embed>(Sym_Beg, (u0)(Sym_End) - (u0)(Sym_Beg));
			},

			.open_rw = [](void*, std::string_view fpath) -> sptr<std::iostream>
			{
				throw std::filesystem::filesystem_error("The embedded file cannot be opened with RW", fpath, std::make_error_code(std::errc::read_only_file_system));
			},
		});
	}

}
