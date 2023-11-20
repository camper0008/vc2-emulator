; [0x1000-0x100F] = temp variable
; eq
%define cmp_equal 0x04
%define cmp_less_i 0x08
%define cmp_less_u 0x10

; background
%define rect_color 0x1100
%define rect_width 0x1104
%define rect_height 0x1108
%define __card_background_drawing 0x1110

; cards
; card format: 0b0RCCTTTT
; T = card type from 0-12
; C = color (0-3 -> light red, dark red, light blue, dark blue)
; R = revealed
; there are 4 cards packed per word, first from the left
%define card_alignment 8
%define cards_packed 4
%define user_cards_len 0x11C0
%define user_cards_bp 0x11C4
%define dealer_cards_len 0x11E0
%define dealer_cards_bp 0x11E4
; one bit per card, starting from the left with light red, dark red, light blue, dark blue
%define taken_cards_0 0x11B0
%define taken_cards_1 0x11B4

; background colors
%define clr_general_bg 0x44444400
%define clr_light_orange 0xF69A7900
%define clr_dark_orange 0xC03C0C00
%define clr_light_blue 0xABA4CB00
%define clr_dark_blue 0x24203800

; screen
%define vram_location 0x2034
%define screen_width 0x2038
%define screen_height 0x203C

; image + background
; r0 = start offset
%define return_address 0x1500 ; as done by linker in the case of image

main:
    .bg:
        mov r1, [screen_width]
        mov [rect_width], r1
        mov r1, [screen_height]
        mov [rect_height], r1
        mov [rect_color], clr_general_bg
        mov r0, 0
        mov [return_address], abs_bg_done
        jmp draw_rect
    bg_done:
        mov [return_address], abs_cards_done
        
        ; user cards
        mov [user_cards_len], 7
        mov [user_cards_bp], 0b01000000_01010001_01100010_01110011
        mov r1, user_cards_bp
        add r1, 4
        mov [r1], 0b01000100_01010101_01100110_1111_1111
        
        ; dealer cards
        mov [dealer_cards_len], 7
        mov [dealer_cards_bp], 0b01_11_0111__01_00_1000__01_01_1001__01_11_1010
        mov r1, dealer_cards_bp
        add r1, 4
        mov [r1], 0b01111011_01001100_00000001_1111_1111

        jmp draw_cards

cards_done:
    jmpabs 0xFFFFFFFF

draw_cards:
    %define .saved_return_address 0x1210
    mov r1, [return_address]
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
            mov [return_address], .abs_render_frame
            ; jmp draw back if revealed == 0
            jz card_back, r1
            .draw_card_rect:
                ; figure out color
                mov r1, [.card]
                and r1, 0b0011_0000
                shr r1, 4

                ; return addr
                mov [return_address], .abs_draw_number
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
                mov [return_address], .abs_render_frame
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
                mov [return_address], .abs_done_rendering_card
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
    mov [return_address], r1
    jmpabs [return_address]

draw_card_background:

    ; preserve real return adddress
    mov r1, [return_address]
    mov [__card_background_drawing], r1

    ; width of 16
    mov [rect_width], 16
    ; height of 22
    mov [rect_height], 22
    mov r1, [screen_width]
    mul r1, 4
    add r0, r1
    mov [return_address], .abs_done_drawing_card
    jmp draw_rect
    .done_drawing_card:
    ; reset position
    mov r1, [screen_width]
    mul r1, 4
    sub r0, r1
    ; go back to return position
    mov r1, [__card_background_drawing]
    mov [return_address], r1
    jmpabs [return_address]
    

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

    jmpabs [return_address]

done:
    jmpabs 0xFFFFFF
