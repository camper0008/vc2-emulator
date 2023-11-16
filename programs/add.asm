add r0, 25
main:
    sub r0, 1
    jmp .L0
    add r0, 1
    .L0:
        add r1, 1
        jnz main, r0
jmpabs 0x3FFF