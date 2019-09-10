// force-host
// no-prefer-dynamic

#![crate_type = "proc-macro"]

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn hidden_repr(attr: TokenStream, item: TokenStream) -> TokenStream {
    format!("#[repr({})] {}", attr, item).parse().unwrap()
}

#[proc_macro]
pub fn hidden_repr_macro(item: TokenStream) -> TokenStream {
    format!("#[repr(packed)] {}", item).parse().unwrap()
}
