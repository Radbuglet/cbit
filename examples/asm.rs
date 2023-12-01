use std::{hint::black_box, ops::ControlFlow};

fn main() {
    dbg!(regular(black_box(10)));
    dbg!(cbit(black_box(10)));
}

// See the assembly with `cargo asm --example asm asm::regular`.
//
// On my machine, this looks like:
//
// ```
// asm::regular:
// Lfunc_begin7:
//         push rbp
//         mov rbp, rsp
//         test rdi, rdi
//         je LBB7_1
//         lea rax, [rdi - 1]
//         lea rcx, [rdi - 2]
//         mul rcx
//         shld rdx, rax, 63
//         lea rax, [rdi + rdx - 1]
//         pop rbp
//         ret
// LBB7_1:
//         xor eax, eax
//         pop rbp
//         ret
// ```
#[inline(never)]
pub fn regular(n: u64) -> u64 {
    let mut c = 0;
    for i in 0..n {
        c += i;
    }
    c
}

// See the assembly with `cargo asm --example asm asm::cbit`.
//
// On my machine, this looks like:
//
// ```
// asm::cbit:
// Lfunc_begin8:
//         push rbp
//         mov rbp, rsp
//         test rdi, rdi
//         je LBB8_1
//         lea rax, [rdi - 1]
//         lea rcx, [rdi - 2]
//         mul rcx
//         shld rdx, rax, 63
//         lea rax, [rdi + rdx - 1]
//         pop rbp
//         ret
// LBB8_1:
//         xor eax, eax
//         pop rbp
//         ret
// ```
#[inline(never)]
pub fn cbit(n: u64) -> u64 {
    let mut c = 0;
    cbit::cbit!(for i in up_to(n) {
        c += i;
    });
    c
}

fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
    for i in 0..n {
        f(i)?;
    }
    ControlFlow::Continue(())
}
