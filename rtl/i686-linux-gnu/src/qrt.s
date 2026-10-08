.text
.globl _start
.type _start, @function
_start:
	# Frame pointer'ı sıfırla (GDB standardı için)
	xorl %ebp, %ebp

	# Stack'i 16-byte hizala
	andl $-16, %esp

	# qw_entry çağrısı öncesi yığını 16b hizala
	subl $12, %esp

	# Tam hazır olmasa bile girişi tetikle
	call qw_entry

	# 0 ile çıkış yap (Linux i386 __NR_exit_group = 252, durum = 0)
	movl $252, %eax
	xorl %ebx, %ebx
	int $0x80
.size _start, .-_start


.section .note.GNU-stack, "", @progbits
