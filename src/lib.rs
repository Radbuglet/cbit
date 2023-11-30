use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Lifetime, Token};
use syntax::CbitForExpr;

mod syntax;

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

    // Define an enum for our control flow
    let control_flow_enum_def;
    let control_flow_ty_decl;
    let control_flow_ty_use;
    {
        let break_variant_names = in_break_labels
            .iter()
            .map(derive_early_break_variant_name)
            .collect::<Vec<_>>();

        control_flow_enum_def = quote! {
            #[allow(non_camel_case_types)]
            #[allow(clippy::enum_variant_names)]
            enum OurControlFlowResult<EarlyReturn, EarlyBreak #(, #break_variant_names)*> {
                EarlyReturn(EarlyReturn),
                EarlyBreak(EarlyBreak),
                #(#break_variant_names (#break_variant_names),)*
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

    // Define our initial layer of break layers
    let aborter = |resolution: TokenStream| {
        quote! {
            how_to_resolve_pending = #option_::Some(#resolution);
            #future_::pending::<()>().await;
            #core_::unreachable!();
        }
    };

    let wrap_absorber = |body: TokenStream, label: Option<&Lifetime>| -> TokenStream {
        let break_aborter = match label {
            Some(label) => {
                let variant_name = derive_early_break_variant_name(label);
                aborter(quote! {
                    #ops_::ControlFlow::Break(OurControlFlowResult::#variant_name(break_result))
                })
            }
            None => aborter(quote! {
                #ops_::ControlFlow::Break(OurControlFlowResult::EarlyBreak(break_result))
            }),
        };

        let outer_label = match label {
            Some(label) => Lifetime::new(
                &format!("'__cbit_absorber_magic_for_{}", label.ident),
                label.span(),
            ),
            None => Lifetime::new("'__cbit_absorber_magic_regular", Span::call_site()),
        }
        .into_token_stream();

        let inner_label = label.map_or(TokenStream::new(), |label| quote! { #label: });

        quote! {#outer_label: {
            let mut did_run = false;
            let break_result = #inner_label loop {
                if did_run {
                    // The user must have used `continue`.
                    break #outer_label;
                }

                did_run = true;
                { #body };

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
        }}
    };

    let for_body = input.body.body;
    let for_body = wrap_absorber(for_body, None);

    // Build up an onion of user-specified break layers

    let for_body = {
        let mut for_body = for_body;
        for break_label in in_break_labels {
            for_body = wrap_absorber(for_body, Some(break_label));
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

    // Build up a list of break handlers
    let break_out_matchers = in_break_labels.iter().map(|label| {
        let variant_name = derive_early_break_variant_name(label);
        quote! {
            OurControlFlowResult::#variant_name(break_out) => break #label break_out,
        }
    });

    // Build up our function call site
    let driver_call_site = {
        let driver_attrs = &input.call.attrs;
        let driver_fn_expr = &input.call.func;
        let driver_fn_args = input.call.args.iter();

        quote! {
            #(#driver_attrs)*
            let result: #control_flow_ty_use = #driver_fn_expr (#(#driver_fn_args,)* #for_body);
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
            },
            #ops_::ControlFlow::Continue(result) => result,
        }
    }}
    .into()
}
