use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Index, ItemStruct, Result};

use crate::utils::collect_cfg;

use super::Context;

pub(super) fn validate(ident: &Ident, fields: &Fields) -> Result<()> {
    match fields {
        Fields::Named(FieldsNamed { named: f, .. })
        | Fields::Unnamed(FieldsUnnamed { unnamed: f, .. })
            if f.is_empty() =>
        {
            Err(error!(
                fields,
                "#[pin_project] attribute may not be used on structs with zero fields"
            ))
        }
        Fields::Unit => {
            Err(error!(ident, "#[pin_project] attribute may not be used on structs with units"))
        }
        _ => Ok(()),
    }
}

pub(super) fn parse(cx: &mut Context, mut item: ItemStruct) -> Result<TokenStream> {
    validate(&item.ident, &item.fields)?;

    let (proj_fields, proj_init) = match &mut item.fields {
        Fields::Named(fields) => named(cx, fields)?,
        Fields::Unnamed(fields) => unnamed(cx, fields)?,
        Fields::Unit => unreachable!(),
    };

    let proj_ident = &cx.proj_ident;
    let proj_generics = cx.proj_generics();
    let where_clause = item.generics.split_for_impl().2;

    let mut proj_items = quote! {
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
        #[allow(dead_code)] // This lint warns unused fields/variants.
        struct #proj_ident #proj_generics #where_clause #proj_fields
    };

    let project_body = quote! {
        let this = self.as_mut().get_unchecked_mut();
        #proj_ident #proj_init
    };
    let project_into_body = quote! {
        let this = self.get_unchecked_mut();
        #proj_ident #proj_init
    };

    proj_items.extend(cx.make_proj_impl(&project_body, &project_into_body));

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
        let cfg = collect_cfg(attrs);
        if cx.find_pin_attr(attrs)? {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                #(#cfg)* #ident: ::core::pin::Pin<&#lifetime mut #ty>
            });
            proj_init.push(quote! {
                #(#cfg)* #ident: ::core::pin::Pin::new_unchecked(&mut this.#ident)
            });
        } else {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                #(#cfg)* #ident: &#lifetime mut #ty
            });
            proj_init.push(quote! {
                #(#cfg)* #ident: &mut this.#ident
            });
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
    for (index, Field { attrs, ty, .. }) in fields.iter_mut().enumerate() {
        let index = Index::from(index);
        let cfg = collect_cfg(attrs);
        if !cfg.is_empty() {
            return Err(error!(
                cfg.first(),
                "`cfg` attributes on the field of tuple structs are not supported"
            ));
        }
        if cx.find_pin_attr(attrs)? {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                ::core::pin::Pin<&#lifetime mut #ty>
            });
            proj_init.push(quote! {
                ::core::pin::Pin::new_unchecked(&mut this.#index)
            });
        } else {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                &#lifetime mut #ty
            });
            proj_init.push(quote! {
                &mut this.#index
            });
        }
    }

    let proj_fields = quote!((#(#proj_fields,)*););
    let proj_init = quote!((#(#proj_init,)*));
    Ok((proj_fields, proj_init))
}
