use std::result;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Generics};

pub(super) type Result<T> = result::Result<T, TokenStream>;

#[inline(never)]
pub(super) fn compile_err(msg: &str) -> TokenStream {
    TokenStream::from(quote!(compile_error!(#msg);))
}

pub(super) fn pin() -> TokenStream2 {
    quote!(::core::pin::Pin)
}
pub(super) fn unpin() -> TokenStream2 {
    quote!(::core::marker::Unpin)
}

pub(super) fn find_remove(attrs: &mut Vec<Attribute>, ident: &str) -> Option<Attribute> {
    fn remove<T>(v: &mut Vec<T>, index: usize) -> T {
        match v.len() {
            1 => v.pop().unwrap(),
            2 => v.swap_remove(index),
            _ => v.remove(index),
        }
    }

    attrs
        .iter()
        .position(|Attribute { path, tts, .. }| path.is_ident(ident) && tts.is_empty())
        .map(|i| remove(attrs, i))
}

pub(super) fn parse_args(
    args: TokenStream,
    generics: &Generics,
    name: &str,
) -> Result<Option<Generics>> {
    match &*args.to_string() {
        "" => Ok(None),
        "Unpin" => Ok(Some(generics.clone())),
        _ => Err(compile_err(&format!(
            "`{}` an invalid argument was passed",
            name
        )))?,
    }
}
