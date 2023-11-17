
main:
    ; 0x500 = counter
    mov [0x400], 0 ; star offset

    .draw:
        mov [0x500], 0

        ; 0x501 = screen size
        mov r0, [0x2038]
        mul r0, [0x203C]
        mov [0x501], r0

        ; 0x502 = vram address
        mov r0, [0x2034]
        mov [0x502], r0

        .loop:
            mov r0, [0x500]
            add r0, [0x502]
            mov r1, r0
            add r1, [0x400]
            rem r1, 0x100
            mov [r0], 0xCCCCCC00
            jz .skip, r1
            mov [r0], r0
            .skip:
                add [0x500], 1
                sub [0x501], 1
                mov r0, [0x501]
                jnz .loop, r0
                mov r0, [0x2038]
                add [0x400], r0
                jmp .draw
