
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 6
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_5_start
    .quad app_5_end

    .global _app_names
_app_names:
    .string "exec"
    .string "fork"
    .string "get"
    .string "hello_world"
    .string "initproc"
    .string "shell"

    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "../usr/target/riscv64gc-unknown-none-elf/release/exec"
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
    .align 3
app_1_start:
    .incbin "../usr/target/riscv64gc-unknown-none-elf/release/fork"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
    .align 3
app_2_start:
    .incbin "../usr/target/riscv64gc-unknown-none-elf/release/get"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
    .align 3
app_3_start:
    .incbin "../usr/target/riscv64gc-unknown-none-elf/release/hello_world"
app_3_end:

    .section .data
    .global app_4_start
    .global app_4_end
    .align 3
app_4_start:
    .incbin "../usr/target/riscv64gc-unknown-none-elf/release/initproc"
app_4_end:

    .section .data
    .global app_5_start
    .global app_5_end
    .align 3
app_5_start:
    .incbin "../usr/target/riscv64gc-unknown-none-elf/release/shell"
app_5_end:
