.syntax unified
.text
.globl _start
.type _start, %function
_start:
	// Frame pointer ve Link Register'ı sıfırla
	mov fp, #0
	mov lr, #0

	// Çekirdeğin yığına koyduğu argc'yi r1'e al
	pop {r1}

	// argv başlangıcı pop sonrasındaki sp'dir (2. argüman: r2)
	mov r2, sp

	/* Dynamic linker r0 içinde rtld_fini işaretçisini verir.
	   AAPCS'e göre __libc_start_main parametreleri:
	   r0: main (main_stub)
	   r1: argc
	   r2: argv
	   r3: init (NULL, 0)
	   Yığında:
	     fini      (NULL, 0)
	     rtld_fini (orijinal r0)
	     stack_end (sp)
	*/
	mov ip, r0      // rtld_fini değerini ip (r12) içinde geçici sakla
	mov r3, #0      // init = 0

	// Stack'i 8-byte hizala
	and sp, sp, #-8

	// Yığına argümanları ve dolguyu yerleştir (16 bayt, 8-byte hizalı):
	mov r0, #0      // NULL
	push {r0}       // 8-byte hizalama dolgusu
	push {sp}       // stack_end
	push {ip}       // rtld_fini
	push {r0}       // fini = NULL

	// r0'a main_stub adresini yükle
	ldr r0, =main_stub

	// Glibc başlatıcısını çağır
	bl __libc_start_main

	// Güvenlik mekanizması: __libc_start_main asla geri dönmemeli
	bkpt #0
.size _start, .-_start


.globl main_stub
.type main_stub, %function
main_stub:
	// Frame pointer ve Link Register'ı kaydet (8-byte hizalı)
	push {fp, lr}
	mov fp, sp

	// Girişi tetikle
	bl qw_entry

	// Çıkış kodunu 0 olarak ayarla
	mov r0, #0
	pop {fp, pc}
.size main_stub, .-main_stub


.section .note.GNU-stack, "", %progbits
