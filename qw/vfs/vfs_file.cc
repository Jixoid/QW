/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <filesystem>
#include <fstream>
#include <iostream>
#include <istream>
#include <stdexcept>
#include <string>
#include <string_view>
#include "qw/basis.hh"
#include "qw/vfs/vfs.hh"

#if defined(__unix__) || defined(__APPLE__)
	#include <unistd.h>
  #include <fcntl.h>
  #include <sys/stat.h>
  #include <sys/mman.h>
#elif defined(_WIN32)
  #ifndef WIN32_LEAN_AND_MEAN
  #define WIN32_LEAN_AND_MEAN
  #endif
  #include <windows.h>
#endif



namespace vfs::__file
{

	struct mapped__file: mapped
  {
    public:
      explicit mapped__file(std::string fpath);
      ~mapped__file();

    private:
      #if defined(__unix__) || defined(__APPLE__)
      int fd{-1};
      #elif defined(_WIN32)
      void* hFile;
      void* hMapping;
      #endif

  };


	#if defined(__unix__) || defined(__APPLE__)

	mapped__file::mapped__file(std::string fpath)
	{
		m_data = (void*)(-1);

		if (!std::filesystem::is_regular_file(fpath))
			throw std::runtime_error("file not found: "+fpath+".");

		fd = open(fpath.c_str(), O_RDONLY);
		if (fd == -1)
			throw std::runtime_error("System error: " + std::to_string(errno));

		struct stat st;
		fstat(fd, &st);
		m_size = st.st_size;
		
		m_data = mmap(nil, m_size, PROT_READ, MAP_PRIVATE, fd, 0);
		if (m_data == MAP_FAILED)
			throw std::runtime_error("System error: " + std::to_string(errno));
	}

	mapped__file::~mapped__file() {
		if (m_data != MAP_FAILED) munmap(m_data, m_size);
		if (fd != -1) close(fd);
	}

	#elif defined(_WIN32)

	mappedFile::mappedFile(std::string fpath)
	{
		if (!std::filesystem::is_regular_file(fpath))
			throw std::runtime_error("file not found: "+fpath+".");
		
		hFile = CreateFileA(
			std::string(fpath).c_str(), 
			GENERIC_READ, 
			FILE_SHARE_READ, 
			NULL, 
			OPEN_EXISTING, 
			FILE_ATTRIBUTE_NORMAL, 
			NULL
		);

		if (hFile == INVALID_HANDLE_VALUE)
			throw std::runtime_error("Could not open shader file.");

		LARGE_INTEGER fileSize;
		if (!GetFileSizeEx(hFile, &fileSize)) {
			CloseHandle(hFile);
			throw std::runtime_error("Could not get file size.");
		}
		m_size = static_cast<size_t>(fileSize.QuadPart);

		hMapping = CreateFileMapping(hFile, NULL, PAGE_READONLY, 0, 0, NULL);
		if (hMapping == NULL) {
			CloseHandle(hFile);
			throw std::runtime_error("Could not create shader file mapping object.");
		}

		m_data = MapViewOfFile(hMapping, FILE_MAP_READ, 0, 0, 0);
		if (m_data == NULL) {
			CloseHandle(hMapping);
			CloseHandle(hFile);
			throw std::runtime_error("Could not map shader file to memory.");
		}
	}

	mappedFile::~mappedFile() {
		UnmapViewOfFile(m_data);
		CloseHandle(hMapping);
		CloseHandle(hFile);
	}

	#endif



	fun get() -> vfs_raii
	{
		return vfs_raii("file", new vfs_provider{
			.init =	[]() -> void* {return nil; },
			.fini = [](void* _Data) {},

			.open_ro = [](void*, std::string_view fpath) -> sptr<std::istream>
			{
				return make_sptr(new std::ifstream(std::string(fpath)));
			},

			.open_map = [](void*, std::string_view fpath) -> sptr<mapped>
			{
				return make_sptr(new mapped__file(std::string(fpath)));
			},

			.open_rw = [](void*, std::string_view fpath) -> sptr<std::iostream>
			{
				return make_sptr(new std::fstream(std::string(fpath)));
			}
		});
	}

}
