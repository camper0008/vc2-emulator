; [0x1500] = return address after image jmping
; as done by linker

main:
    mov r0, [0x203C]
    div r0, 2
    sub r0, 12
    mul r0, [0x2038]
    
    mov r1, [0x2038]
    div r1, 2
    sub r1, 8
    add r0, r1

    mov [0x1500], abs_back_done
    jmp card_frame
back_done:
    mov [0x1500], abs_done
    jmp card_back
done:
    jmpabs 0xFFFFFF
