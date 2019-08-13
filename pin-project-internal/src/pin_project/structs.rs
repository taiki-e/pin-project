use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Nothing, Field, Fields, FieldsNamed, FieldsUnnamed, Index, ItemStruct, Result};

use crate::utils::VecExt;

use super::{proj_generics, Context, PIN};

pub(super) fn parse(mut cx: Context, mut item: ItemStruct) -> Result<TokenStream> {
    let (proj_item_body, proj_init_body) = match &mut item.fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. })
            if fields.is_empty() =>
        {
            return Err(error!(item.fields, "cannot be implemented for structs with zero fields"))
        }
        Fields::Unit => return Err(error!(item, "cannot be implemented for structs with units")),

        Fields::Named(fields) => named(&mut cx, fields)?,
        Fields::Unnamed(fields) => unnamed(&mut cx, fields)?,
    };

    let orig_ident = &cx.original;
    let proj_ident = &cx.projected;
    let lifetime = &cx.lifetime;
    let impl_drop = cx.impl_drop(&item.generics);
    let proj_generics = proj_generics(&item.generics, &cx.lifetime);
    let proj_ty_generics = proj_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut proj_items = quote! {
        struct #proj_ident #proj_generics #where_clause #proj_item_body
    };
    let proj_method = quote! {
        impl #impl_generics #orig_ident #ty_generics #where_clause {
            fn project<#lifetime>(self: ::core::pin::Pin<&#lifetime mut Self>) -> #proj_ident #proj_ty_generics {
                unsafe {
                    let this = ::core::pin::Pin::get_unchecked_mut(self);
                    #proj_ident #proj_init_body
                }
            }
        }
    };

    proj_items.extend(impl_drop.build(orig_ident));
    proj_items.extend(cx.impl_unpin.build(orig_ident));
    proj_items.extend(proj_method);

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}

fn named(
    Context { lifetime, impl_unpin, .. }: &mut Context,
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
) -> Result<(TokenStream, TokenStream)> {
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    for Field { attrs, ident, ty, .. } in fields {
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            impl_unpin.push(ty);
            proj_fields.push(quote!(#ident: ::core::pin::Pin<&#lifetime mut #ty>));
            proj_init.push(quote!(#ident: ::core::pin::Pin::new_unchecked(&mut this.#ident)));
        } else {
            proj_fields.push(quote!(#ident: &#lifetime mut #ty));
            proj_init.push(quote!(#ident: &mut this.#ident));
        }
    }

    let proj_item_body = quote!({ #(#proj_fields,)* });
    let proj_init_body = quote!({ #(#proj_init,)* });
    Ok((proj_item_body, proj_init_body))
}

fn unnamed(
    Context { lifetime, impl_unpin, .. }: &mut Context,
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
) -> Result<(TokenStream, TokenStream)> {
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    for (i, Field { attrs, ty, .. }) in fields.iter_mut().enumerate() {
        let i = Index::from(i);
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            impl_unpin.push(ty);
            proj_fields.push(quote!(::core::pin::Pin<&#lifetime mut #ty>));
            proj_init.push(quote!(::core::pin::Pin::new_unchecked(&mut this.#i)));
        } else {
            proj_fields.push(quote!(&#lifetime mut #ty));
            proj_init.push(quote!(&mut this.#i));
        }
    }

    let proj_item_body = quote!((#(#proj_fields,)*););
    let proj_init_body = quote!((#(#proj_init,)*));
    Ok((proj_item_body, proj_init_body))
}
