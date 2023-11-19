; [0x1000-0x100F] = temp variable

; === background ===
; [0x10BC] = background card color
; [0x10B0] = background width
; [0x10B1] = background height
; [0x10CB] = reserved for card background drawing

; background colors:
%define general_bg 0x44444400
%define orange_light 0xF69A7900
%define orange_dark 0xC03C0C00
%define blue_light 0xABA4CB00
%define blue_dark 0x24203800

; === image + background ===
; r0 = start offset
; [0x1500] = return address after rendering
; as done by linker in the case of image

main:
    .bg:
        mov r1, [0x2038]
        mov [0x10B0], r1
        mov r1, [0x203C]
        mov [0x10B1], r1
        mov [0x10BC], general_bg
        mov r0, 0
        mov [0x1500], abs_bg_done
        jmp draw_background
    bg_done:
    .center_card:
        mov r0, [0x203C]
        div r0, 2
        sub r0, 12
        mul r0, [0x2038]
        
        mov r1, [0x2038]
        div r1, 2
        sub r1, 8
        add r0, r1

    mov [0x1500], abs_background_done
    mov [0x10BC], orange_light
    jmp draw_card
background_done:
    mov [0x1500], abs_frame_done
    jmp card_frame
frame_done:
    mov [0x1500], abs_done
    jmp two

draw_card:
    ; preserve real return adddress
    mov r1, [0x1500]
    mov [0x10CB], r1

    ; shift one down
    add r0, [0x2038]
    ; width of 16
    mov [0x10B0], 16
    ; height of 22
    mov [0x10B1], 22
    mov [0x1500], abs_done_drawing_card
    jmp draw_background
    done_drawing_card:
        ; reset position
        sub r0, [0x2038]
        ; go back to return position
        jmpabs [0x10CB]
    

draw_background:
    ; we try to avoid overwriting the r0 register to follow the image semantics
    mov [0x1000], 0 ; width
    mov [0x1001], 0 ; height

    .loop:
        ; adjust height to vram height
        mov r1, [0x1001]
        mul r1, [0x2038]

        ; add width
        add r1, [0x1000]

        ; add vram offset
        add r1, [0x2034]

        ; add original positioning
        add r1, r0

        ; move background color into the fl register to avoid using r0
        mov fl, [0x10BC]
        mov [r1], fl
        
        ; reached end of width?
        add [0x1000], 1
        mov r1, [0x1000]
        cmp r1, [0x10B0]
        and fl, 0b100
        ; [0x1010] != [w] && jmp
        jz .is_not_eol, fl
        mov [0x1000], 0

        add [0x1001], 1

        .is_not_eol:
        mov r1, [0x1001]
        cmp r1, [0x10B1]
        and fl, 0b100

        ; [0x1009] != [h] && jmp
        jz .loop, fl
    jmpabs [0x1500]

done:
    jmpabs 0xFFFFFF
