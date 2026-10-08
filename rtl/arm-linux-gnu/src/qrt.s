.syntax unified
.text
.globl _start
.type _start, %function
_start:
	// Frame pointer ve Link Register'ı sıfırla (GDB standardı için)
	mov fp, #0
	mov lr, #0

	// Tam hazır olmasa bile girişi tetikle
	bl qw_entry

	// 0 ile çıkış yap (Linux ARM EABI __NR_exit_group = 248, durum = 0)
	mov r0, #0
	mov r7, #248
	svc #0
.size _start, .-_start


.section .note.GNU-stack, "", %progbits
