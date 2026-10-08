.text
.globl main
.def main; .scl 2; .type 32; .endef
main:
	# Stack'i 16-byte hizala (cdecl)
	subq $8, %rsp

	# Girişi tetikle (cdecl)
	call qw_entry

	# Stack'i geri yükle (cdecl caller cleanup)
	addq $8, %rsp

	# 0 ile Çıkış yap
	xorl %eax, %eax
	ret
