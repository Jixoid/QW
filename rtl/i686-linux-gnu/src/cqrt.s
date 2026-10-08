.text
.globl _start
.type _start, @function
_start:
	# Frame pointer'ı sıfırla (GDB standardı için)
	xorl %ebp, %ebp

	# Stack'ten argc'yi çek (%esi) ve argv başlangıcını al (%ecx)
	popl %esi
	movl %esp, %ecx

	# Stack'i 16-byte hizala
	andl $-16, %esp

	/* Glibc __libc_start_main argümanlarını yığına yerleştir (sağdan sola):
		stack_end (%esp)
		rtld_fini (%edx)
		fini      (NULL, 0)
		init      (NULL, 0)
		argv      (%ecx)
		argc      (%esi)
		main      (main_stub)
	   Toplam 7 argüman = 28 bayt. 16-byte hizayı korumak için 4 bayt dolgu (padding) eklenir:
	   28 + 4 = 32 bayt.
	*/
	pushl %eax       # 16-byte hizalama dolgusu

	pushl %esp       # stack_end
	pushl %edx       # rtld_fini (dinamik bağlayıcı)
	pushl $0         # fini = NULL
	pushl $0         # init = NULL
	pushl %ecx       # argv
	pushl %esi       # argc
	pushl $main_stub # main fonksiyonu

	# Glibc başlatıcısını çağır
	call __libc_start_main

	# Güvenlik mekanizması: __libc_start_main asla geri dönmemeli
	hlt
.size _start, .-_start


.globl main_stub
.type main_stub, @function
main_stub:
	# Stack'i 16-byte hizala
	subl $12, %esp

	# Girişi tetikle
	call qw_entry

	# Stack'i geri yükle
	addl $12, %esp

	# Çıkış kodunu 0 olarak ayarla
	xorl %eax, %eax
	ret
.size main_stub, .-main_stub


.section .note.GNU-stack, "", @progbits
