.text
.globl main
.type main, @function
main:
	// Frame pointer ve Return Address'i kaydet (16-byte hizalı)
	stp x29, x30, [sp, #-16]!
	mov x29, sp

	// Girişi tetikle
	bl qw_entry

	// Frame pointer ve Return Address'i geri yükle
	ldp x29, x30, [sp], #16

	// 0 ile Çıkış yap
	mov w0, #0
	ret
.size main, .-main


.section .note.GNU-stack, "", @progbits
