#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes, unreachable_pub)]
#![warn(clippy::all)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use syn::*;

#[proc_macro_attribute]
pub fn hidden_repr(args: TokenStream, input: TokenStream) -> TokenStream {
    format!("#[repr({})] {}", args, input).parse().unwrap()
}

#[proc_macro]
pub fn hidden_repr_macro(input: TokenStream) -> TokenStream {
    format!("#[repr(packed)] {}", input).parse().unwrap()
}

#[proc_macro_attribute]
pub fn hidden_repr_cfg_not_any(args: TokenStream, input: TokenStream) -> TokenStream {
    format!("#[cfg_attr(not(any()), repr({}))] {}", args, input).parse().unwrap()
}

#[proc_macro_attribute]
pub fn add_pinned_field(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(input as ItemStruct);
    let fields = if let Fields::Named(fields) = &mut item.fields { fields } else { unreachable!() };
    fields.named.push(Field {
        attrs: vec![syn::parse_quote!(#[pin])],
        vis: Visibility::Inherited,
        ident: Some(format_ident!("__field")),
        colon_token: Some(<Token![:]>::default()),
        ty: syn::parse_quote!(::std::marker::PhantomPinned),
    });
    item.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn remove_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(input as ItemStruct);
    match &*args.to_string() {
        "field" => {
            if let Fields::Named(fields) = &mut item.fields { fields } else { unreachable!() }
                .named
                .iter_mut()
                .for_each(|field| field.attrs.clear())
        }
        "struct" => item.attrs.clear(),
        _ => unreachable!(),
    }

    item.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn add_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(input as ItemStruct);
    assert_eq!(&*args.to_string(), "struct");
    item.attrs.push(syn::parse_quote!(#[pin]));
    item.into_token_stream().into()
}
