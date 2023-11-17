
; 0x512 = color
mov [0x512], 0x0000FF00

main:
    .reloop:

    ; 0x501 = screen size
    mov r0, [0x2038]
    mul r0, [0x203C]
    mov [0x501], r0

    ; 0x502 = vram address
    mov r0, [0x2034]
    mov [0x502], r0

    ; 0x500 = counter
    mov [0x500], 0

    ; check if red
    cmp [0x512], 0xFF000000
    and fl, 0b0100
    jz .is_not_red, fl
    mov [0x512], 0x00FF0000
    jmp .loop

    ; check if green
    .is_not_red:
    cmp [0x512], 0x00FF0000
    and fl, 0b0100
    jz .is_not_green, fl
    mov [0x512], 0x0000FF00
    jmp .loop

    ; check if blue
    .is_not_green:
    cmp [0x512], 0x0000FF00
    and fl, 0b0100
    jz .loop, fl
    mov [0x512], 0xFF000000

    .loop:
        mov r1, [0x512]
        mov r0, [0x500]
        add r0, [0x502]
        mov [r0], r1
        add [0x500], 1
        sub [0x501], 1
        mov r0, [0x501]
        jnz .loop, r0
        jmp .reloop
