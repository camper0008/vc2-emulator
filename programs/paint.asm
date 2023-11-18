; Painting buffer = 9120
; VRAM orig buffer = 9120 + 120*0.5 * 96*0.5 = 12000

; 0x100C     [C]ursor 
; 0x105C     [5]elected [C]olor
; 0x115C     [1]ndex [5]elected [C]olor
; 0x10B1     Painting [B]uffer [1]ndex
             
; 0x2034     Vram address
; 0x2024     Key event happened. 1 if press, 2 if release, else 0 (Read)
; 0x2028     Keycode (Read)
; 0x202c     Key event callback address. 0 disables callback (Write)
; 0x2034     VRAM address (Read+Write)
; 0x2038     Screen resolution width (Read)
; 0x203c     Screen resolution height (Read)

prepare:
    ; abs_ prefix means absolute position instead of relative position in preprocessor
    mov [0x202C], abs_key_event_happened
    ; set defaults
    mov [0x105C], 0xFF000000
    mov [0x100C], 9120
    mov [0x10B1], 9120
    mov [0x2034], 12000
    ; fill painting buffer with white
    .fill_with_white:
        ; get current buffer idx
        mov r0, [0x10B1]
        ; turn white until we reach the vram buffer
        mov [r0], 0xFFFFFF00
        add [0x10B1], 1
        cmp [0x10B1], 12000
        ; check if should loop
        and fl, 0b100
        jz .fill_with_white, fl

        ; reset buffer index and paint
        mov [0x10B1], 9120
        jmp render_painting_buffer

map_selected_color:
    mov r0, [0x115C]

    .set_red:
        mov [0x105C], 0xFF000000
        jz render_painting_buffer, r0
        sub r0, 1

    .set_yellow:
        mov [0x105C], 0xFFFF0000
        jz render_painting_buffer, r0
        sub r0, 1

    .set_green:
        mov [0x105C], 0x00FF0000
        jz render_painting_buffer, r0
        sub r0, 1

    .set_teal:
        mov [0x105C], 0x00FFFF00
        jz render_painting_buffer, r0
        sub r0, 1

    .set_blue:
        mov [0x105C], 0x0000FF00
        jz render_painting_buffer, r0
        sub r0, 1

    .set_purple:
        mov [0x105C], 0xFF00FF00
        jz render_painting_buffer, r0
        sub r0, 1

    .set_gray:
        mov [0x105C], 0xCCCCCC00
        jz render_painting_buffer, r0
        sub r0, 1

    .set_white:
        mov [0x105C], 0xFFFFFF00
        jz render_painting_buffer, r0
        sub r0, 1

    .set_dark_red:
        mov [0x105C], 0x99000000
        jz render_painting_buffer, r0
        sub r0, 1

    .set_dark_yellow:
        mov [0x105C], 0x99990000
        jz render_painting_buffer, r0
        sub r0, 1

    .set_dark_green:
        mov [0x105C], 0x00990000
        jz render_painting_buffer, r0
        sub r0, 1

    .set_dark_teal:
        mov [0x105C], 0x00999900
        jz render_painting_buffer, r0
        sub r0, 1

    .set_dark_blue:
        mov [0x105C], 0x00009900
        jz render_painting_buffer, r0
        sub r0, 1

    .set_dark_purple:
        mov [0x105C], 0x99009900
        jz render_painting_buffer, r0
        sub r0, 1

    .set_dark_gray:
        mov [0x105C], 0x55555500
        jz render_painting_buffer, r0
        sub r0, 1

    .set_black:
        mov [0x105C], 0x00000000
        jz render_painting_buffer, r0
        sub r0, 1

previous_selected_color:
    sub [0x115C], 1
    rem [0x115C], 16
    jmp map_selected_color

next_selected_color:
    add [0x115C], 1
    rem [0x115C], 16
    jmp map_selected_color

render_painting_buffer:
    ; get current buffer idx
    ; get y position
    ; x offset
    mov r1, [0x10B1]
    sub r1, 9120
    rem r1, 60

    ; y position
    mov r0, [0x10B1]
    sub r0, 9120
    sub r0, r1
    mul r0, 2

    ; x position
    mov r1, [0x10B1]
    sub r1, 9120
    mul r1, 2

    ; move offset to base ptr
    add r0, r1
    add r0, [0x2034]

    ; get color at position
    mov r1, [0x10B1]
    mov r1, [r1]

    ; draw current color if at bottom of screen
    cmp [0x10B1], 11940 ; 60*47 + 9120
    and fl, 0b1000
    jnz .skip_draw_current_color, fl

    mov r1, [0x105C]

    .skip_draw_current_color:

    ; turn color
    mov [r0], r1
    ; draw corners
    add r0, 1
    mov [r0], r1
    add r0, 119
    mov [r0], r1
    add r0, 1
    mov [r0], r1

    ; draw cursor if position == cursor position
    mov r1, [0x10B1]
    cmp [0x100C], r1
    and fl, 0b100
    jz .skip_draw_cursor, fl

    mov r1, [0x10B1]
    mov r1, [r1]
    not r1
    mov [r0], r1
    sub r0, 1
    mov [r0], r1

    .skip_draw_cursor:

    ; increment idx
    add [0x10B1], 1
    ; has reached end?
    cmp [0x10B1], 12000
    and fl, 0b100
    jz render_painting_buffer, fl

    ; reset buffer index and go back to waiting for keypress
    mov [0x10B1], 9120
    jmp wait_for_keypress

key_event_happened:
    ; 0x2024 	Key event happened. 1 if press, 2 if release, else 0
    ; 0x2028 	Keycode

    mov r0, [0x2024]
    jz wait_for_keypress, r0

    ; check keycode
    cmp [0x2028], 82
    and fl, 0b100
    jnz .key_up, fl

    cmp [0x2028], 81
    and fl, 0b100
    jnz .key_down, fl

    cmp [0x2028], 80
    and fl, 0b100
    jnz .key_left, fl

    cmp [0x2028], 79
    and fl, 0b100
    jnz .key_right, fl

    cmp [0x2028], 20
    and fl, 0b100
    jnz .key_q, fl

    cmp [0x2028], 8
    and fl, 0b100
    jnz .key_e, fl

    cmp [0x2028], 44
    and fl, 0b100
    jnz .key_space, fl

    ; irrelevant key was pressed
    jmp wait_for_keypress
    
    .key_up:
        ; at top of buffer
        cmp [0x100C], 9180 ; 9120 + 60 (assuming width is 60)
        and fl, 0b1000

        ; save original spot
        mov r1, [0x100C]
        ; move to other end; will skip moving to actual spot if at end
        add [0x100C], 2760 ; 60*46
        jnz render_painting_buffer, fl

        ; move to intended spot
        mov [0x100C], r1
        sub [0x100C], 60

        jmp render_painting_buffer
    .key_left:
        ; at start of buffer
        cmp [0x100C], 9120
        and fl, 0b100
        
        ; save original spot
        mov r1, [0x100C]
        ; move to other end; will skip moving to actual spot if at end
        mov [0x100C], 11939 ; 60 * 47 + 9120 - 1
        jnz render_painting_buffer, fl

        ; move to actual spot
        mov [0x100C], r1
        sub [0x100C], 1

        jmp render_painting_buffer
    .key_right:
        ; at end of buffer
        cmp [0x100C], 11939 ; 60 × 47 + 9120 - 1
        and fl, 0b100
        
        ; save original spot
        mov r1, [0x100C]
        ; move to other end; will skip moving to actual spot if at end
        mov [0x100C], 9120 ; start of buffer
        jnz render_painting_buffer, fl

        ; move to actual spot
        mov [0x100C], r1
        add [0x100C], 1

        jmp render_painting_buffer
    .key_down:
        ; at bottom of buffer
        cmp [0x100C], 11880 ; 60 * 46 + 9120
        and fl, 0b1000

        ; save original spot
        mov r1, [0x100C]
        ; move to other end; will skip moving to actual spot if at end
        sub [0x100C], 2760 ; 60*46
        jz render_painting_buffer, fl

        ; move to intended spot
        mov [0x100C], r1
        add [0x100C], 60

        jmp render_painting_buffer
    .key_q:
        jmp previous_selected_color
    .key_e:
        jmp next_selected_color
    .key_space:
        mov r0, [0x105C]
        mov r1, [0x100C]
        mov [r1], r0
        jmp render_painting_buffer

wait_for_keypress:
    hlt
