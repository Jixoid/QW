.text
.globl _start
.type _start, @function
_start:
	// Frame pointer ve Link Register'ı sıfırla (GDB standardı için)
	mov x29, #0
	mov x30, #0

	// Tam hazır olmasa bile girişi tetikle
	bl qw_entry

	// 0 ile çıkış yap (AArch64 Linux __NR_exit_group = 94, kod = 0)
	mov x0, #0
	mov x8, #94
	svc #0
.size _start, .-_start


.section .note.GNU-stack, "", @progbits
