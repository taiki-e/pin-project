use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, Result, Variant};

use crate::utils::collect_cfg;

use super::Context;

pub(super) fn parse(cx: &mut Context, mut item: ItemEnum) -> Result<TokenStream> {
    if item.variants.is_empty() {
        return Err(syn::Error::new(
            item.brace_token.span,
            "#[pin_project] attribute may not be used on enums without variants",
        ));
    }
    let has_field = item.variants.iter().try_fold(false, |has_field, v| {
        if let Some((_, e)) = &v.discriminant {
            Err(error!(e, "#[pin_project] attribute may not be used on enums with discriminants"))
        } else if let Fields::Unit = v.fields {
            Ok(has_field)
        } else {
            Ok(true)
        }
    })?;
    if !has_field {
        return Err(error!(
            item.variants,
            "#[pin_project] attribute may not be used on enums that have no field"
        ));
    }

    let (proj_variants, proj_arms) = variants(cx, &mut item)?;

    let proj_ident = &cx.proj_ident;
    let proj_generics = cx.proj_generics();
    let where_clause = item.generics.split_for_impl().2;

    let mut proj_items = quote! {
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
        #[allow(dead_code)] // This lint warns unused fields/variants.
        enum #proj_ident #proj_generics #where_clause { #(#proj_variants,)* }
    };

    let project_body = quote! {
        match self.as_mut().get_unchecked_mut() {
            #(#proj_arms,)*
        }
    };
    let project_into_body = quote! {
        match self.get_unchecked_mut() {
            #(#proj_arms,)*
        }
    };

    proj_items.extend(cx.make_proj_impl(&project_body, &project_into_body));

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}

fn variants(cx: &mut Context, item: &mut ItemEnum) -> Result<(Vec<TokenStream>, Vec<TokenStream>)> {
    let mut proj_variants = Vec::with_capacity(item.variants.len());
    let mut proj_arms = Vec::with_capacity(item.variants.len());
    for Variant { attrs, fields, ident, .. } in &mut item.variants {
        let (proj_pat, proj_body, proj_fields) = match fields {
            Fields::Unnamed(fields) => unnamed(cx, fields)?,
            Fields::Named(fields) => named(cx, fields)?,
            Fields::Unit => (TokenStream::new(), TokenStream::new(), TokenStream::new()),
        };
        let cfg = collect_cfg(attrs);
        let Context { orig_ident, proj_ident, .. } = &cx;
        proj_variants.push(quote! {
            #(#cfg)* #ident #proj_fields
        });
        proj_arms.push(quote! {
            #(#cfg)* #orig_ident::#ident #proj_pat => {
                #proj_ident::#ident #proj_body
            }
        });
    }

    Ok((proj_variants, proj_arms))
}

fn named(
    cx: &mut Context,
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
    let mut proj_pat = Vec::with_capacity(fields.len());
    let mut proj_body = Vec::with_capacity(fields.len());
    let mut proj_fields = Vec::with_capacity(fields.len());
    for Field { attrs, ident, ty, .. } in fields {
        let cfg = collect_cfg(attrs);
        if cx.find_pin_attr(attrs)? {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                #(#cfg)* #ident: ::core::pin::Pin<&#lifetime mut #ty>
            });
            proj_body.push(quote! {
                #(#cfg)* #ident: ::core::pin::Pin::new_unchecked(#ident)
            });
        } else {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                #(#cfg)* #ident: &#lifetime mut #ty
            });
            proj_body.push(quote! {
                #(#cfg)* #ident: #ident
            });
        }
        proj_pat.push(quote! {
            #(#cfg)* #ident
        });
    }

    let proj_pat = quote!({ #(#proj_pat),* });
    let proj_body = quote!({ #(#proj_body),* });
    let proj_fields = quote!({ #(#proj_fields),* });
    Ok((proj_pat, proj_body, proj_fields))
}

fn unnamed(
    cx: &mut Context,
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
    let mut proj_pat = Vec::with_capacity(fields.len());
    let mut proj_body = Vec::with_capacity(fields.len());
    let mut proj_fields = Vec::with_capacity(fields.len());
    for (i, Field { attrs, ty, .. }) in fields.iter_mut().enumerate() {
        let id = format_ident!("_x{}", i);
        let cfg = collect_cfg(attrs);
        if !cfg.is_empty() {
            return Err(error!(
                cfg.first(),
                "`cfg` attributes on the field of tuple variants are not supported"
            ));
        }
        if cx.find_pin_attr(attrs)? {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                ::core::pin::Pin<&#lifetime mut #ty>
            });
            proj_body.push(quote! {
                ::core::pin::Pin::new_unchecked(#id)
            });
        } else {
            let lifetime = &cx.lifetime;
            proj_fields.push(quote! {
                &#lifetime mut #ty
            });
            proj_body.push(quote! {
                #id
            });
        }
        proj_pat.push(quote! {
            #id
        });
    }

    let proj_pat = quote!((#(#proj_pat),*));
    let proj_body = quote!((#(#proj_body),*));
    let proj_fields = quote!((#(#proj_fields),*));
    Ok((proj_pat, proj_body, proj_fields))
}
