
; 0x1512 = color
mov [0x1512], 0x00008800
; current buffer
; [0x1520]
; buffer counter
; [0x1521]

buffer_one:
    mov [0x2034], 12000
    mov [0x1520], 23000
    mov [0x1521], 1
    jmp main

buffer_two:
    mov [0x2034], 23000
    mov [0x1520], 12000
    mov [0x1521], 0
    jmp main

main:
    ; 0x1501 = screen size
    mov r0, [0x2038]
    mul r0, [0x203C]
    mov [0x1501], r0

    ; 0x1502 = vram address
    mov r0, [0x1520]
    mov [0x1502], r0

    ; 0x1500 = counter
    mov [0x1500], 0

    ; check if red
    cmp [0x1512], 0x88000000
    and fl, 0b0100
    jz .is_not_red, fl
    mov [0x1512], 0x00880000
    jmp .loop

    ; check if green
    .is_not_red:
    cmp [0x1512], 0x00880000
    and fl, 0b0100
    jz .is_not_green, fl
    mov [0x1512], 0x00008800
    jmp .loop

    ; check if blue
    .is_not_green:
    cmp [0x1512], 0x00008800
    and fl, 0b0100
    jz .loop, fl
    mov [0x1512], 0x88000000

    .loop:

        mov r1, [0x1512]
        mov r0, [0x1500]
        add r0, [0x1502]
        mov [r0], r1

        add [0x1500], 1
        sub [0x1501], 1

        mov r0, [0x1501]
        jnz .loop, r0
        mov r0, [0x1521]
        jz buffer_one, r0
        jmp buffer_two
