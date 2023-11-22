

; vram
%define VRAM 0x2034
%define VRAM_START_INDEX 0xDB80

%define KEY_EVENT_TYPE 0x2024
%define KEY_CODE 0x2028
%define KEY_CALLBACK 0x202C
%define SCREEN_WIDTH 0x2038
%define SCREEN_HEIGHT 0x203C

; user defined
%define CURSOR 0x1000
%define SELECTED_COLOR 0x1004
%define SELECTED_COLOR_INDEX 0x1008
%define PAINT_BUFFER_INDEX 0x100C
%define PAINT_BUFFER_END_INDEX 0x1010

; paint buffer
%define PAINT_BUFFER_START_INDEX 0x4EE0

; masks
%define cmp_equal 0b100
%define cmp_less_signed 0b1000

jmp prepare

key_event_happened:
    mov r0, [KEY_EVENT_TYPE]
    jz wait_for_keypress, r0

    ; check keycode
    cmp [KEY_CODE], 82
    and fl, cmp_equal
    jnz .key_up, fl

    cmp [KEY_CODE], 81
    and fl, cmp_equal
    jnz .key_down, fl

    cmp [KEY_CODE], 80
    and fl, cmp_equal
    jnz .key_left, fl

    cmp [KEY_CODE], 79
    and fl, cmp_equal
    jnz .key_right, fl

    cmp [KEY_CODE], 20
    and fl, cmp_equal
    jnz .key_q, fl

    cmp [KEY_CODE], 8
    and fl, cmp_equal
    jnz .key_e, fl

    cmp [KEY_CODE], 44
    and fl, cmp_equal
    jnz .key_space, fl

    ; irrelevant key was pressed
    jmp wait_for_keypress
    
    .key_up:
        mov r0, [SCREEN_WIDTH]
        mul r0, 2
        sub [CURSOR], r0

        cmp [CURSOR], PAINT_BUFFER_START_INDEX
        and fl, cmp_less_signed
        jz render_paint_buffer, fl
        mul r0, [SCREEN_HEIGHT]
        div r0, 2
        add [CURSOR], r0

        jmp render_paint_buffer
    .key_left:
        mov r0, [CURSOR]
        sub r0, PAINT_BUFFER_START_INDEX
        mov r1, [SCREEN_WIDTH]
        mul r1, 2
        rem r0, r1
        jz .left_at_edge, r0
        sub [CURSOR], 4
        jmp render_paint_buffer
        .left_at_edge:
        mov r0, [SCREEN_WIDTH]
        mul r0, 2
        sub r0, 4
        add [CURSOR], r0
        jmp render_paint_buffer
    .key_right:
        add [CURSOR], 4

        mov r0, [CURSOR]
        sub r0, PAINT_BUFFER_START_INDEX
        mov r1, [SCREEN_WIDTH]
        mul r1, 2
        rem r0, r1

        jnz .right_not_at_edge, r0
        jmp .key_up
        .right_not_at_edge:
        jmp render_paint_buffer
    .key_down:
        mov r0, [SCREEN_WIDTH]
        mul r0, 2
        add [CURSOR], r0

        mov r1, [PAINT_BUFFER_END_INDEX]
        cmp [CURSOR], r1
        and fl, cmp_less_signed
        jnz render_paint_buffer, fl
        mul r0, [SCREEN_HEIGHT]
        div r0, 2
        sub [CURSOR], r0

        jmp render_paint_buffer
    .key_q:
        jmp previous_selected_color
    .key_e:
        jmp next_selected_color
    .key_space:
        mov r0, [CURSOR]
        mov r1, [SELECTED_COLOR]
        mov [r0], r1
        jmp render_paint_buffer

wait_for_keypress:
    hlt

prepare:
    mov [KEY_CALLBACK], key_event_happened
    ; set defaults
    mov [SELECTED_COLOR], 0xFF000000
    mov [CURSOR], PAINT_BUFFER_START_INDEX
    mov [PAINT_BUFFER_INDEX], PAINT_BUFFER_START_INDEX
    mov [VRAM], VRAM_START_INDEX

    ; get end of paint buffer (half of screen size on each axis)
    .calculate_buffer_end:
        mov r1, [SCREEN_WIDTH]
        mul r1, [SCREEN_HEIGHT]
        ; width*0.5 * height*0.5 * alignment (4) = width * height
        add r1, PAINT_BUFFER_START_INDEX
        mov [PAINT_BUFFER_END_INDEX], r1

    ; fill paint buffer with white
    .fill_with_white:
        ; get current buffer idx
        mov r0, [PAINT_BUFFER_INDEX]
        ; turn white until we reach the vram buffer
        mov [r0], 0xFFFFFF00
        add [PAINT_BUFFER_INDEX], 4
        ; compare with end of buffer, which is still in r1
        cmp [PAINT_BUFFER_INDEX], r1
        and fl, cmp_less_signed
        jnz .fill_with_white, fl

        jmp render_paint_buffer

map_selected_color:
    mov r0, [SELECTED_COLOR_INDEX]

    .set_red:
        mov [SELECTED_COLOR], 0xFF000000
        jz render_paint_buffer, r0
        sub r0, 1

    .set_yellow:
        mov [SELECTED_COLOR], 0xFFFF0000
        jz render_paint_buffer, r0
        sub r0, 1

    .set_green:
        mov [SELECTED_COLOR], 0x00FF0000
        jz render_paint_buffer, r0
        sub r0, 1

    .set_teal:
        mov [SELECTED_COLOR], 0x00FFFF00
        jz render_paint_buffer, r0
        sub r0, 1

    .set_blue:
        mov [SELECTED_COLOR], 0x0000FF00
        jz render_paint_buffer, r0
        sub r0, 1

    .set_purple:
        mov [SELECTED_COLOR], 0xFF00FF00
        jz render_paint_buffer, r0
        sub r0, 1

    .set_gray:
        mov [SELECTED_COLOR], 0xCCCCCC00
        jz render_paint_buffer, r0
        sub r0, 1

    .set_white:
        mov [SELECTED_COLOR], 0xFFFFFF00
        jz render_paint_buffer, r0
        sub r0, 1

    .set_dark_red:
        mov [SELECTED_COLOR], 0x99000000
        jz render_paint_buffer, r0
        sub r0, 1

    .set_dark_yellow:
        mov [SELECTED_COLOR], 0x99990000
        jz render_paint_buffer, r0
        sub r0, 1

    .set_dark_green:
        mov [SELECTED_COLOR], 0x00990000
        jz render_paint_buffer, r0
        sub r0, 1

    .set_dark_teal:
        mov [SELECTED_COLOR], 0x00999900
        jz render_paint_buffer, r0
        sub r0, 1

    .set_dark_blue:
        mov [SELECTED_COLOR], 0x00009900
        jz render_paint_buffer, r0
        sub r0, 1

    .set_dark_purple:
        mov [SELECTED_COLOR], 0x99009900
        jz render_paint_buffer, r0
        sub r0, 1

    .set_dark_gray:
        mov [SELECTED_COLOR], 0x55555500
        jz render_paint_buffer, r0
        sub r0, 1

    .set_black:
        mov [SELECTED_COLOR], 0x00000000
        jz render_paint_buffer, r0
        sub r0, 1

previous_selected_color:
    sub [SELECTED_COLOR_INDEX], 1
    rem [SELECTED_COLOR_INDEX], 16
    jmp map_selected_color
next_selected_color:
    add [SELECTED_COLOR_INDEX], 1
    rem [SELECTED_COLOR_INDEX], 16
    jmp map_selected_color

render_paint_buffer:
    mov [PAINT_BUFFER_INDEX], PAINT_BUFFER_START_INDEX
    ; get end of paint buffer (half of screen size on each axis)
    .loop:
        .copy_pixel:
            .calculate_position:
                ; t = i % (screen_width * 0.5) * alignment (4)
                mov r0, [SCREEN_WIDTH]
                mul r0, 2
                mov r1, [PAINT_BUFFER_INDEX]
                sub r1, PAINT_BUFFER_START_INDEX
                rem r1, r0

                ; y = (i - t) * 2
                mov r0, [PAINT_BUFFER_INDEX]
                sub r0, PAINT_BUFFER_START_INDEX
                sub r0, r1
                mul r0, 2

                ; x = i * 2
                mov r1, [PAINT_BUFFER_INDEX]
                sub r1, PAINT_BUFFER_START_INDEX
                mul r1, 2

                ; move offset to base ptr
                add r0, r1
                add r0, [VRAM]
            .draw_square:
                mov r1, [PAINT_BUFFER_INDEX]
                mov r1, [r1]
                mov [r0], r1

                add r0, 4
                mov [r0], r1
                
                mov r1, [PAINT_BUFFER_INDEX]
                cmp r1, [CURSOR]
                and fl, cmp_equal
                jz .draw_bottom, fl
            .draw_cursor:
                mov r1, [SCREEN_WIDTH]
                mul r1, 4
                
                add r0, r1
                mov [r0], r1

                mov r1, [SELECTED_COLOR]
                
                sub r0, 4
                mov [r0], r1
                jmp .square_drawn
            .draw_bottom:
                mov r1, [SCREEN_WIDTH]
                mul r1, 4
                
                add r0, r1
                mov r1, [PAINT_BUFFER_INDEX]
                mov r1, [r1]
                mov [r0], r1
                
                sub r0, 4
                mov [r0], r1
            .square_drawn:
        .increment_and_check:
            add [PAINT_BUFFER_INDEX], 4
            mov r1, [PAINT_BUFFER_END_INDEX]
            cmp [PAINT_BUFFER_INDEX], r1
            and fl, cmp_less_signed
            jnz .loop, fl

    jmp wait_for_keypress
