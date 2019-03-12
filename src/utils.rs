use std::result;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Attribute;

pub(crate) type Result<T> = result::Result<T, TokenStream>;

/// Makes the ident of projected type from the reference of the original ident.
pub(crate) fn proj_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("__{}Projection", ident), Span::call_site())
}

#[inline(never)]
pub(crate) fn failed<T>(name: &str, msg: &str) -> Result<T> {
    #[inline(never)]
    fn compile_err(msg: &str) -> TokenStream {
        quote!(compile_error!(#msg);)
    }

    Err(compile_err(&format!("`{}` {}", name, msg)))
}

pub(crate) trait VecExt {
    fn find_remove(&mut self, ident: &str) -> bool;
}

impl VecExt for Vec<Attribute> {
    fn find_remove(&mut self, ident: &str) -> bool {
        self.iter()
            .position(|Attribute { path, tts, .. }| path.is_ident(ident) && tts.is_empty())
            .map(|i| self.remove(i))
            .is_some()
    }
}
