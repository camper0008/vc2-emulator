jmp one

sp:
    db 0x12 0x34 0x56 0x78
bp:
    dw 0x87654321

one:
    %define .sub 2
    mov r1, .sub
    jmp two@label
two:
    .label:
        %define .sub 3
        mov r1, .sub
    mov r0, [abs_sp]
    mov r1, [abs_bp]
    jmpabs 0xFFFFFFFF
