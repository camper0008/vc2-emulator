# image-asm

generates vc2 assembly based on an image

pixels with an alpha value < 255 are ignored

write the position of the image to the r0 register before jumping to the subroutine

for example:

```x86asm
main:
    mov r0, 64
    mul r0, [0x2038] ; screen width location
    add r0, 32
    jmp my_image

my_image:
    (...)
```

will draw my\_image at (32, 64)
