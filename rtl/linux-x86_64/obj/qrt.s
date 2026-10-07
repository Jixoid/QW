.text
.globl _start
.type _start, @function
_start:
	# Frame pointer'ı sıfırla (GDB standardı için)
	xorq %rbp, %rbp

	# Stack'i x86_64 ABI standardına göre 16-byte hizala
	andq $~15, %rsp

	# Tam hazır olmasa bile girişi tetikle
	call qw_entry@PLT

	# 0 ile çıkış yap
	movq $231, %rax
	xorl %edi, %edi
    syscall
.size _start, .-_start


.section .note.GNU-stack, "", @progbits
