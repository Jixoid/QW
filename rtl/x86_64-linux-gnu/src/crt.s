.text
.globl main
.type main, @function
main:
    # Stack'i 16b ye hizala
    subq $8, %rsp

    # Girişi tetkile
    call qw_entry@PLT

    # Stack'i geri yükle
    addq $8, %rsp

    # 0 ile Çıkış yap
    xorl %eax, %eax
    ret
.size main, .-main


.section .note.GNU-stack, "", @progbits
