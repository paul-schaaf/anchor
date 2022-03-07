use crate::{Error, ErrorArgs, ErrorCode};
use syn::parse::{Parse, Result as ParseResult};
use syn::{Expr, Token};

// Removes any internal #[msg] attributes, as they are inert.
pub fn parse(error_enum: &mut syn::ItemEnum, args: Option<ErrorArgs>) -> Error {
    let ident = error_enum.ident.clone();
    let mut last_discriminant = 0;
    let codes: Vec<ErrorCode> = error_enum
        .variants
        .iter_mut()
        .map(|variant: &mut syn::Variant| {
            let msg = parse_error_attribute(variant);
            let ident = variant.ident.clone();
            let id = match &variant.discriminant {
                None => last_discriminant,
                Some((_, disc)) => match disc {
                    syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                        syn::Lit::Int(int) => {
                            int.base10_parse::<u32>().expect("Must be a base 10 number")
                        }
                        _ => panic!("Invalid error discriminant"),
                    },
                    _ => panic!("Invalid error discriminant"),
                },
            };
            last_discriminant = id + 1;

            // Remove any non-doc attributes on the error variant.
            variant.attrs = variant
                .attrs
                .iter()
                .filter(|attr| attr.path.segments[0].ident == "doc")
                .cloned()
                .collect();

            ErrorCode { id, ident, msg }
        })
        .collect();
    Error {
        name: error_enum.ident.to_string(),
        raw_enum: error_enum.clone(),
        ident,
        codes,
        args,
    }
}

fn parse_error_attribute(variant: &syn::Variant) -> Option<String> {
    let attrs = variant
        .attrs
        .iter()
        .filter(|attr| attr.path.segments[0].ident != "doc")
        .collect::<Vec<_>>();
    match attrs.len() {
        0 => None,
        1 => {
            let attr = &attrs[0];
            let attr_str = attr.path.segments[0].ident.to_string();
            assert!(&attr_str == "msg", "Use msg to specify error strings");

            let mut tts = attr.tokens.clone().into_iter();
            let g_stream = match tts.next().expect("Must have a token group") {
                proc_macro2::TokenTree::Group(g) => g.stream(),
                _ => panic!("Invalid syntax"),
            };

            let msg = match g_stream.into_iter().next() {
                None => panic!("Must specify a message string"),
                Some(msg) => msg.to_string().replace('\"', ""),
            };

            Some(msg)
        }
        _ => {
            panic!("Too many attributes found. Use `msg` to specify error strings");
        }
    }
}

pub struct ErrorInput {
    pub error_code: Expr,
    pub expected_actual: Option<ExpectedActualInput>,
}

impl Parse for ErrorInput {
    fn parse(stream: syn::parse::ParseStream) -> ParseResult<Self> {
        let error_code = stream.call(Expr::parse)?;
        let expected_actual = match stream.parse::<Token!(,)>() {
            Ok(_) => Some(stream.parse::<ExpectedActualInput>()?),
            _ => None,
        };
        Ok(Self {
            error_code,
            expected_actual,
        })
    }
}

pub struct ErrorWithAccountNameInput {
    pub error_code: Expr,
    pub account_name: Expr,
    pub expected_actual: Option<ExpectedActualInput>,
}

impl Parse for ErrorWithAccountNameInput {
    fn parse(stream: syn::parse::ParseStream) -> ParseResult<Self> {
        let error_code = stream.call(Expr::parse)?;
        let _ = stream.parse::<Token!(,)>();
        let account_name = stream.call(Expr::parse)?;
        let expected_actual = match stream.parse::<Token!(,)>() {
            Ok(_) => Some(stream.parse::<ExpectedActualInput>()?),
            _ => None,
        };
        Ok(Self {
            error_code,
            account_name,
            expected_actual,
        })
    }
}

pub struct ExpectedActualInput {
    pub pubkeys: bool,
    pub expected: Expr,
    pub actual: Expr,
}

impl Parse for ExpectedActualInput {
    fn parse(stream: syn::parse::ParseStream) -> ParseResult<Self> {
        let expected = stream.call(Expr::parse)?;
        let _ = stream.parse::<Token!(,)>();
        let actual = stream.call(Expr::parse)?;
        Ok(Self {
            pubkeys: false,
            expected,
            actual,
        })
    }
}
