use std::convert::identity;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, ItemStruct};

use crate::utils::{Result, *};

use super::{NAME, PIN};

pub(super) fn unsafe_project(args: TokenStream, item: ItemStruct) -> TokenStream {
    parse(args, item).unwrap_or_else(identity)
}

fn parse(args: TokenStream, item: ItemStruct) -> Result<TokenStream> {
    match &item.fields {
        Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => {}
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if !unnamed.is_empty() => {}
        Fields::Named(_) | Fields::Unnamed(_) => parse_failed(NAME, "structs with zero fields")?,
        Fields::Unit => parse_failed(NAME, "structs with units")?,
    }

    ImplUnpin::parse(args, &item.generics, NAME).map(|impl_unpin| proj_impl(item, impl_unpin))
}

fn proj_impl(mut item: ItemStruct, mut impl_unpin: ImplUnpin) -> TokenStream {
    let (proj_item_body, proj_init_body) = match &mut item.fields {
        Fields::Named(fields) => named(fields, &mut impl_unpin),
        Fields::Unnamed(fields) => unnamed(fields, &mut impl_unpin),
        Fields::Unit => unreachable!(),
    };

    let pin = pin();
    let ident = &item.ident;
    let proj_ident = proj_ident(&item.ident);
    let proj_generics = proj_generics(&item.generics);
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let proj_ty_generics = proj_generics.split_for_impl().1;

    let proj_item = quote! {
        struct #proj_ident #proj_generics #where_clause #proj_item_body
    };

    let proj_impl = quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            fn project<'__a>(self: #pin<&'__a mut Self>) -> #proj_ident #proj_ty_generics {
                unsafe {
                    let this = #pin::get_unchecked_mut(self);
                    #proj_ident #proj_init_body
                }
            }
        }
    };

    let impl_unpin = impl_unpin.build(ident);
    let mut item = item.into_token_stream();
    item.extend(proj_item);
    item.extend(proj_impl);
    item.extend(impl_unpin);
    item
}

fn named(
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream, TokenStream) {
    let pin = pin();
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    fields.iter_mut().for_each(
        |Field {
             attrs, ident, ty, ..
         }| {
            if find_remove(attrs, PIN) {
                impl_unpin.push(ty);
                proj_fields.push(quote!(#ident: #pin<&'__a mut #ty>));
                proj_init.push(quote!(#ident: #pin::new_unchecked(&mut this.#ident)));
            } else {
                proj_fields.push(quote!(#ident: &'__a mut #ty));
                proj_init.push(quote!(#ident: &mut this.#ident));
            }
        },
    );

    let proj_item_body = quote!({ #(#proj_fields,)* });
    let proj_init_body = quote!({ #(#proj_init,)* });

    (proj_item_body, proj_init_body)
}

fn unnamed(
    FieldsUnnamed {
        unnamed: fields, ..
    }: &mut FieldsUnnamed,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream, TokenStream) {
    let pin = pin();
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    fields
        .iter_mut()
        .enumerate()
        .for_each(|(n, Field { attrs, ty, .. })| {
            if find_remove(attrs, PIN) {
                impl_unpin.push(ty);
                proj_fields.push(quote!(#pin<&'__a mut #ty>));
                proj_init.push(quote!(#pin::new_unchecked(&mut this.#n)));
            } else {
                proj_fields.push(quote!(&'__a mut #ty));
                proj_init.push(quote!(&mut this.#n));
            }
        });

    let proj_item_body = quote!((#(#proj_fields,)*););
    let proj_init_body = quote!((#(#proj_init,)*));

    (proj_item_body, proj_init_body)
}
