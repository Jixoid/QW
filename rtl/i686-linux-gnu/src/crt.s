.text
.globl main
.type main, @function
main:
	# Stack'i 16b ye hizala (4b dönüş adresi + 12b = 16b)
	subl $12, %esp

	# Girişi tetikle
	call qw_entry

	# Stack'i geri yükle
	addl $12, %esp

	# 0 ile Çıkış yap
	xorl %eax, %eax
	ret
.size main, .-main


.section .note.GNU-stack, "", @progbits
