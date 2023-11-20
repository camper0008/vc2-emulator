; program that displays a block moving like the dvd patterns

%define COUNTER 0x1000
%define X_VELOCITY 0x1004
%define X 0x1008
%define Y_VELOCITY 0x100C
%define Y 0x1010
%define SCREEN_SIZE 0x1014

main:
    mov [X_VELOCITY], 1
    ; &X = x
    mov [X], 0

    ; &Y_VELOCITY = y_velocity
    mov [Y_VELOCITY], 1

    ; &Y = y
    mov [Y], 0

    jmp game_tick

sleep:
    mov r0, 0xFFFFF
    .loop:
    sub r0, 1
    jnz .loop, r0
    jmp game_tick

clear_screen:
    ; COUNTER = counter
    mov [COUNTER], 0

    ; SCREEN_SIZE = screen size
    mov r0, [0x2038]
    imul r0, [0x203C]
    mov [SCREEN_SIZE], r0

    .loop:
        mov r0, [COUNTER]
        add r0, [0x2034]
        mov [r0], 0x00007700
        add [COUNTER], 4
        sub [SCREEN_SIZE], 1
        mov r0, [SCREEN_SIZE]
        jnz .loop, r0

        ; draw blob
        ;r0 = y
        mov r0, [Y]
        ;r0 *= SCREEN_WIDTH
        imul r0, [0x2038]
        ;r0 += x
        add r0, [X]
        ;r0 *= 4
        mul r0, 4
        ;r0 += VRAM_ADDR
        add r0, [0x2034]
        ;*r0 = 0
        mov [r0], 0xFFFFFFFF

    jmp sleep

invert_x_velocity:
    imul [X_VELOCITY], 0xFFFFFFFF
    jmp game_tick

invert_y_velocity:
    imul [Y_VELOCITY], 0xFFFFFFFF
    jmp game_tick

game_tick:
    ;x += x_velocity
    mov r0, [X_VELOCITY]
    add r0, [X]
    mov [X], r0
    ;y += y_velocity
    mov r0, [Y_VELOCITY]
    add r0, [Y]
    mov [Y], r0


    ; check x hit left wall
    cmp [X], 0
    ; if x < 0
    and fl, 0b1000
    jnz invert_x_velocity, fl

    ; check x hit right wall
    mov r0, [0x2038]
    cmp [X], r0
    ; if !(x < SCREEN_WIDTH)
    and fl, 0b1000
    jz invert_x_velocity, fl

    ; check y hit top wall
    cmp [Y], 0
    ; if y < 0
    and fl, 0b1000
    jnz invert_y_velocity, fl

    ; check y hit bottom wall
    mov r0, [0x203C]
    cmp [Y], r0
    ; if !(y < SCREEN_HEIGHT)
    and fl, 0b1000
    jz invert_y_velocity, fl

    jmp clear_screen
