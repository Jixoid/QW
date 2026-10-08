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

	# Tam hazır olmasa bile girişi tetikle (cdecl)
	call qw_entry

	# Stack'i geri yükle (cdecl caller cleanup)
	addq $8, %rsp

	# 0 ile çıkış yap (kernel32.dll ExitProcess(0))
	subq $40, %rsp
	xorl %ecx, %ecx
	call ExitProcess

	# Güvenlik mekanizması
	addq $40, %rsp
	ret
