jmp one

hello_world: db "hello world" '\0'
with_commas: db "hello", "\tworld", '\0'
sp: db 0x12 0x34 0x56 0x78
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
    mov r0, [sp]
    mov r1, [bp]
    jmp 0xFFFFFFFF
