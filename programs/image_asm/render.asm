main:
    mov [0x1500], abs_end
    mov r0, [0x203C]
    add r0, 48 ; 12 * 4
    mul r0, [0x2038]
    mov r1, [0x2038]
    mul r1, 2
    sub r1, 32 ; 8 * 4
    add r0, r1

    jmp card

end:
    jmp 0xFFFFFFFF
