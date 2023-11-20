

%define COLOR 0x1514
%define CURRENT_BUFFER 0x1520
%define BUFFER_COUNTER 0x1510
%define WORKING_ON_INDEX 0x1500
%define SCREEN_SIZE 0x1400
%define WORKING_ON_ADDRESS 0x1450

%define V_ADDR_0 100000
%define V_ADDR_1 48000

prepare:
    mov [COLOR], 0x00008800

buffer_one:
    mov [0x2034], V_ADDR_0
    mov [CURRENT_BUFFER], V_ADDR_1
    mov [BUFFER_COUNTER], 1
    jmp main

buffer_two:
    mov [0x2034], V_ADDR_1
    mov [CURRENT_BUFFER], V_ADDR_0
    mov [BUFFER_COUNTER], 0
    jmp main

main:
    mov r0, [0x2038]
    mul r0, [0x203C]
    mov [SCREEN_SIZE], r0

    mov [WORKING_ON_INDEX], 0

    ; check if red
    cmp [COLOR], 0x88000000
    and fl, 0b0100
    jz .is_not_red, fl
    mov [COLOR], 0x00880000
    jmp .loop

    ; check if green
    .is_not_red:
    cmp [COLOR], 0x00880000
    and fl, 0b0100
    jz .is_not_green, fl
    mov [COLOR], 0x00008800
    jmp .loop

    ; check if blue
    .is_not_green:
    cmp [COLOR], 0x00008800
    and fl, 0b0100
    jz .loop, fl
    mov [COLOR], 0x88000000

    .loop:
        mov r1, [COLOR]
        mov r0, [WORKING_ON_INDEX]
        add r0, [CURRENT_BUFFER]
        mov [r0], r1

        add [WORKING_ON_INDEX], 4
        sub [SCREEN_SIZE], 1

        mov r0, [SCREEN_SIZE]
        jnz .loop, r0
        mov r0, [BUFFER_COUNTER]
        jz buffer_one, r0
        jmp buffer_two
