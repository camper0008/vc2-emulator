add r0, 25
main:
    add r1, 1
    sub r0, 1
    jnz main, r0
jmp 0x500