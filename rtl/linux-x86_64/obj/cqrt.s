.text
.globl _start
.type _start, @function
_start:
	/* Frame pointer'ı sıfırla (GDB standardı için) */
	xorq %rbp, %rbp

	/* Glibc'nin aradığı argümanları ayarla:
		%rdi: Çalıştırılacak ana fonksiyon (main_stub)
		%rsi: argc
		%rdx: argv
		%rcx: init fonksiyonu (Gerek yok, NULL)
		%r8 : fini fonksiyonu (Gerek yok, NULL)
		%r9 : rtld_fini (Çekirdek bunu %rdx ile verir, korumalıyız!)
	*/
	movq %rdx, %r9   /* rtld_fini adresini 6. argümana (%r9) taşı */
	popq %rsi        /* Stack'ten argc'yi çek ve 2. argümana (%rsi) koy */
	movq %rsp, %rdx  /* pop sonrası %rsp artık argv'yi gösteriyor. 3. argümana (%rdx) koy */

	/* Modern Glibc için init ve fini rutinlerini NULL (0) yapıyoruz */
	xorq %rcx, %rcx  /* %rcx = 0 */
	xorq %r8, %r8    /* %r8  = 0 */

	/* Stack'i x86_64 ABI standardına göre 16-byte hizala */
	andq $~15, %rsp

	pushq %rax  /* byte çöp veri atarak hizalamayı boz (bir sonraki push eşitleyecek) */
	pushq %rsp  /* argüman: stack_end (Glibc yığın sonunu bilmek ister) */

	/* Glibc'ye ilk argüman (%rdi) olarak bizim main_stub adresini ver */
	leaq main_stub(%rip), %rdi

	/* Glibc başlatıcısını çağır. TLS ve I/O burada kurulacak! */
	call __libc_start_main@PLT

	/* Güvenlik mekanizması: __libc_start_main asla geri dönmemeli */
	hlt
.size _start, .-_start


.globl main_stub
.type main_stub, @function
main_stub:
	/* Glibc bizi çağırdığında stack'te geri dönüş adresi (8 byte) olduğu için
			hizalama bozuktur. Çağrı yapmadan önce stack'i 16-byte hizasına çekiyoruz. */
	subq $8, %rsp

	/* Bu aşamada TLS tamamen kurulu ve güvenlidir, asla patlamaz! */
	call qw_entry@PLT

	/* Stack'i eski haline getir */
	addq $8, %rsp

	/* Çıkış kodunu 0 (başarılı) olarak ayarla. 
			Glibc bu dönüş değerini alıp otomatik olarak exit(0) yapacaktır. */
	xorl %eax, %eax
	ret
.size main_stub, .-main_stub


.section .note.GNU-stack, "", @progbits
