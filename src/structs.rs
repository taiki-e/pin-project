use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, ItemStruct};

use crate::utils::*;

pub(super) fn unsafe_project(args: TokenStream, item: ItemStruct) -> TokenStream {
    Struct::parse(args, item)
        .map(|parsed| TokenStream::from(parsed.proj_impl()))
        .unwrap_or_else(|e| e)
}

struct Struct {
    item: ItemStruct,
    impl_unpin: ImplUnpin,
    proj_ident: Ident,
}

impl Struct {
    fn parse(args: TokenStream, item: ItemStruct) -> Result<Self> {
        match &item.fields {
            Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => {}
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if !unnamed.is_empty() => {}
            Fields::Named(_) | Fields::Unnamed(_) => {
                parse_failed("unsafe_project", "structs with zero fields")?
            }
            Fields::Unit => parse_failed("unsafe_project", "structs with units")?,
        }

        Ok(Self {
            impl_unpin: ImplUnpin::parse(args, &item.generics, "unsafe_project")?,
            proj_ident: proj_ident(&item.ident),
            item,
        })
    }

    fn proj_impl(mut self) -> TokenStream2 {
        let (proj_item, proj_init) = match &self.item.fields {
            Fields::Named(_) => self.named(),
            Fields::Unnamed(_) => self.unnamed(),
            Fields::Unit => unreachable!(),
        };

        let pin = pin();
        let ident = &self.item.ident;
        let proj_ident = &self.proj_ident;
        let proj_generics = proj_generics(&self.item.generics);
        let (impl_generics, ty_generics, where_clause) = self.item.generics.split_for_impl();

        let proj_impl = quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                fn project<'__a>(self: #pin<&'__a mut Self>) -> #proj_ident #proj_generics {
                    unsafe {
                        let this = #pin::get_unchecked_mut(self);
                        #proj_init
                    }
                }
            }
        };

        let impl_unpin = self.impl_unpin.build(impl_generics, ident, ty_generics);
        let mut item = self.item.into_token_stream();
        item.extend(proj_item);
        item.extend(proj_impl);
        item.extend(impl_unpin);
        item
    }

    fn named(&mut self) -> (TokenStream2, TokenStream2) {
        let fields = match &mut self.item.fields {
            Fields::Named(FieldsNamed { named, .. }) => named,
            _ => unreachable!(),
        };

        let pin = pin();
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_init = Vec::with_capacity(fields.len());
        let mut impl_unpin = self.impl_unpin.take();
        fields.iter_mut().for_each(
            |Field {
                 attrs, ident, ty, ..
             }| {
                if find_remove(attrs, "pin").is_some() {
                    impl_unpin.push(ty);
                    proj_fields.push(quote!(#ident: #pin<&'__a mut #ty>));
                    proj_init.push(quote!(#ident: #pin::new_unchecked(&mut this.#ident)));
                } else {
                    proj_fields.push(quote!(#ident: &'__a mut #ty));
                    proj_init.push(quote!(#ident: &mut this.#ident));
                }
            },
        );
        self.impl_unpin = impl_unpin;

        let proj_ident = &self.proj_ident;
        let proj_generics = proj_generics(&self.item.generics);
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
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_init = Vec::with_capacity(fields.len());
        let mut impl_unpin = self.impl_unpin.take();
        fields
            .iter_mut()
            .enumerate()
            .for_each(|(n, Field { attrs, ty, .. })| {
                if find_remove(attrs, "pin").is_some() {
                    impl_unpin.push(ty);
                    proj_fields.push(quote!(#pin<&'__a mut #ty>));
                    proj_init.push(quote!(#pin::new_unchecked(&mut this.#n)));
                } else {
                    proj_fields.push(quote!(&'__a mut #ty));
                    proj_init.push(quote!(&mut this.#n));
                }
            });
        self.impl_unpin = impl_unpin;

        let proj_ident = &self.proj_ident;
        let proj_generics = proj_generics(&self.item.generics);
        let proj_item = quote! {
            struct #proj_ident #proj_generics(#(#proj_fields,)*);
        };
        let proj_init = quote!(#proj_ident(#(#proj_init,)*));

        (proj_item, proj_init)
    }
}
