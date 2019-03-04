use std::result;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Attribute;

pub(super) type Result<T> = result::Result<T, TokenStream>;

/// Makes the ident of projected type from the reference of the original ident.
pub(super) fn proj_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("__{}Projection", ident), Span::call_site())
}

#[inline(never)]
pub(super) fn failed<T>(name: &str, msg: &str) -> Result<T> {
    #[inline(never)]
    fn compile_err(msg: &str) -> TokenStream {
        quote!(compile_error!(#msg);)
    }

    Err(compile_err(&format!("`{}` {}", name, msg)))
}

pub(super) fn find_remove(attrs: &mut Vec<Attribute>, ident: &str) -> bool {
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
        .is_some()
}
