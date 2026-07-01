
#ifndef _GNU_SOURCE
	#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <string.h>
#include <stddef.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>
#include <stdbool.h>
#include <sys/mman.h>
#include <sys/types.h>




#pragma region sys::exit

// sys::exit::panic
void __attribute__((weak, noreturn)) qwrtl_exit_panic(const char* message) {
  fprintf(stderr, "\nQW RUNTIME PANIC\n");
  
	if (message)
    fprintf(stderr, "message: %s\n", message);
	else
    fprintf(stderr, "unknown panic\n");
  
	abort();
}

// sys::exit::exit
void __attribute__((weak, noreturn)) qwrtl_exit_exit(int status) {
  exit(status);
}

#pragma endregion



#pragma region sys::io

// sys::io::write
ssize_t __attribute__((weak)) qwrtl_io_write(int fd, void* ptr, size_t size) {
  ssize_t ret = write(fd, ptr, size);

	return ret < 0 ? -1 : ret;
}

// sys::io::read
ssize_t __attribute__((weak)) qwrtl_io_read(int fd, void* ptr, size_t size) {
	ssize_t ret = read(fd, ptr, size);

	return ret < 0 ? -1 : ret;
}

#pragma endregion



#pragma region sys::signal

// sys::signal::hook
bool __attribute__((weak)) qwrtl_signal_hook(int sig, void (*handler)(int)) {
	return signal(sig, handler) != SIG_ERR;
}

#pragma endregion



#pragma region sys::heap

// sys::heap::alloc
void* __attribute__((weak)) qwrtl_heap_alloc(size_t align, size_t size) {
  return aligned_alloc(align, size);
}

// sys::heap::dispose
bool __attribute__((weak)) qwrtl_heap_dispose(void* ptr, size_t align, size_t size) {
  free_aligned_sized(ptr, align, size);
  return true;
}

// sys::heap::realloc
void* __attribute__((weak)) qwrtl_heap_realloc(void* ptr, size_t align, size_t old_size, size_t new_size) {
	void* new_ptr = aligned_alloc(align, new_size);
	if (!new_ptr)
		return NULL;

	size_t copy_size = (old_size < new_size) ? old_size : new_size;
	memcpy(new_ptr, ptr, copy_size);

	free_aligned_sized(ptr, align, old_size); 

	return new_ptr;
}

#pragma endregion



#pragma region sys::page

// sys::page::prot::* for linux
enum Prot: int {
  None  = 0,
  Read  = 0x1,
  Write = 0x2,
  Exec  = 0x4,
};

// sys::page::flag::* for linux
enum Flag: int {
  File      =	0,
  Private   =	0x02,
  Shared    =	0x03,
  Droppable =	0x08,
  Fixed     =	0x10,
  Anonymous =	0x20,
	Give32bit = 0x40,
	Locked    =	0x2000,
	Populate	= 0x8000,
};



// sys::page::alloc
void* __attribute__((weak)) qwrtl_page_alloc(size_t size, int prot, int flag) {
  void* ret = mmap(
    NULL,
    size,
    prot,
    flag,
    -1,
    0
  );

  return ret == MAP_FAILED ? NULL : ret;
}

// sys::page::dispose
bool __attribute__((weak)) qwrtl_page_dispose(void* ptr, size_t size) {
  return munmap(ptr, size) == 0;
}

// sys::page::realloc
void* __attribute__((weak)) qwrtl_page_realloc(void* ptr, size_t old_size, size_t new_size) {
	void* ret = mremap(
		ptr,
		old_size, new_size,
		MREMAP_MAYMOVE
	);
	return ret == MAP_FAILED ? NULL : ret;
}

// sys::page::protect
bool __attribute__((weak)) qwrtl_page_protect(void* ptr, size_t size, int prot) {
	return mprotect(
		ptr,
		size,
		prot
	) == 0;
}

#pragma endregion



#pragma region entry

int qw_entry();

int __argc = 0;
char** __argv = NULL;
char** __envp;


int main(int argc, char** argv, char** envp) {
	__argc = argc;
	__argv = argv;
	__envp = envp;

	return qw_entry();
}

#pragma endregion



#pragma region sys::args

// sys::args::count
int __attribute__((weak)) qwrtl_args_count() {
	return __argc;
}

// sys::args::get
char* __attribute__((weak)) qwrtl_args_get(int index) {
	return __argv[index];
}

#pragma endregion



#pragma region sys::env

// sys::env::get
char* __attribute__((weak)) qwrtl_env_get(char* name) {
  size_t len = strlen(name);
  for (int i = 0; __envp[i] != NULL; i++) {
    if (strncmp(__envp[i], name, len) == 0 && __envp[i][len] == '=') {
      return __envp[i] + len + 1;
    }
  }
  return NULL;
}

#pragma endregion

