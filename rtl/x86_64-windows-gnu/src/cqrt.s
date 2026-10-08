.text
.globl _start
.def _start; .scl 2; .type 32; .endef
.globl mainCRTStartup
.def mainCRTStartup; .scl 2; .type 32; .endef
_start:
mainCRTStartup:
	# Frame pointer'ı sıfırla (GDB standardı için)
	xorq %rbp, %rbp

	# Stack'i 16-byte hizala (cdecl)
	subq $8, %rsp

	# Girişi tetikle (cdecl)
	call qw_entry

	# Stack'i geri yükle (cdecl caller cleanup)
	addq $8, %rsp

	# msvcrt exit(0) ile temiz çıkış yap
	subq $40, %rsp
	xorl %ecx, %ecx
	call exit

	# Güvenlik mekanizması
	addq $40, %rsp
	ret


.globl main_stub
.def main_stub; .scl 2; .type 32; .endef
main_stub:
	# Stack'i 16-byte hizala (cdecl)
	subq $8, %rsp

	# Girişi tetikle (cdecl)
	call qw_entry

	# Stack'i geri yükle (cdecl caller cleanup)
	addq $8, %rsp

	# 0 ile Çıkış yap
	xorl %eax, %eax
	ret
