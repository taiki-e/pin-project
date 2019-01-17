use std::mem;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse_quote, Field, Fields, FieldsNamed, Generics, ItemStruct};

use crate::utils::*;

pub(super) fn unsafe_fields(args: TokenStream, input: TokenStream) -> TokenStream {
    Struct::parse(args, input)
        .map(|parsed| TokenStream::from(parsed.proj_impl()))
        .unwrap_or_else(|e| e)
}

struct Struct {
    item: ItemStruct,
    impl_unpin: Option<Generics>,
    len: usize,
}

impl Struct {
    fn parse(args: TokenStream, input: TokenStream) -> Result<Self, TokenStream> {
        let item: ItemStruct = match syn::parse(input) {
            Err(_) => Err(compile_err("`unsafe_fields` may only be used on structs"))?,
            Ok(item) => item,
        };

        let impl_unpin = match &*args.to_string() {
            "" => None,
            "Unpin" => Some(item.generics.clone()),
            _ => Err(compile_err(
                "`unsafe_fields` an invalid argument was passed",
            ))?,
        };

        let len = match &item.fields {
            Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => named.len(),
            Fields::Named(_) => Err(err("zero fields"))?,
            Fields::Unnamed(_) => Err(err("unnamed fields"))?,
            Fields::Unit => Err(err("with units"))?,
        };

        Ok(Self {
            item,
            impl_unpin,
            len,
        })
    }

    fn proj_impl(mut self) -> TokenStream2 {
        let proj_methods = match &self.item.fields {
            Fields::Named(_) => self.named(),
            _ => unreachable!(),
        };

        let unpin = unpin();
        let ident = &self.item.ident;
        let (impl_generics, ty_generics, where_clause) = self.item.generics.split_for_impl();
        let proj_impl = quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#proj_methods)*
            }
        };

        let impl_unpin = self
            .impl_unpin
            .as_ref()
            .map(|generics| {
                let where_clause = generics.split_for_impl().2;
                quote! {
                    impl #impl_generics #unpin for #ident #ty_generics #where_clause {}
                }
            })
            .unwrap_or_default();

        let mut item = self.item.into_token_stream();
        item.extend(proj_impl);
        item.extend(impl_unpin);
        item
    }

    fn named(&mut self) -> Vec<TokenStream2> {
        let fields = match &mut self.item.fields {
            Fields::Named(FieldsNamed { named, .. }) => named,
            _ => unreachable!(),
        };

        let pin = pin();
        let unpin = unpin();
        let mut proj_methods = Vec::with_capacity(self.len);
        let mut impl_unpin = self.impl_unpin.take();
        fields.iter_mut().for_each(
            |Field {
                 attrs, ident, ty, ..
             }| {
                if find_remove(attrs, "skip").is_none() {
                    if find_remove(attrs, "pin").is_some() {
                        proj_methods.push(quote! {
                            fn #ident<'__a>(self: #pin<&'__a mut Self>) -> #pin<&'__a mut #ty> {
                                unsafe { #pin::map_unchecked_mut(self, |x| &mut x.#ident) }
                            }
                        });

                        if let Some(generics) = &mut impl_unpin {
                            generics
                                .make_where_clause()
                                .predicates
                                .push(parse_quote!(#ty: #unpin));
                        }
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

        mem::replace(&mut self.impl_unpin, impl_unpin);
        proj_methods
    }
}
