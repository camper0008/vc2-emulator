one:
    %define .sub 2
    mov r1, .sub
    jmp two@label
two:
    .label:
        %define .sub 3
        mov r1, .sub