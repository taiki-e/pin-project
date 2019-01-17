use std::mem;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse_quote, Field, Fields, FieldsNamed, FieldsUnnamed, Generics, ItemStruct};

use crate::utils::*;

pub(super) fn unsafe_project(args: TokenStream, input: TokenStream) -> TokenStream {
    Struct::parse(args, input)
        .map(|parsed| TokenStream::from(parsed.proj_impl()))
        .unwrap_or_else(|e| e)
}

struct Struct {
    item: ItemStruct,
    impl_unpin: Option<Generics>,
    proj_ident: Ident,
    len: usize,
}

impl Struct {
    fn parse(args: TokenStream, input: TokenStream) -> Result<Self, TokenStream> {
        let item: ItemStruct = match syn::parse(input) {
            Err(_) => Err(compile_err("`unsafe_project` may only be used on structs"))?,
            Ok(item) => item,
        };

        let impl_unpin = match &*args.to_string() {
            "" => None,
            "Unpin" => Some(item.generics.clone()),
            _ => Err(compile_err(
                "`unsafe_project` an invalid argument was passed",
            ))?,
        };

        let len = match &item.fields {
            Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => named.len(),
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if !unnamed.is_empty() => unnamed.len(),
            Fields::Named(_) | Fields::Unnamed(_) => Err(err("zero fields"))?,
            Fields::Unit => Err(err("with units"))?,
        };

        let proj_ident = Ident::new(&format!("__{}Projection", item.ident), Span::call_site());
        Ok(Self {
            item,
            impl_unpin,
            proj_ident,
            len,
        })
    }

    fn proj_impl(mut self) -> TokenStream2 {
        let (proj_item, proj_init) = match &self.item.fields {
            Fields::Named(_) => self.named(),
            Fields::Unnamed(_) => self.unnamed(),
            Fields::Unit => unreachable!(),
        };

        let pin = pin();
        let unpin = unpin();
        let ident = &self.item.ident;
        let proj_ident = &self.proj_ident;
        let proj_generics = self.proj_generics();
        let (impl_generics, ty_generics, where_clause) = self.item.generics.split_for_impl();

        let proj_impl = quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                fn project<'__a>(self: #pin<&'__a mut Self>) -> #proj_ident #proj_generics {
                    let this = unsafe { #pin::get_unchecked_mut(self) };
                    #proj_init
                }
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
        item.extend(proj_item);
        item.extend(proj_impl);
        item.extend(impl_unpin);
        item
    }

    fn proj_generics(&self) -> TokenStream2 {
        let generics = self.item.generics.params.iter();
        quote!(<'__a, #(#generics),*>)
    }

    fn named(&mut self) -> (TokenStream2, TokenStream2) {
        let fields = match &mut self.item.fields {
            Fields::Named(FieldsNamed { named, .. }) => named,
            _ => unreachable!(),
        };

        let pin = pin();
        let unpin = unpin();
        let mut proj_fields = Vec::with_capacity(self.len);
        let mut proj_init = Vec::with_capacity(self.len);
        let mut impl_unpin = self.impl_unpin.take();
        fields.iter_mut().for_each(
            |Field {
                 attrs, ident, ty, ..
             }| {
                if find_remove(attrs, "pin").is_some() {
                    proj_fields.push(quote!(#ident: #pin<&'__a mut #ty>));
                    proj_init
                        .push(quote!(#ident: unsafe { #pin::new_unchecked(&mut this.#ident) }));

                    if let Some(generics) = &mut impl_unpin {
                        generics
                            .make_where_clause()
                            .predicates
                            .push(parse_quote!(#ty: #unpin));
                    }
                } else {
                    proj_fields.push(quote!(#ident: &'__a mut #ty));
                    proj_init.push(quote!(#ident: &mut this.#ident));
                }
            },
        );

        mem::replace(&mut self.impl_unpin, impl_unpin);
        let proj_ident = &self.proj_ident;
        let proj_generics = self.proj_generics();
        let proj_item = quote! {
            struct #proj_ident #proj_generics {
                #(#proj_fields,)*
            }
        };
        let proj_init = quote!(#proj_ident { #(#proj_init,)* });

        (proj_item, proj_init)
    }

    fn unnamed(&mut self) -> (TokenStream2, TokenStream2) {
        let fields = match &mut self.item.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
            _ => unreachable!(),
        };

        let pin = pin();
        let unpin = unpin();
        let mut proj_fields = Vec::with_capacity(self.len);
        let mut proj_init = Vec::with_capacity(self.len);
        let mut impl_unpin = self.impl_unpin.take();
        fields
            .iter_mut()
            .enumerate()
            .for_each(|(n, Field { attrs, ty, .. })| {
                if find_remove(attrs, "pin").is_some() {
                    proj_fields.push(quote!(#pin<&'__a mut #ty>));
                    proj_init.push(quote!(unsafe { #pin::new_unchecked(&mut this.#n) }));

                    if let Some(generics) = &mut impl_unpin {
                        generics
                            .make_where_clause()
                            .predicates
                            .push(parse_quote!(#ty: #unpin));
                    }
                } else {
                    proj_fields.push(quote!(&'__a mut #ty));
                    proj_init.push(quote!(&mut this.#n));
                }
            });

        mem::replace(&mut self.impl_unpin, impl_unpin);
        let proj_ident = &self.proj_ident;
        let proj_generics = self.proj_generics();
        let proj_item = quote! {
            struct #proj_ident #proj_generics(#(#proj_fields,)*);
        };
        let proj_init = quote!(#proj_ident(#(#proj_init,)*));

        (proj_item, proj_init)
    }
}
