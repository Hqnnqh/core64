bits 64

extern kernel_main ; rust main

; kernel entry setup

section .text
    global _start
    _start:
      jmp kernel_main
