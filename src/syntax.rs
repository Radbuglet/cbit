use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Brace,
    Expr, ExprCall, ExprMethodCall, Label, Lifetime, Pat, Token,
};

#[derive(Clone)]
pub struct CbitForExpr {
    pub label: Option<Label>,
    pub kw_for: Token![for],
    pub body_pattern: Pat,
    pub kw_in: Token![in],
    pub call: AnyCallExpr,
    pub breaks: Option<CbitForExprBreaks>,
    pub body: OpaqueBody,
}

impl Parse for CbitForExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            label: input.parse()?,
            kw_for: input.parse()?,
            body_pattern: Pat::parse_single(input)?,
            kw_in: input.parse()?,
            call: input.parse()?,
            breaks: CbitForExprBreaks::parse(input)?,
            body: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub struct CbitForExprBreaks {
    pub kw_break: Token![break],
    pub lt: Punctuated<CbitForExprSingleBreak, Token![,]>,
}

impl CbitForExprBreaks {
    pub fn parse(input: ParseStream) -> syn::Result<Option<Self>> {
        let Ok(kw_break) = input.parse::<Token![break]>() else {
            return Ok(None);
        };

        Ok(Some(Self {
            kw_break,
            lt: Punctuated::parse_separated_nonempty(input)?,
        }))
    }
}

#[derive(Clone)]
pub struct CbitForExprSingleBreak {
    pub kw_loop: Option<Token![loop]>,
    pub lt: Lifetime,
}

impl Parse for CbitForExprSingleBreak {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            kw_loop: input.parse()?,
            lt: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub enum AnyCallExpr {
    Function(ExprCall),
    Method(ExprMethodCall),
}

impl Parse for AnyCallExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<Expr>()? {
            Expr::Call(func) => Ok(Self::Function(func)),
            Expr::MethodCall(method) => Ok(Self::Method(method)),
            _ => Err(input.error("expected a function or method call")),
        }
    }
}

#[derive(Clone)]
pub struct OpaqueBody {
    pub brace: Brace,
    pub body: TokenStream,
}

impl Parse for OpaqueBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let body;
        Ok(Self {
            brace: braced!(body in input),
            body: body.parse()?,
        })
    }
}
