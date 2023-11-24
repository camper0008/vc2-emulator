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
%define clr_general_bg 0x22222200
%define clr_light_orange 0xF69A7900
%define clr_dark_orange 0xC03C0C00
%define clr_light_blue 0xABA4CB00
%define clr_dark_blue 0x24203800

; io
%define vram_location 0x2034
%define screen_width 0x2038
%define screen_height 0x203C
%define keyboard_callback 0x202C
%define keyboard_scancode 0x2028
%define keyboard_event_type 0x2024

; image + background
; r0 = start offset
image_draw_return_address: dw 0 ; as done by linker in the case of image

%define seed_modulus 0x8000_0000
%define seed_multiplier 1103515245
%define seed_increment 12345
seed: dw 2228321880
seed_increment_return_address: dw 0

increment_seed:
    mul [seed], seed_multiplier
    add [seed], seed_increment
    rem [seed], seed_modulus
    jmp [seed_increment_return_address]

user_index_counter: dw 0
user_value_counter: dw 0
dealer_index_counter: dw 0
dealer_value_counter: dw 0

keyboard_event_happened:
    mov [keyboard_callback], 0

    %define .sdl_scancode_h 0x00B
    %define .sdl_scancode_s 0x016
    
    sub [keyboard_event_type], 2
    jnz .not_release, [keyboard_event_type]

    .check_if_h:
        cmp [keyboard_scancode], .sdl_scancode_h
        and fl, cmp_equal
        jz .check_if_s, fl
        cmp [user_cards_len], 7
        and fl, cmp_equal
        jnz .check_if_s, fl

        mov [push_card_initial_card], 0b0100_0000
        mov [push_card_len_addr], user_cards_len
        mov [push_card_return_address], .card_pushed
        jmp push_card
        .card_pushed:
            mov [image_draw_return_address], .check_win_condition
            jmp draw_cards
    .check_if_s:
        cmp [keyboard_scancode], .sdl_scancode_s
        and fl, cmp_equal
        jz .check_win_condition, fl
        ; draw some cards while dealer < 17
        .if_s__count_dealer_pts:
            ; push card to dealer
            mov [push_card_initial_card], 0b0100_0000
            mov [push_card_len_addr], dealer_cards_len
            mov [push_card_return_address], .count_if_not_end
            mov r0, [dealer_cards_len]
            mov [dealer_index_counter], r0
            jmp push_card
            .count_if_not_end:
                sub [dealer_index_counter], 1
                mov r1, [dealer_index_counter]
                rem r1, 4
                mov r0, [dealer_index_counter]
                sub r0, r1
                add r0, dealer_cards_bp
                mov r0, [r0]
                mul r1, 8
                add r1, 8
                shl r0, r1
                and r0, 0b0000_1111
                add r0, 1
                cmp r0, 10
                and fl, cmp_less_u
                jnz .if_s__dealer_less_than_10, fl
                mov r0, 10
                .if_s__dealer_less_than_10:
                add [dealer_value_counter], r0
                jnz .count_if_not_end, [dealer_index_counter]
            ; if dealer_pts < 17 -> 1
            cmp [dealer_value_counter], 17
            and fl, cmp_less_u
            jz .if_s__count_dealer_pts, fl
                
        ; reveal the cards
        or [dealer_cards_bp], 0b0100_0000_0100_0000_0100_0000_0100_0000
        ; draw revealed cards
        mov [image_draw_return_address], .check_win_condition
        jmp draw_cards

    .check_win_condition:
        mov r0, [user_cards_len]
        mov [user_index_counter], r0 
        mov [user_value_counter], 0
        mov r0, [dealer_cards_len]
        mov [dealer_index_counter], r0 
        mov [dealer_value_counter], 0
        .count_user_pts:
            sub [user_index_counter], 1

            ; r0 = outer_idx ( (len - (len % 4))/4 * 4 )
            mov r1, [user_index_counter]
            rem r1, 4
            mov r0, [user_index_counter]
            sub r0, r1

            ; r0 = outer_idx + cards_base
            add r0, user_cards_bp
            ; r0 = card
            mov r0, [r0]
            ; card = [r0] << (inner_idx*8 + 8)
            ; inner = 2
            ; 0x00_00_FF_00
            ; inner << 2*8
            ; 0xFF_00_00_00
            ; inner << 8
            ; 0x00_00_00_FF
            mul r1, 8
            add r1, 8
            shl r0, r1
            ; get card value
            and r0, 0b0000_1111
            ; our points dont start at 0
            add r0, 1
            ; if value > 10, then value = 10 (image cards are all 10)
            cmp r0, 10
            and fl, cmp_less_u
            jnz .user_less_than_10, fl
            mov r0, 10
            .user_less_than_10:
            ; add value
            add [user_value_counter], r0

            jnz .count_user_pts, [user_index_counter]
        
        ; do the same for the dealer

        .count_dealer_pts:
            sub [dealer_index_counter], 1
            mov r1, [dealer_index_counter]
            rem r1, 4
            mov r0, [dealer_index_counter]
            sub r0, r1
            add r0, dealer_cards_bp
            mov r0, [r0]
            mul r1, 8
            add r1, 8
            shl r0, r1
            and r0, 0b0000_1111
            add r0, 1
            cmp r0, 10
            and fl, cmp_less_u
            jnz .dealer_less_than_10, fl
            mov r0, 10
            .dealer_less_than_10:
            add [dealer_value_counter], r0
            jnz .count_dealer_pts, [dealer_index_counter]

        ; check for tie if score = 21
        cmp [user_value_counter], 21
        and fl, cmp_equal
        jnz .player_maybe_won, fl

        ; lost if > 21
        cmp [user_value_counter], 21
        and fl, cmp_less_u
        jz draw_player_status@lost, fl

        ; if player didnt lose, check if dealer > player if player did stand, else check if dealer lost
        cmp [keyboard_scancode], .sdl_scancode_s
        and fl, cmp_equal
        jz .check_dealer_lost, fl
        mov r0, [dealer_value_counter]
        ; if user < dealer, fl > 0
        cmp [user_value_counter], r0
        and fl, cmp_less_u
        jnz draw_player_status@lost, fl
        jmp draw_player_status@won

        .player_maybe_won:
        cmp [dealer_value_counter], 21
        and fl, cmp_equal
        jnz draw_player_status@tie, fl
        jmp draw_player_status@won

        .check_dealer_lost:
        cmp [dealer_value_counter], 21
        and fl, cmp_less_u
        jz draw_player_status@won, fl

    .not_release:
        mov [keyboard_callback], keyboard_event_happened
        hlt

draw_player_status:
    .lost:
        mov [rect_color], 0xCC000000
        jmp .draw
    .won:
        mov [rect_color], 0x00CC0000
        jmp .draw
    .tie:
        mov [rect_color], 0xCCCCCC00
        jmp .draw
    .draw:
        mov r1, [screen_width]
        mov [rect_width], r1
        mov r1, 1
        mov [rect_height], r1
        mov r0, [screen_height]
        mul r0, [screen_width]
        sub r0, [screen_width]
        mul r0, 4
        mov [image_draw_return_address], .done
        jmp draw_rect
    .done:
        ; reveal dealer card
        or [dealer_cards_bp], 0b0100_0000_0100_0000_0100_0000_0100_0000
        ; draw revealed cards
        mov [image_draw_return_address], .cards_drawn
        jmp draw_cards
        .cards_drawn:
            hlt

main:
    .initial_bg:
        mov r1, [screen_width]
        mov [rect_width], r1
        mov r1, [screen_height]
        mov [rect_height], r1
        mov [rect_color], clr_dark_blue
        mov r0, 0
        mov [image_draw_return_address], .initial_bg_done
        jmp draw_rect
    .initial_bg_done:

    mov [image_draw_return_address], .start_text_drawn
    jmp intro

    .start_text_drawn:
        mov [keyboard_callback], .randomness_is_seeded
        mov [seed_increment_return_address], increment_seed
        jmp increment_seed
    .randomness_is_seeded:
        mov [keyboard_callback], 0
        mov [rect_color], clr_general_bg
        mov [image_draw_return_address], .bg_done
        jmp draw_rect
    .bg_done:

    .push_initial_cards:
        mov [push_card_len_addr], user_cards_len
        mov [push_card_return_address], .l0
        jmp push_card
        .l0:
        mov [push_card_return_address], .l1
        jmp push_card
        .l1:
        mov [push_card_len_addr], dealer_cards_len
        mov [push_card_return_address], .l2
        jmp push_card
        .l2:
        mov [push_card_initial_card], 0
        mov [push_card_len_addr], dealer_cards_len
        mov [push_card_return_address], .l3
        jmp push_card
        .l3:

        mov [image_draw_return_address], .cards_done
        jmp draw_cards

    .cards_done:
        mov [keyboard_callback], keyboard_event_happened
        hlt

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
    mov r0, [push_card_initial_card]
    mov [push_card_value], r0
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

push_card_initial_card: dw 0b0100_0000
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

    ; r0 = outer_idx ( (len - (len % 4))/4 * 4 )
    mov r0, [push_card_len_addr]
    mov r0, [r0]
    mov r1, r0
    rem r1, 4
    sub r0, r1

    ; r0 = outer_idx + cards_base
    add r0, [push_card_len_addr]
    add r0, 4
    
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

    
