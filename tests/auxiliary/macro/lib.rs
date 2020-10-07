#![cfg(nightly)]
#![warn(rust_2018_idioms, single_use_lifetimes)]

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, Field, Fields, ItemStruct, Token, Visibility};

fn tokens2(tokens: TokenStream) -> proc_macro2::TokenStream {
    tokens.into()
}

#[proc_macro_attribute]
pub fn hidden_repr(args: TokenStream, input: TokenStream) -> TokenStream {
    let (args, input) = (tokens2(args), tokens2(input));
    quote!(#[repr(#args)] #input).into()
}

#[proc_macro]
pub fn hidden_repr_macro(input: TokenStream) -> TokenStream {
    let input = tokens2(input);
    quote!(#[repr(packed)] #input).into()
}

#[proc_macro_derive(HiddenRepr)]
pub fn hidden_repr_derive(_input: TokenStream) -> TokenStream {
    quote!(#[repr(packed)]).into()
}

#[proc_macro_attribute]
pub fn hidden_repr_cfg_not_any(args: TokenStream, input: TokenStream) -> TokenStream {
    let (args, input) = (tokens2(args), tokens2(input));
    quote!(#[cfg_attr(not(any()), repr(#args))] #input).into()
}

#[proc_macro_attribute]
pub fn add_pinned_field(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: ItemStruct = syn::parse_macro_input!(input);
    let fields = if let Fields::Named(fields) = &mut item.fields { fields } else { unreachable!() };
    fields.named.push(Field {
        attrs: vec![parse_quote!(#[pin])],
        vis: Visibility::Inherited,
        ident: Some(format_ident!("__field")),
        colon_token: Some(<Token![:]>::default()),
        ty: parse_quote!(::std::marker::PhantomPinned),
    });

    item.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn remove_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: ItemStruct = syn::parse_macro_input!(input);
    match &*args.to_string() {
        "field_all" => {
            if let Fields::Named(fields) = &mut item.fields { fields } else { unreachable!() }
                .named
                .iter_mut()
                .for_each(|field| field.attrs.clear())
        }
        "struct_all" => item.attrs.clear(),
        "struct_pin" => {
            while item
                .attrs
                .iter()
                .position(|a| a.path.is_ident("pin"))
                .map(|i| item.attrs.remove(i))
                .is_some()
            {}
        }
        _ => unreachable!(),
    }

    item.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn add_pin_attr(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: ItemStruct = syn::parse_macro_input!(input);
    assert_eq!(&*args.to_string(), "struct");
    item.attrs.push(parse_quote!(#[pin]));
    item.into_token_stream().into()
}
