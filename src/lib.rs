#![allow(rustdoc::redundant_explicit_links)] // For cargo-rdme's sake

//! Provides a proc-macro to use callback-based iterators with `for`-loop syntax and functionality.
//!
//! ## Overview
//!
//! `cbit` (short for **c**losure-**b**ased **it**erator) is a crate which allows you to use iterator
//! functions which call into a closure to process each element as if they were just a regular Rust
//! [`Iterator`](std::iter::Iterator) in a `for` loop. To create an iterator, just define a function
//! which takes in a closure as its last argument. Both the function and the closure must return a
//! [`ControlFlow`](std::ops::ControlFlow) object with some generic `Break` type.
//!
//! ```
//! use std::ops::ControlFlow;
//!
//! fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
//!     for i in 0..n {
//!         f(i)?;
//!     }
//!     ControlFlow::Continue(())
//! }
//! ```
//!
//! From there, you can use the iterator like a regular `for`-loop by driving it using the
//! [`cbit!`](cbit!) macro.
//!
//! ```rust
//! # use std::ops::ControlFlow;
//! # fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
//! #     for i in 0..n {
//! #         f(i)?;
//! #     }
//! #     ControlFlow::Continue(())
//! # }
//! fn demo(n: u64) -> u64 {
//!     let mut c = 0;
//!     cbit::cbit!(for i in up_to(n) {
//!         c += i;
//!     });
//!     c
//! }
//! ```
//!
//! Although the body of the `for` loop is technically nested in a closure, it supports all the
//! regular control-flow mechanisms one would expect:
//!
//! You can early-`return` to the outer function...
//!
//! ```rust
//! # use std::ops::ControlFlow;
//! # fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
//! #     for i in 0..n {
//! #         f(i)?;
//! #     }
//! #     ControlFlow::Continue(())
//! # }
//! fn demo(n: u64) -> u64 {
//!     let mut c = 0;
//!     cbit::cbit!(for i in up_to(n) {
//!         c += i;
//!         if c > 1000 {
//!             return u64::MAX;
//!         }
//!     });
//!     c
//! }
//!
//! assert_eq!(demo(500), u64::MAX);
//! ```
//!
//! You can `break` and `continue` in the body...
//!
//! ```rust
//! # use std::ops::ControlFlow;
//! # fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
//! #     for i in 0..n {
//! #         f(i)?;
//! #     }
//! #     ControlFlow::Continue(())
//! # }
//! fn demo(n: u64) -> u64 {
//!     let mut c = 0;
//!     cbit::cbit!('me: for i in up_to(n) {
//!         if i == 2 {
//!             continue 'me;  // This label is optional.
//!         }
//!
//!         c += i;
//!
//!         if c > 5 {
//!             break;
//!         }
//!     });
//!     c
//! }
//!
//! assert_eq!(demo(4), 1 + 3 + 4);
//! ```
//!
//! And you can even `break` and `continue` to scopes outside the body!
//!
//! ```rust
//! # use std::ops::ControlFlow;
//! # fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
//! #     for i in 0..n {
//! #         f(i)?;
//! #     }
//! #     ControlFlow::Continue(())
//! # }
//! fn demo(n: u64) -> u64 {
//!     let mut c = 0;
//!     'outer_1: loop {
//!         let did_just_break = 'outer_2: {
//!             cbit::cbit!(for i in up_to(n) break loop 'outer_1, 'outer_2 {
//!                 if i == 5 && c < 20 {
//!                     continue 'outer_1;
//!                 }
//!
//!                 if c == 3 {
//!                     break 'outer_2 true;
//!                 }
//!                 c += i;
//!             });
//!             false
//!         };
//!
//!         if did_just_break {
//!             assert_eq!(c, 3);
//!         }
//!     }
//!     c
//! }
//!
//! demo(10);  // I'm not exactly sure what this is supposed to equal.
//! ```
//!
//! Check the documentation of [`cbit!`] for more details on its syntax and specific behavior.
//!
//! ## Advantages and Drawbacks
//!
//! Closure-based iterators play much nicer with the Rust optimizer than coroutines and their
//! [stable `async` userland counterpart](TODO) do as of `rustc 1.74.0`.
//!
//! Here is the disassembly of a regular loop implementation of factorial:
//!
//! ```
//! pub fn regular(n: u64) -> u64 {
//!     let mut c = 0;
//!     for i in 0..n {
//!         c += i;
//!     }
//!     c
//! }
//! ```
//!
//! ```text
//! asm::regular:
//! Lfunc_begin7:
//!         push rbp
//!         mov rbp, rsp
//!         test rdi, rdi
//!         je LBB7_1
//!         lea rax, [rdi - 1]
//!         lea rcx, [rdi - 2]
//!         mul rcx
//!         shld rdx, rax, 63
//!         lea rax, [rdi + rdx - 1]
//!         pop rbp
//!         ret
//! LBB7_1:
//!         xor eax, eax
//!         pop rbp
//!         ret
//! ```
//!
//! ...and here is the disassembly of the loop reimplemented in cbit:
//!
//! ```
//! use std::ops::ControlFlow;
//!
//! pub fn cbit(n: u64) -> u64 {
//!     let mut c = 0;
//!     cbit::cbit!(for i in up_to(n) {
//!         c += i;
//!     });
//!     c
//! }
//!
//! fn up_to<B>(n: u64, mut f: impl FnMut(u64) -> ControlFlow<B>) -> ControlFlow<B> {
//!     for i in 0..n {
//!         f(i)?;
//!     }
//!     ControlFlow::Continue(())
//! }
//! ```
//!
//! ```text
//! asm::cbit:
//! Lfunc_begin8:
//!         push rbp
//!         mov rbp, rsp
//!         test rdi, rdi
//!         je LBB8_1
//!         lea rax, [rdi - 1]
//!         lea rcx, [rdi - 2]
//!         mul rcx
//!         shld rdx, rax, 63
//!         lea rax, [rdi + rdx - 1]
//!         pop rbp
//!         ret
//! LBB8_1:
//!         xor eax, eax
//!         pop rbp
//!         ret
//! ```
//!
//! Except for the label names, they're entirely identical!
//!
//! TODO

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, Lifetime, Token};
use syntax::CbitForExpr;

mod syntax;

/// A proc-macro to use callback-based iterators with for-loop syntax and functionality.
///
/// ## Syntax
///
/// ```text
/// ('<loop-label: lifetime>:)? for <binding: pattern> in <iterator: function-call-expr>
///     (break ((loop)? '<extern-label: lifetime>)*)?
/// {
///     <body: token stream>
/// }
/// ```
///
/// Arguments:
///
/// - `loop-label`: This is the optional label used by your virtual loop. `break`'ing or `continue`'ing
///   to this label will break and continue the iterator respectively.
/// - `binding`: This is the irrefutable pattern the iterator's arguments will be decomposed into.
/// - `iterator`: Syntactically, this can be any (potentially generic) function or method call
///   expression and generics can be explicitly supplied if desired. See the [iteration protocol](#iteration-protocol)
///   section for details on the semantic requirements for this function.
/// - The loop also contains an optional list of external `break` labels which is started by the
///   `break` keyword and is followed by a non-empty non-trailing comma-separated list of...
///      - An optional `loop` keyword which, if specified, asserts that the label can accept `continue`s
///        in addition to `break`s.
///      - `extern-label`: the label the `cbit!` body is allowed to `break` or `continue` out to.
///
/// ## Iteration Protocol
///
/// The called function or method can take on any non-zero number of arguments but must accept a
/// single-argument function closure as its last argument. The closure must be able to accept a
/// [`ControlFlow`] object with a generic `Break` type and the function must return a [`ControlFlow`]
/// object with the same `Break` type.
///
/// ```
/// use std::{iter::IntoIterator, ops::ControlFlow};
///
/// fn enumerate<I: IntoIterator, B>(
///     values: I,
///     index_offset: usize,
///     mut f: impl FnMut((usize, I::Item),
/// ) -> ControlFlow<B>) -> ControlFlow<B> {
///     for (i, v) in values.into_iter().enumerate() {
///         f((i + index_offset, v))?;
///     }
///     ControlFlow::Continue(())
/// }
/// ```
///
/// TODO

#[proc_macro]
pub fn cbit(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as CbitForExpr);

    // Define some common syntax trees
    let core_ = quote! { ::core };
    let ops_ = quote! { #core_::ops };
    let pin_ = quote! { #core_::pin };
    let task_ = quote! { #core_::task };
    let future_ = quote! { #core_::future };
    let option_ = quote! { #core_::option::Option };

    // Extract our break labels
    let empty_punct_list = Punctuated::new();
    let in_break_labels = input
        .breaks
        .as_ref()
        .map_or(&empty_punct_list, |breaks| &breaks.lt);

    let derive_early_break_variant_name =
        |lt: &Lifetime| Ident::new(&format!("EarlyBreakTo_{}", lt.ident), lt.span());

    let derive_early_continue_variant_name =
        |lt: &Lifetime| Ident::new(&format!("EarlyContinueTo_{}", lt.ident), lt.span());

    // Define an enum for our control flow
    let control_flow_enum_def;
    let control_flow_ty_decl;
    let control_flow_ty_use;
    {
        let break_variant_names = in_break_labels
            .iter()
            .map(|v| derive_early_break_variant_name(&v.lt))
            .collect::<Vec<_>>();

        let continue_variant_names = in_break_labels
            .iter()
            .filter(|&v| v.kw_loop.is_some())
            .map(|v| derive_early_continue_variant_name(&v.lt));

        control_flow_enum_def = quote! {
            #[allow(non_camel_case_types)]
            #[allow(clippy::enum_variant_names)]
            enum OurControlFlowResult<EarlyReturn, EarlyBreak #(, #break_variant_names)*> {
                EarlyReturn(EarlyReturn),
                EarlyBreak(EarlyBreak),
                #(#break_variant_names (#break_variant_names),)*
                #(#continue_variant_names,)*
            }
        };

        control_flow_ty_decl = quote! {
            #[allow(non_camel_case_types)]
            type OurControlFlow<EarlyReturn, EarlyBreak #(, #break_variant_names)*> = #ops_::ControlFlow<
                OurControlFlowResult<EarlyReturn, EarlyBreak #(, #break_variant_names)*>,
                EarlyBreak,
            >;
        };

        let underscores =
            (0..(break_variant_names.len() + 2)).map(|_| Token![_](Span::call_site()));

        control_flow_ty_use = quote! { OurControlFlow<#(#underscores),*> };
    }

    // Define our initial break layer
    let aborter = |resolution: TokenStream| {
        quote! {
            how_to_resolve_pending = #option_::Some(#resolution);
            #future_::pending::<()>().await;
            #core_::unreachable!();
        }
    };

    let for_body = input.body.body;
    let for_body = {
        let optional_label = &input.label;
        let break_aborter = aborter(quote! {
            #ops_::ControlFlow::Break(OurControlFlowResult::EarlyBreak(break_result))
        });

        quote! {
            '__cbit_absorber_magic_innermost: {
                let mut did_run = false;
                let break_result = #optional_label loop {
                    if did_run {
                        // The user must have used `continue`.
                        break '__cbit_absorber_magic_innermost;
                    }

                    did_run = true;
                    { #for_body };

                    // The user completed the loop.
                    #[allow(unreachable_code)]
                    {
                        break '__cbit_absorber_magic_innermost;
                    }
                };

                // The user broke out of the loop.
                #[allow(unreachable_code)]
                {
                    #break_aborter
                }
            }
        }
    };

    // Build up an onion of user-specified break layers
    let for_body = {
        let mut for_body = for_body;
        for break_label_entry in in_break_labels {
            let break_label = &break_label_entry.lt;

            let break_aborter = {
                let variant_name = derive_early_break_variant_name(break_label);
                aborter(quote! {
                    #ops_::ControlFlow::Break(OurControlFlowResult::#variant_name(break_result))
                })
            };

            let outer_label = Lifetime::new(
                &format!("'__cbit_absorber_magic_for_{}", break_label.ident),
                break_label.span(),
            );

            if break_label_entry.kw_loop.is_some() {
                let continue_aborter = {
                    let variant_name = derive_early_continue_variant_name(break_label);
                    aborter(quote! {
                        #ops_::ControlFlow::Break(OurControlFlowResult::#variant_name)
                    })
                };

                for_body = quote! {#outer_label: {
                    let mut did_run = false;
                    let break_result = #break_label: loop {
                        if did_run {
                            // The user must have used `continue`.
                            #continue_aborter
                        }

                        did_run = true;
                        { #for_body };

                        // The user completed the loop.
                        #[allow(unreachable_code)]
                        {
                            break #outer_label;
                        }
                    };

                    // The user broke out of the loop.
                    #[allow(unreachable_code)]
                    {
                        #break_aborter
                    }
                }};
            } else {
                for_body = quote! {#outer_label: {
                    let break_result = #break_label: {
                        { #for_body };

                        // The user completed the loop.
                        #[allow(unreachable_code)]
                        {
                            break #outer_label;
                        }
                    };

                    // The user broke out of the block.
                    #[allow(unreachable_code)]
                    {
                        #break_aborter
                    }
                }};
            }
        }

        for_body
    };

    // Build up a layer to capture early returns and generally process arguments
    let for_body = {
        let body_input_pat = &input.body_pattern;
        let early_return_aborter = aborter(quote! { #ops_::ControlFlow::Continue(()) });
        quote! {
            |#body_input_pat| {
                let mut how_to_resolve_pending = #option_::None;

                let body = #pin_::pin!(async {
                    { #for_body };

                    #[allow(unreachable_code)] { #early_return_aborter }
                });

                match #future_::Future::poll(
                    body,
                    &mut #task_::Context::from_waker(&{  // TODO: Use `Waker::noop` once it stabilizes
                        const VTABLE: #task_::RawWakerVTable = #task_::RawWakerVTable::new(
                            // Cloning just returns a new no-op raw waker
                            |_| RAW,
                            // `wake` does nothing
                            |_| {},
                            // `wake_by_ref` does nothing
                            |_| {},
                            // Dropping does nothing as we don't allocate anything
                            |_| {},
                        );
                        const RAW: #task_::RawWaker = #task_::RawWaker::new(#core_::ptr::null(), &VTABLE);
                        unsafe { #task_::Waker::from_raw(RAW) }
                    })
                ) {
                    #task_::Poll::Ready(early_return) => #ops_::ControlFlow::Break(
                        OurControlFlowResult::EarlyReturn(early_return),
                    ),
                    #task_::Poll::Pending => how_to_resolve_pending.expect(
                        "the async block in a cbit iterator is an implementation detail; do not \
                         `.await` in it!"
                    ),
                }
            }
        }
    };

    // Build up a list of break/continue handlers
    let break_out_matchers = in_break_labels.iter().map(|v| {
        let lt = &v.lt;
        let variant_name = derive_early_break_variant_name(lt);
        quote! {
            OurControlFlowResult::#variant_name(break_out) => break #lt break_out,
        }
    });

    let continue_out_matchers = in_break_labels
        .iter()
        .filter(|v| v.kw_loop.is_some())
        .map(|v| {
            let lt = &v.lt;
            let variant_name = derive_early_continue_variant_name(lt);
            quote! {
                OurControlFlowResult::#variant_name => continue #lt,
            }
        });

    // Build up our function call site
    let driver_call_site = match &input.call {
        syntax::AnyCallExpr::Function(call) => {
            let driver_attrs = &call.attrs;
            let driver_fn_expr = &call.func;
            let driver_fn_args = call.args.iter();

            quote! {
                #(#driver_attrs)*
                let result: #control_flow_ty_use = #driver_fn_expr (#(#driver_fn_args,)* #for_body);
            }
        }
        syntax::AnyCallExpr::Method(call) => {
            let driver_attrs = &call.attrs;
            let driver_receiver_expr = &call.receiver;
            let driver_method = &call.method;
            let driver_turbo = &call.turbofish;
            let driver_fn_args = call.args.iter();

            quote! {
                #(#driver_attrs)*
                let result: #control_flow_ty_use =
                    #driver_receiver_expr.#driver_method #driver_turbo (
                        #(#driver_fn_args,)*
                        #for_body
                    );
            }
        }
    };

    // Put it all together
    quote! {{
        // enum ControlFlowResult<...> { ... }
        #control_flow_enum_def

        // type ControlFlow<A, B, ...> = core::ops::ControlFlow<ControlFlowResult<A, B, ...>, A>;
        #control_flow_ty_decl

        // let result = my_fn(args, |...| async { ... });
        #driver_call_site

        match result {
            #ops_::ControlFlow::Break(result) => match result {
                OurControlFlowResult::EarlyReturn(early_result) => return early_result,
                OurControlFlowResult::EarlyBreak(result) => result,
                #(#break_out_matchers)*
                #(#continue_out_matchers)*
            },
            #ops_::ControlFlow::Continue(result) => result,
        }
    }}
    .into()
}
