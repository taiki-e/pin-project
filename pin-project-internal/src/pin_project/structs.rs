use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Nothing, Field, Fields, FieldsNamed, FieldsUnnamed, Index, ItemStruct, Result};

use crate::utils::VecExt;

use super::{Context, PIN};

pub(super) fn parse(cx: &mut Context, mut item: ItemStruct) -> Result<TokenStream> {
    let (proj_fields, proj_init) = match &mut item.fields {
        Fields::Named(FieldsNamed { named: fields, .. })
        | Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. })
            if fields.is_empty() =>
        {
            return Err(error!(
                item.fields,
                "#[pin_project] attribute may not be used on structs with zero fields"
            ))
        }
        Fields::Unit => {
            return Err(error!(
                item,
                "#[pin_project] attribute may not be used on structs with units"
            ))
        }

        Fields::Named(fields) => named(cx, fields)?,
        Fields::Unnamed(fields) => unnamed(cx, fields)?,
    };

    let proj_ident = &cx.proj_ident;
    let proj_generics = cx.proj_generics();
    let where_clause = item.generics.split_for_impl().2;

    let mut proj_items = quote! {
        #[allow(clippy::mut_mut)]
        #[allow(dead_code)]
        struct #proj_ident #proj_generics #where_clause #proj_fields
    };


    let project_body = quote! {
        unsafe {
            let this = self.as_mut().get_unchecked_mut();
            #proj_ident #proj_init
        }
    };

    let project_into_body = quote! {
        unsafe {
            let this = self.get_unchecked_mut();
            #proj_ident #proj_init
        }
    };

    proj_items.extend(cx.make_trait_impl(&project_body, &project_into_body));

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}

fn named(
    cx: &mut Context,
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
) -> Result<(TokenStream, TokenStream)> {
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    for Field { attrs, ident, ty, .. } in fields {
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            cx.push_unpin_bounds(ty.clone());
            let lifetime = &cx.lifetime;
            proj_fields.push(quote!(#ident: ::core::pin::Pin<&#lifetime mut #ty>));
            proj_init.push(quote!(#ident: ::core::pin::Pin::new_unchecked(&mut this.#ident)));
        } else {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote!(#ident: &#lifetime mut #ty));
            proj_init.push(quote!(#ident: &mut this.#ident));
        }
    }

    let proj_fields = quote!({ #(#proj_fields,)* });
    let proj_init = quote!({ #(#proj_init,)* });
    Ok((proj_fields, proj_init))
}

fn unnamed(
    cx: &mut Context,
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
) -> Result<(TokenStream, TokenStream)> {
    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    for (i, Field { attrs, ty, .. }) in fields.iter_mut().enumerate() {
        let i = Index::from(i);
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            cx.push_unpin_bounds(ty.clone());
            let lifetime = &cx.lifetime;
            proj_fields.push(quote!(::core::pin::Pin<&#lifetime mut #ty>));
            proj_init.push(quote!(::core::pin::Pin::new_unchecked(&mut this.#i)));
        } else {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote!(&#lifetime mut #ty));
            proj_init.push(quote!(&mut this.#i));
        }
    }

    let proj_fields = quote!((#(#proj_fields,)*););
    let proj_init = quote!((#(#proj_init,)*));
    Ok((proj_fields, proj_init))
}
