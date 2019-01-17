use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Attribute;

#[inline(never)]
pub(super) fn compile_err(msg: &str) -> TokenStream {
    TokenStream::from(quote!(compile_error!(#msg);))
}

#[inline(never)]
pub(super) fn err(msg: &str) -> TokenStream {
    compile_err(&format!("cannot be implemented for structs with {}", msg))
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
