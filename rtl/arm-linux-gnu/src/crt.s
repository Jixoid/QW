.syntax unified
.text
.globl main
.type main, %function
main:
	// Frame pointer ve Link Register'ı kaydet (8-byte hizalı)
	push {fp, lr}
	mov fp, sp

	// Girişi tetikle
	bl qw_entry

	// 0 ile çıkış yap
	mov r0, #0

	// Frame pointer'ı geri yükle ve dön
	pop {fp, pc}
.size main, .-main


.section .note.GNU-stack, "", %progbits
