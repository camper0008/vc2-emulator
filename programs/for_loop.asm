mov [50], 25
label:
    add [51], 1
    sub [50], 1
    mov r0, label
    sub r0, pc
    sub r0, 8
    jnz r0, [50]
    mov r0, [50]
    mov r1, [51]