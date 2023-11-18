
; Painting buffer = 9120
; VRAM orig buffer = 9120 + 120*0.5 * 96*0.5 = 12000
; VRAM swap buffer = 12000 + 120*96 = 23520

; 0x100B     [B]uffer being rendered
; 0x100C     [C]ursor 
; 0x105C     [5]elected [C]olor
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
    mov [0x100B], 12000
    mov [0x2034], 23520
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

swap_buffer:
    mov r0, [0x100B]
    rem r0, 12000
    jz .buffer_1, r0
    .buffer_0:
        mov [0x100B], 12000
        mov [0x2034], 23520
        jmp wait_for_keypress
    .buffer_1:
        mov [0x2034], 12000
        mov [0x100B], 23520
        jmp wait_for_keypress

previous_selected_color:
    jmp next_selected_color

next_selected_color:
    cmp [0x105C], 0xFF000000
    and fl, 0b100
    jnz .is_red, fl

    cmp [0x105C], 0xFFFF0000
    and fl, 0b100
    jnz .is_yellow, fl

    cmp [0x105C], 0x00FF0000
    and fl, 0b100
    jnz .is_green, fl

    cmp [0x105C], 0x00FFFF00
    and fl, 0b100
    jnz .is_teal, fl

    cmp [0x105C], 0x0000FF00
    and fl, 0b100
    jnz .is_blue, fl

    cmp [0x105C], 0xFF00FF00
    and fl, 0b100
    jnz .is_purple, fl

    cmp [0x105C], 0xFFFFFF00
    and fl, 0b100
    jnz .is_white, fl

    cmp [0x105C], 0x00000000
    and fl, 0b100
    jnz .is_black, fl

    .is_red:
        mov [0x105C], 0xFFFF0000
        jmp render_painting_buffer

    .is_yellow:
        mov [0x105C], 0x00FF0000
        jmp render_painting_buffer

    .is_green:
        mov [0x105C], 0x00FFFF00
        jmp render_painting_buffer

    .is_teal:
        mov [0x105C], 0x0000FF00
        jmp render_painting_buffer

    .is_blue:
        mov [0x105C], 0xFF00FF00
        jmp render_painting_buffer

    .is_purple:
        mov [0x105C], 0xFFFFFF00
        jmp render_painting_buffer

    .is_white:
        mov [0x105C], 0x00000000
        jmp render_painting_buffer

    .is_black:
        mov [0x105C], 0xFF000000
        jmp render_painting_buffer

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
    add r0, [0x100B]

    ; get color at position
    mov r1, [0x10B1]
    mov r1, [r1]

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
    mov r1, [0x105C]
    sub r0, 1
    mov [r0], r1
    .skip_draw_cursor:

    ; increment idx
    add [0x10B1], 1
    ; has reached end?
    cmp [0x10B1], 12000
    and fl, 0b100
    jz render_painting_buffer, fl

    ; reset buffer index and swap buffers
    mov [0x10B1], 9120
    jmp swap_buffer

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
        add [0x100C], 2820
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
        mov [0x100C], 9120
        add [0x100C], 2880
        jnz render_painting_buffer, fl

        ; move to actual spot
        mov [0x100C], r1
        sub [0x100C], 1

        jmp render_painting_buffer
    .key_right:
        ; at end of buffer
        cmp [0x100C], 11819
        and fl, 0b100
        
        ; save original spot
        mov r1, [0x100C]
        ; move to other end; will skip moving to actual spot if at end
        mov [0x100C], 9120
        jnz render_painting_buffer, fl

        ; move to actual spot
        mov [0x100C], r1
        add [0x100C], 1

        jmp render_painting_buffer
    .key_down:
        ; at bottom of buffer
        cmp [0x100C], 11940 ; 12000 - 60
        and fl, 0b1000

        ; save original spot
        mov r1, [0x100C]
        ; move to other end; will skip moving to actual spot if at end
        sub [0x100C], 2820
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
