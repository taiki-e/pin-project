use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Nothing, Field, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, Result, Variant};

use crate::utils::VecExt;

use super::{proj_generics, Context, PIN};

pub(super) fn parse(mut cx: Context, mut item: ItemEnum) -> Result<TokenStream> {
    if item.variants.is_empty() {
        return Err(error!(item, "cannot be implemented for enums without variants"));
    }
    let has_field = item.variants.iter().try_fold(false, |has_field, v| {
        if let Some((_, e)) = &v.discriminant {
            Err(error!(e, "cannot be implemented for enums with discriminants"))
        } else if let Fields::Unit = v.fields {
            Ok(has_field)
        } else {
            Ok(true)
        }
    })?;
    if !has_field {
        return Err(error!(item.variants, "cannot be implemented for enums that have no field"));
    }

    let (proj_variants, proj_arms) = variants(&mut cx, &mut item)?;

    let impl_drop = cx.impl_drop(&item.generics);
    let Context { original, projected, lifetime, impl_unpin, .. } = cx;
    let proj_generics = proj_generics(&item.generics, &lifetime);
    let proj_ty_generics = proj_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut proj_items = quote! {
        enum #projected #proj_generics #where_clause { #(#proj_variants,)* }
    };
    let proj_method = quote! {
        impl #impl_generics #original #ty_generics #where_clause {
            fn project<#lifetime>(self: ::core::pin::Pin<&#lifetime mut Self>) -> #projected #proj_ty_generics {
                unsafe {
                    match ::core::pin::Pin::get_unchecked_mut(self) {
                        #(#proj_arms,)*
                    }
                }
            }
        }
    };

    proj_items.extend(impl_drop.build(&original));
    proj_items.extend(impl_unpin.build(&original));
    proj_items.extend(proj_method);

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}

fn variants(cx: &mut Context, item: &mut ItemEnum) -> Result<(Vec<TokenStream>, Vec<TokenStream>)> {
    let mut proj_variants = Vec::with_capacity(item.variants.len());
    let mut proj_arms = Vec::with_capacity(item.variants.len());
    for Variant { fields, ident, .. } in &mut item.variants {
        let (proj_pat, proj_body, proj_field) = match fields {
            Fields::Unnamed(fields) => unnamed(cx, fields)?,
            Fields::Named(fields) => named(cx, fields)?,
            Fields::Unit => (TokenStream::new(), TokenStream::new(), TokenStream::new()),
        };
        let Context { original, projected, .. } = &cx;
        let proj_arm = quote!(#original::#ident #proj_pat => #projected::#ident #proj_body );
        let proj_variant = quote!(#ident #proj_field);
        proj_arms.push(proj_arm);
        proj_variants.push(proj_variant);
    }

    Ok((proj_variants, proj_arms))
}

fn named(
    Context { lifetime, impl_unpin, .. }: &mut Context,
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
    let mut proj_pat = Vec::with_capacity(fields.len());
    let mut proj_body = Vec::with_capacity(fields.len());
    let mut proj_field = Vec::with_capacity(fields.len());
    for Field { attrs, ident, ty, .. } in fields {
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            impl_unpin.push(ty);
            proj_body.push(quote!(#ident: ::core::pin::Pin::new_unchecked(#ident)));
            proj_field.push(quote!(#ident: ::core::pin::Pin<&#lifetime mut #ty>));
        } else {
            proj_body.push(quote!(#ident));
            proj_field.push(quote!(#ident: &#lifetime mut #ty));
        }
        proj_pat.push(ident);
    }

    let proj_pat = quote!({ #(#proj_pat),* });
    let proj_body = quote!({ #(#proj_body),* });
    let proj_field = quote!({ #(#proj_field),* });
    Ok((proj_pat, proj_body, proj_field))
}

fn unnamed(
    Context { lifetime, impl_unpin, .. }: &mut Context,
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
) -> Result<(TokenStream, TokenStream, TokenStream)> {
    let mut proj_pat = Vec::with_capacity(fields.len());
    let mut proj_body = Vec::with_capacity(fields.len());
    let mut proj_field = Vec::with_capacity(fields.len());
    for (i, Field { attrs, ty, .. }) in fields.iter_mut().enumerate() {
        let x = format_ident!("_x{}", i);
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            impl_unpin.push(ty);
            proj_body.push(quote!(::core::pin::Pin::new_unchecked(#x)));
            proj_field.push(quote!(::core::pin::Pin<&#lifetime mut #ty>));
        } else {
            proj_body.push(quote!(#x));
            proj_field.push(quote!(&#lifetime mut #ty));
        }
        proj_pat.push(x);
    }

    let proj_pat = quote!((#(#proj_pat),*));
    let proj_body = quote!((#(#proj_body),*));
    let proj_field = quote!((#(#proj_field),*));
    Ok((proj_pat, proj_body, proj_field))
}
