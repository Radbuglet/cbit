# Cbit

<!-- cargo-rdme start -->

A proc-macro to use callback-based iterators with `for`-loop syntax and functionality.

### Overview

`cbit` (short for **c**losure-**b**ased **it**erator) is a crate which allows you to use iterator
functions which call into a closure to process each element as if they were just a regular Rust
`Iterator` in a `for` loop. To create an iterator, just define a function
which takes in a closure as its last argument. Both the function and the closure must return a
`ControlFlow` object with some generic `Break` type.

```rust
use std::ops::ControlFlow;

fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
    for i in 0..n {
        f(i)?;
    }
    ControlFlow::Continue(())
}
```

From there, you can use the iterator like a regular `for`-loop by driving it using the
`cbit!` macro.

```rust
fn demo(n: u64) -> u64 {
    let mut c = 0;
    cbit::cbit!(for i in up_to(n) {
        c += i;
    });
    c
}
```

Although the body of the `for` loop is technically nested in a closure, it supports all the
regular control-flow mechanisms one would expect:

You can early-`return` to the outer function...

```rust
fn demo(n: u64) -> u64 {
    let mut c = 0;
    cbit::cbit!(for i in up_to(n) {
        c += i;
        if c > 1000 {
            return u64::MAX;
        }
    });
    c
}

assert_eq!(demo(500), u64::MAX);
```

You can `break` and `continue` in the body...

```rust
fn demo(n: u64) -> u64 {
    let mut c = 0;
    cbit::cbit!('me: for i in up_to(n) {
        if i == 2 {
            continue 'me;  // This label is optional.
        }

        c += i;

        if c > 5 {
            break;
        }
    });
    c
}

assert_eq!(demo(5), 1 + 3 + 4);
```

And you can even `break` and `continue` to scopes outside the body!

```rust
fn demo(n: u64) -> u64 {
    let mut c = 0;
    'outer_1: loop {
        let something = 'outer_2: {
            cbit::cbit!(for i in up_to(n) break loop 'outer_1, 'outer_2 {
                if i == 5 && c < 20 {
                    continue 'outer_1;
                }
                if i == 8 {
                    break 'outer_2 c < 10;
                }
                c += i;
            });
            false
        };

        if something {
            assert!(c < 10);
        } else {
            break;
        }
    }
    c
}

demo(10);  // I'm honestly not really sure what this function is supposed to do.
```

Check the documentation of [`cbit!`] for more details on its syntax and specific behavior.

### Advantages and Drawbacks

Closure-based iterators play much nicer with the Rust optimizer than coroutines and their
[stable `async` userland counterpart](https://docs.rs/genawaiter/latest/genawaiter/) do
as of `rustc 1.74.0`.

Here is the disassembly of a regular loop implementation of factorial:

```rust
pub fn regular(n: u64) -> u64 {
    let mut c = 0;
    for i in 0..n {
        c += i;
    }
    c
}
```

```text
asm::regular:
Lfunc_begin7:
        push rbp
        mov rbp, rsp
        test rdi, rdi
        je LBB7_1
        lea rax, [rdi - 1]
        lea rcx, [rdi - 2]
        mul rcx
        shld rdx, rax, 63
        lea rax, [rdi + rdx - 1]
        pop rbp
        ret
LBB7_1:
        xor eax, eax
        pop rbp
        ret
```

...and here is the disassembly of the loop reimplemented in cbit:

```rust
use std::ops::ControlFlow;

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
```

```text
asm::cbit:
Lfunc_begin8:
        push rbp
        mov rbp, rsp
        test rdi, rdi
        je LBB8_1
        lea rax, [rdi - 1]
        lea rcx, [rdi - 2]
        mul rcx
        shld rdx, rax, 63
        lea rax, [rdi + rdx - 1]
        pop rbp
        ret
LBB8_1:
        xor eax, eax
        pop rbp
        ret
```

Except for the label names, they're entirely identical!

Meanwhile, the same example written with `rustc 1.76.0-nightly (49b3924bd 2023-11-27)`'s coroutines
yields far worse codegen ([permalink](https://godbolt.org/z/Kjh9q195s)):

```no_compile
#![feature(coroutines, coroutine_trait, iter_from_coroutine)]

use std::{iter::from_coroutine, ops::Coroutine};

fn upto_n(n: u64) -> impl Coroutine<Yield = u64, Return = ()> {
    move || {
        for i in 0..n {
            yield i;
        }
    }
}

pub fn sum(n: u64) -> u64 {
    let mut c = 0;
    let mut co = std::pin::pin!(upto_n(n));
    for i in from_coroutine(co) {
        c += i;
    }
    c
}
```

```text
example::sum:
        xor     edx, edx
        xor     eax, eax
        test    edx, edx
        je      .LBB0_4
.LBB0_2:
        cmp     edx, 3
        jne     .LBB0_3
        cmp     rcx, rdi
        jb      .LBB0_7
        jmp     .LBB0_6
.LBB0_4:
        xor     ecx, ecx
        cmp     rcx, rdi
        jae     .LBB0_6
.LBB0_7:
        setb    dl
        movzx   edx, dl
        add     rax, rcx
        add     rcx, rdx
        lea     edx, [2*rdx + 1]
        test    edx, edx
        jne     .LBB0_2
        jmp     .LBB0_4
.LBB0_6:
        ret
.LBB0_3:
        push    rax
        lea     rdi, [rip + str.0]
        lea     rdx, [rip + .L__unnamed_1]
        mov     esi, 34
        call    qword ptr [rip + core::panicking::panic@GOTPCREL]
        ud2
```

A similar thing can be seen with userland implementations of this feature such as
[`genawaiter`](https://docs.rs/genawaiter/latest/genawaiter/index.html).

However, what more general coroutine implementations provide in exchange for potential performance
degradation is immense expressivity. Fundamentally, `cbit` iterators cannot be interwoven, making
adapters such as `zip` impossible to implement—something coroutines have no problem doing.

<!-- cargo-rdme end -->
