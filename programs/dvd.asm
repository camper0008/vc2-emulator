; dvd logo

main:
    ; &700 = x_velocity
    mov [700], 1
    ; &705 = x
    mov [705], 5

    ; &710 = y_velocity
    mov [710], 1

    ; &715 = y
    mov [715], 50

    jmp game_tick

sleep:
    mov r0, 0xFFFFF
    .loop:
    sub r0, 1
    jnz .loop, r0
    jmp game_tick

clear_screen:
    ; 0x500 = counter
    mov [0x500], 0

    ; 0x501 = screen size
    mov r0, [0x2038]
    imul r0, [0x203C]
    mov [0x501], r0

    ; 0x502 = vram address
    mov r0, [0x2034]
    mov [0x502], r0

    .loop:
        mov r0, [0x500]
        add r0, [0x502]
        mov [r0], 0x00007700
        add [0x500], 1
        sub [0x501], 1
        mov r0, [0x501]
        jnz .loop, r0
        ; draw blob
        ;r0 = y
        mov r0, [715]
        ;r0 *= SCREEN_WIDTH
        imul r0, [0x2038]
        ;r0 += x
        add r0, [705]
        ;r0 += VRAM_ADDR
        add r0, [0x2034]
        ;*r0 = 0
        mov [r0], 0xFFFFFFFF

    jmp sleep

invert_x_velocity:
    imul [700], 0xFFFFFFFF
    jmp game_tick

invert_y_velocity:
    imul [710], 0xFFFFFFFF
    jmp game_tick

game_tick:

    ;x += x_velocity
    mov r0, [700]
    add r0, [705]
    mov [705], r0
    ;y += y_velocity
    mov r0, [710]
    add r0, [715]
    mov [715], r0


    ; check x hit left wall
    cmp [705], 0
    ; if x <= 0
    and fl, 0b1100
    jnz invert_x_velocity, fl

    ; check x hit right wall
    mov r0, [0x2038]
    cmp [705], r0
    ; if !(x <= SCREEN_WIDTH)
    and fl, 0b1000
    jz invert_x_velocity, fl

    ; check y hit left wall
    cmp [715], 0
    ; if y <= 0
    and fl, 0b1100
    jnz invert_y_velocity, fl

    ; check y hit right wall
    mov r0, [0x203C]
    cmp [715], r0
    ; if !(y <= SCREEN_HEIGHT)
    and fl, 0b1000
    jz invert_y_velocity, fl

    jmp clear_screen
