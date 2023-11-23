jmp main

; [0x1000-0x100F] = temp variable
; eq
%define cmp_equal 0x04
%define cmp_less_i 0x08
%define cmp_less_u 0x10

; background
rect_color: dw 0
rect_width: dw 0
rect_height: dw 0
card_background_drawing: dw 0

; cards
; card format: 0b0RCCTTTT
; T = card type from 0-12
; C = color (0-3 -> light red, dark red, light blue, dark blue)
; R = revealed
; there are 4 cards packed per word, first from the left
%define card_alignment 8
%define cards_packed 4
user_cards_len: dw 0
user_cards_bp: dw 0
%offset_word 1
dealer_cards_len: dw 0
dealer_cards_bp: dw 0
%offset_word 1

; one bit per card, starting from the right
taken_card_index: dw 0
; light red 0-12, dark red 0-12, light blue 0-12, dark blue 0-12
taken_cards: 
    dw 0b11111111111111111111111111111111
    dw 0b00000000000111111111111111111111

; background colors
%define clr_general_bg 0x44444400
%define clr_light_orange 0xF69A7900
%define clr_dark_orange 0xC03C0C00
%define clr_light_blue 0xABA4CB00
%define clr_dark_blue 0x24203800

; io
%define vram_location 0x2034
%define screen_width 0x2038
%define screen_height 0x203C
%define keyboard_callback 0x202C
%define keyboard_keycode 0x2028

; image + background
; r0 = start offset
image_draw_return_address: dw 0 ; as done by linker in the case of image

%define seed_modulus 0x8000_0000
%define seed_multiplier 1103515245
%define seed_increment 12345
seed: dw 180504
seed_increment_return_address: dw 0

increment_seed:
    mul [seed], seed_multiplier
    add [seed], seed_increment
    rem [seed], seed_modulus
    jmp [seed_increment_return_address]

main:
    mov [keyboard_callback], .randomness_is_seeded
    mov [seed_increment_return_address], increment_seed
    jmp increment_seed
    .randomness_is_seeded:
    mov [keyboard_callback], 0

    ; .bg:
    ;     mov r1, [screen_width]
    ;     mov [rect_width], r1
    ;     mov r1, [screen_height]
    ;     mov [rect_height], r1
    ;     mov [rect_color], clr_general_bg
    ;     mov r0, 0
    ;     mov [image_draw_return_address], .bg_done
    ;     jmp draw_rect

    .bg_done:
        mov [push_card_len_addr], user_cards_len
        mov [push_card_return_address], .l0
        jmp push_card
        .l0:
        mov [push_card_return_address], .l1
        jmp push_card
        .l1:
        mov [push_card_return_address], .l2
        jmp push_card
        .l2:
        mov [push_card_len_addr], dealer_cards_len
        mov [push_card_return_address], .l3
        jmp push_card
        .l3:
        mov [push_card_len_addr], dealer_cards_len
        mov [push_card_return_address], .l4
        jmp push_card
        .l4:

        mov [image_draw_return_address], .cards_done
        jmp draw_cards

    .cards_done:
        mov r0, [seed]
        jmp 0xFFFFFFFF

set_card_from_index:
    ; card_mask = [taken_cards + r1] >> r0
    ; check if card == 1
    mov r0, [taken_card_index]
    mov r1, r0
    rem r0, 32
    sub r1, r0
    div r1, 8 ; (x / 32) * 4

    ; ran out of registers
    mov fl, 1
    shl fl, r0
    
    add r1, taken_cards
    and fl, [r1]
    
    jz push_card@select_card, fl
    ; flip bit bc taken
    not fl
    and [r1], fl

    ; set value based on card
    ; revealed
    mov [push_card_value], 0b0100_0000
    ; color
    mov r0, [taken_card_index]

    mov r1, r0
    rem r1, 13

    sub r0, r1
    div r0, 13
    shl r0, 4
    or [push_card_value], r0
    ; type
    mov r0, [taken_card_index]
    rem r0, 13
    or [push_card_value], r0
    ; adjust
    shl [push_card_value], 24
    jmp push_card@card_set

push_card_return_address: dw 0
push_card_value: dw 0
push_card_len_addr: dw 0
push_card:
    .select_card:
        mov [seed_increment_return_address], .seed_incremented
        .seed_incremented:
        mov r0, [seed]
        add [taken_card_index], r0
        rem [taken_card_index], 52
        jmp set_card_from_index

    .card_set:

    ; bp = len_ptr + 4
    ; r0 = (*cards_len - inner_idx) / 4
    ; reset any carrying bit from seed incrementing
    and fl, 0

    mov r0, [push_card_len_addr]
    mov r0, [r0]
    mov r1, r0
    rem r1, 4
    sub r0, r1
    div r0, 4

    ; r0 = outer_idx + cards_bp
    add r0, 4
    add r0, [push_card_len_addr]
    
    ; r1 = word >> inner_idx * 8
    mul r1, 8
    shr [push_card_value], r1
    mov r1, [push_card_value]
    ; [r0] |= word
    or [r0], r1
    mov r0, [push_card_len_addr]
    add [r0], 1
    jmp [push_card_return_address]

draw_cards:
    %define .saved_return_address 0x1210
    mov r1, [image_draw_return_address]
    mov [.saved_return_address], r1
    %define .card_bp_addr 0x1214
    mov [.card_bp_addr], user_cards_bp
    %define .card_len_addr 0x1218
    mov [.card_len_addr], user_cards_len
    %define .y_offset_addr 0x1220
    mov [.y_offset_addr], 0
    .cards_inner:
        ; for i = 0; i < len; i++
        %define .cards_index 0x1224
        mov [.cards_index], 0 ; i
        .cards_loop:
            %define .inner_idx 0x1228

            ; inner_idx = i % cards_packed
            mov r1, [.cards_index]
            mov [.inner_idx], r1 ; inner_idx
            rem [.inner_idx], cards_packed
            
            ; outer_idx = (i - inner_idx) / cards_packed
            mov r1, [.cards_index]
            sub r1, [.inner_idx]
            div r1, cards_packed
            %define .outer_idx 0x1230
            mov [.outer_idx], r1
            mul [.outer_idx], 4
            
            ; alignment = (cards_packed - 1 - inner_idx) * card_alignment
            mov r1, cards_packed
            sub r1, 1
            sub r1, [.inner_idx]
            mul r1, card_alignment
            %define .alignment 0x1234
            mov [.alignment], r1

            ; card = cards[outer_idx] >> alignment
            mov r1, [.outer_idx]
            add r1, [.card_bp_addr]
            mov r1, [r1]
            shr r1, [.alignment]
            %define .card 0x1238
            mov [.card], r1

            ; render card
            ; offset
            mov r1, [.y_offset_addr]
            mul r1, [screen_width]
            mul r1, 4
            mov r0, 16
            mul r0, [.cards_index]
            add r0, [screen_width]
            add r0, [.cards_index]
            mul r0, 4
            add r0, 4
            add r0, r1

            ; figure out current color
            ; is revealed?
            mov r1, [.card]
            and r1, 0b0100_0000
            mov [image_draw_return_address], .render_frame
            ; jmp draw back if revealed == 0
            jz card_back, r1
            .draw_card_rect:
                ; figure out color
                mov r1, [.card]
                and r1, 0b0011_0000
                shr r1, 4

                ; return addr
                mov [image_draw_return_address], .draw_number
                ; select clr 
                cmp r1, 0
                and fl, cmp_equal
                mov [rect_color], clr_light_orange
                jnz draw_card_background, fl

                cmp r1, 1
                and fl, cmp_equal
                mov [rect_color], clr_dark_orange
                jnz draw_card_background, fl

                cmp r1, 2
                and fl, cmp_equal
                mov [rect_color], clr_light_blue
                jnz draw_card_background, fl
                
                cmp r1, 3
                and fl, cmp_equal
                mov [rect_color], clr_dark_blue
                jnz draw_card_background, fl
            .draw_number:
                mov r1, [.card]
                and r1, 0b0000_1111
                ; return addr
                mov [image_draw_return_address], .render_frame
                ; select clr 
                cmp r1, 0
                and fl, cmp_equal
                jnz ace, fl

                cmp r1, 1
                and fl, cmp_equal
                jnz two, fl

                cmp r1, 2
                and fl, cmp_equal
                jnz three, fl

                cmp r1, 3
                and fl, cmp_equal
                jnz four, fl

                cmp r1, 4
                and fl, cmp_equal
                jnz five, fl

                cmp r1, 5
                and fl, cmp_equal
                jnz six, fl

                cmp r1, 6
                and fl, cmp_equal
                jnz seven, fl

                cmp r1, 7
                and fl, cmp_equal
                jnz eight, fl

                cmp r1, 8
                and fl, cmp_equal
                jnz nine, fl

                cmp r1, 9
                and fl, cmp_equal
                jnz ten, fl

                cmp r1, 10
                and fl, cmp_equal
                jnz jack, fl

                cmp r1, 11
                and fl, cmp_equal
                jnz queen, fl

                cmp r1, 12
                and fl, cmp_equal
                jnz king, fl

            ; draw frame
            .render_frame:
                mov [image_draw_return_address], .done_rendering_card
                jmp card_frame

        .done_rendering_card:
            ; i++
            add [.cards_index], 1
            ; if i < len, jmp
            mov r1, [.card_len_addr]
            mov r1, [r1]
            cmp [.cards_index], r1
            ; if fl == 1, then i < len
            and fl, cmp_less_u
            jnz .cards_loop, fl
            
            ; if rendering user cards, render dealer cards
            cmp [.card_bp_addr], user_cards_bp
            and fl, cmp_equal
            mov [.card_bp_addr], dealer_cards_bp
            mov [.card_len_addr], dealer_cards_len

            ; offset dealer cards
            mov r1, [screen_height]
            div r1, 2
            add r1, 1
            mov [.y_offset_addr], r1
            jnz .cards_inner, fl

    mov r1, [.saved_return_address]
    mov [image_draw_return_address], r1
    jmp [image_draw_return_address]

draw_card_background:

    ; preserve real return adddress
    mov r1, [image_draw_return_address]
    mov [card_background_drawing], r1

    ; width of 16
    mov [rect_width], 16
    ; height of 22
    mov [rect_height], 22
    mov r1, [screen_width]
    mul r1, 4
    add r0, r1
    mov [image_draw_return_address], .done_drawing_card
    jmp draw_rect
    .done_drawing_card:
    ; reset position
    mov r1, [screen_width]
    mul r1, 4
    sub r0, r1
    ; go back to return position
    mov r1, [card_background_drawing]
    mov [image_draw_return_address], r1
    jmp [image_draw_return_address]
    

draw_rect:
    ; we try to avoid overwriting the r0 register to follow the image semantics
    %define .width 0x1D00
    %define .height 0x1D04
    mov [.width], 0 ; width
    mov [.height], 0 ; height
    .loop:
        
        ; y * screen_width + x + vram_offset
        mov r1, [.height]
        mul r1, [screen_width]
        add r1, [.width]
        add r1, [vram_location]
        ; add caller offset
        add r1, r0

        ; use fl to avoid overwriting register
        mov fl, [rect_color]
        mov [r1], fl

        add [.width], 4
        mov r1, [rect_width]
        mul r1, 4
        cmp [.width], r1
        and fl, cmp_equal
        ; if x != width, draw next pixel
        jz .loop, fl
        ; x = 0; y += 1;
        mov [.width], 0
        add [.height], 4

        ; if y != height, draw next pixel
        mov r1, [rect_height]
        mul r1, 4
        cmp [.height], r1
        and fl, cmp_equal
        jz .loop, fl

    jmp [image_draw_return_address]

    
