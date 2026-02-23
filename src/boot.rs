use core::arch::global_asm;

global_asm!(r#"
    .set ALIGN,    1<<0
    .set MEMINFO,  1<<1
    .set FLAGS,    ALIGN | MEMINFO
    .set MAGIC,    0x1BADB002
    .set CHECKSUM, -(MAGIC + FLAGS)

    .section .multiboot, "a"
    .align 4
    .long MAGIC
    .long FLAGS
    .long CHECKSUM
"#);
