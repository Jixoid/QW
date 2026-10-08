.text
.globl _start
.type _start, @function
_start:
	// Frame pointer ve Link Register'ı sıfırla (GDB / stack unwinding için)
	mov x29, #0
	mov x30, #0

	/* Glibc'nin aradığı argümanları ayarla (AAPCS64):
		x0: Çalıştırılacak ana fonksiyon (main_stub)
		x1: argc
		x2: argv
		x3: init fonksiyonu (Gerek yok, NULL)
		x4: fini fonksiyonu (Gerek yok, NULL)
		x5: rtld_fini (Çekirdek bunu x0 ile verir, korumalıyız!)
		x6: stack_end (sp)
	*/
	mov x5, x0          // rtld_fini adresini 6. argümana (x5) taşı
	ldr x1, [sp]        // Stack'ten argc'yi çek ve 2. argümana (x1) koy
	add x2, sp, #8      // argv başlangıcı: sp + 8 (3. argüman)
	mov x6, sp          // stack_end: sp (7. argüman)

	// Modern Glibc için init ve fini rutinlerini NULL (0) yapıyoruz
	mov x3, #0          // x3 = 0
	mov x4, #0          // x4 = 0

	// Glibc'ye ilk argüman (x0) olarak main_stub adresini ver
	adrp x0, main_stub
	add  x0, x0, :lo12:main_stub

	// Glibc başlatıcısını çağır. TLS ve I/O burada kurulacak!
	bl __libc_start_main

	// Güvenlik mekanizması: __libc_start_main asla geri dönmemeli
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
