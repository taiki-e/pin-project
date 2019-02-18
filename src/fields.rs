use std::convert::identity;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, ItemStruct};

use crate::utils::*;

pub(super) fn unsafe_fields(args: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse(input)
        .map_err(|_| compile_err("`unsafe_fields` may only be used on structs"))
        .and_then(|item| Struct::parse(args, item))
        .map(|parsed| TokenStream::from(parsed.proj_impl()))
        .unwrap_or_else(identity)
}

struct Struct {
    item: ItemStruct,
    impl_unpin: ImplUnpin,
}

impl Struct {
    fn parse(args: TokenStream, item: ItemStruct) -> Result<Self> {
        match &item.fields {
            Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => {}
            Fields::Named(_) => parse_failed("unsafe_fields", "structs with zero fields")?,
            Fields::Unnamed(_) => parse_failed("unsafe_fields", "structs with unnamed fields")?,
            Fields::Unit => parse_failed("unsafe_fields", "structs with units")?,
        }

        Ok(Self {
            impl_unpin: ImplUnpin::parse(args, &item.generics, "unsafe_fields")?,
            item,
        })
    }

    fn proj_impl(self) -> TokenStream2 {
        let Self {
            mut item,
            mut impl_unpin,
        } = self;

        let proj_methods = match &mut item.fields {
            Fields::Named(fields) => named(fields, &mut impl_unpin),
            _ => unreachable!(),
        };

        let ident = &item.ident;
        let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
        let proj_impl = quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#proj_methods)*
            }
        };

        let impl_unpin = impl_unpin.build(impl_generics, ident, ty_generics);
        let mut item = item.into_token_stream();
        item.extend(proj_impl);
        item.extend(impl_unpin);
        item
    }
}

fn named(
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    impl_unpin: &mut ImplUnpin,
) -> Vec<TokenStream2> {
    let pin = pin();
    let mut proj_methods = Vec::with_capacity(fields.len());
    fields.iter_mut().for_each(
        |Field {
             attrs, ident, ty, ..
         }| {
            if find_remove(attrs, "skip").is_none() {
                if find_remove(attrs, "pin").is_some() {
                    impl_unpin.push(ty);
                    proj_methods.push(quote! {
                        fn #ident<'__a>(self: #pin<&'__a mut Self>) -> #pin<&'__a mut #ty> {
                            unsafe { #pin::map_unchecked_mut(self, |x| &mut x.#ident) }
                        }
                    });
                } else {
                    proj_methods.push(quote! {
                        fn #ident<'__a>(self: #pin<&'__a mut Self>) -> &'__a mut #ty {
                            unsafe { &mut #pin::get_unchecked_mut(self).#ident }
                        }
                    });
                }
            }
        },
    );

    proj_methods
}
