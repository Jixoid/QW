.text
.globl _start
.type _start, @function
_start:
	// Frame pointer ve Link Register'ı sıfırla (GDB / unwinding için)
	mov x29, #0
	mov x30, #0

	// raw_args olarak çekirdeğin verdiği yığın işaretçisini ayarla
	mov x0, sp

	// Yığında structors_array_t için yer aç (3 pointer = 24 bayt, 16-bayt hizalı 32 bayt)
	sub sp, sp, #32

	// structors_array_t alanlarını NULL (0) yap
	stp xzr, xzr, [sp]
	str xzr, [sp, #16]

	// Bionic __libc_init argümanları (AArch64 AAPCS):
	// x0: raw_args (kernel sp)
	// x1: onexit = NULL (0)
	mov x1, #0

	// x2: slingshot (main_stub adresi)
	adrp x2, main_stub
	add  x2, x2, :lo12:main_stub

	// x3: structors array işaretçisi
	mov x3, sp

	// Bionic libc başlatıcısını çağır
	bl __libc_init

	// Güvenlik mekanizması: __libc_init asla geri dönmemeli
	hlt #0
.size _start, .-_start


.globl main_stub
.type main_stub, @function
main_stub:
	// Frame pointer ve Return Address'i kaydet (16-byte hizalı)
	stp x29, x30, [sp, #-16]!
	mov x29, sp

	// Girişi tetikle
	bl qw_entry

	// Frame pointer ve Return Address'i geri yükle
	ldp x29, x30, [sp], #16

	// Çıkış kodunu 0 (başarılı) olarak ayarla
	mov w0, #0
	ret
.size main_stub, .-main_stub


.section .note.GNU-stack, "", @progbits
