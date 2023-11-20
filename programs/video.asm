%define star_offset 0x1000
%define i 0x1004
%define screen_size 0x1008

main:
    mov [star_offset], 0 ; star offset

    .draw:
        mov [i], 0

        mov r0, [0x2038]
        mul r0, [0x203C]
        mov [screen_size], r0

        .loop:
            mov r0, [i]
            add r0, [0x2034]
            mov r1, r0
            add r1, [star_offset]
            rem r1, 0x104
            mov [r0], 0xCCCCCC00
            jz .skip, r1
            mov [r0], r0
            .skip:
                add [i], 4
                sub [screen_size], 1
                mov r0, [screen_size]
                jnz .loop, r0
                mov r0, [0x2038]
                mul r0, 4
                add [star_offset], r0
                add [star_offset], 4
    .sleep:
        mov r1, 0x4FFFF
        .sleep_inner:
            sub r1, 1
            jnz .sleep_inner, r1
            jmp .draw
