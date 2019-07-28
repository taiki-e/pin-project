use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, Index, ItemStruct, Result};

use crate::utils::{proj_ident, VecExt};

use super::*;

pub(super) fn parse(
    args: TokenStream,
    mut item: ItemStruct,
    pinned_drop: Option<ItemFn>,
) -> Result<TokenStream> {
    let impl_drop = ImplDrop::new(item.generics.clone(), pinned_drop)?;
    let mut impl_unpin = ImplUnpin::new(args, &item.generics)?;

    let (proj_item_body, proj_init_body) = match &mut item.fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. })
            if fields.is_empty() =>
        {
            return Err(error!(item.fields, "cannot be implemented for structs with zero fields"))
        }
        Fields::Unit => return Err(error!(item, "cannot be implemented for structs with units")),

        Fields::Named(fields) => named(fields, &mut impl_unpin),
        Fields::Unnamed(fields) => unnamed(fields, &mut impl_unpin),
    };

    let ident = &item.ident;
    let proj_ident = proj_ident(&item.ident);
    let proj_generics = proj_generics(&item.generics);
    let proj_ty_generics = proj_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut proj_items = quote! {
        struct #proj_ident #proj_generics #where_clause #proj_item_body
    };

    proj_items.extend(impl_drop.build(ident));
    proj_items.extend(impl_unpin.build(ident));
    proj_items.extend(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            fn project<'__a>(self: ::core::pin::Pin<&'__a mut Self>) -> #proj_ident #proj_ty_generics {
                unsafe {
                    let this = ::core::pin::Pin::get_unchecked_mut(self);
                    #proj_ident #proj_init_body
                }
            }
        }
    });

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}

fn named(
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream, TokenStream) {
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    fields.iter_mut().for_each(|Field { attrs, ident, ty, .. }| {
        if attrs.find_remove(PIN) {
            impl_unpin.push(ty);
            proj_fields.push(quote!(#ident: ::core::pin::Pin<&'__a mut #ty>));
            proj_init.push(quote!(#ident: ::core::pin::Pin::new_unchecked(&mut this.#ident)));
        } else {
            proj_fields.push(quote!(#ident: &'__a mut #ty));
            proj_init.push(quote!(#ident: &mut this.#ident));
        }
    });

    let proj_item_body = quote!({ #(#proj_fields,)* });
    let proj_init_body = quote!({ #(#proj_init,)* });

    (proj_item_body, proj_init_body)
}

fn unnamed(
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream, TokenStream) {
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    fields.iter_mut().enumerate().for_each(|(i, Field { attrs, ty, .. })| {
        let i = Index::from(i);
        if attrs.find_remove(PIN) {
            impl_unpin.push(ty);
            proj_fields.push(quote!(::core::pin::Pin<&'__a mut #ty>));
            proj_init.push(quote!(::core::pin::Pin::new_unchecked(&mut this.#i)));
        } else {
            proj_fields.push(quote!(&'__a mut #ty));
            proj_init.push(quote!(&mut this.#i));
        }
    });

    let proj_item_body = quote!((#(#proj_fields,)*););
    let proj_init_body = quote!((#(#proj_init,)*));

    (proj_item_body, proj_init_body)
}
