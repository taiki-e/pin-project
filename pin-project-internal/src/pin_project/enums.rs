use proc_macro2::{Ident, TokenStream};
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

    let (proj_item_body, proj_arms) = variants(&mut cx, &mut item)?;

    let orig_ident = &cx.original;
    let proj_ident = &cx.projected;
    let lifetime = &cx.lifetime;
    let impl_drop = cx.impl_drop(&item.generics);
    let proj_generics = proj_generics(&item.generics, &lifetime);
    let proj_ty_generics = proj_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut proj_items = quote! {
        enum #proj_ident #proj_generics #where_clause #proj_item_body
    };
    let proj_method = quote! {
        impl #impl_generics #orig_ident #ty_generics #where_clause {
            fn project<#lifetime>(self: ::core::pin::Pin<&#lifetime mut Self>) -> #proj_ident #proj_ty_generics {
                unsafe {
                    match ::core::pin::Pin::get_unchecked_mut(self) {
                        #proj_arms
                    }
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

fn variants(
    cx: &mut Context,
    ItemEnum { variants, .. }: &mut ItemEnum,
) -> Result<(TokenStream, TokenStream)> {
    let mut arm_vec = Vec::with_capacity(variants.len());
    let mut ty_vec = Vec::with_capacity(variants.len());
    for Variant { fields, ident, .. } in variants {
        let (proj_arm, proj_ty) = match fields {
            Fields::Unnamed(fields) => unnamed(cx, fields, ident)?,
            Fields::Named(fields) => named(cx, fields, ident)?,
            Fields::Unit => unit(cx, ident),
        };
        arm_vec.push(proj_arm);
        ty_vec.push(proj_ty);
    }

    let proj_item_body = quote!({ #(#ty_vec,)* });
    let proj_arms = quote!(#(#arm_vec,)*);
    Ok((proj_item_body, proj_arms))
}

fn named(
    Context { original, projected, lifetime, impl_unpin, .. }: &mut Context,
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    variant_ident: &Ident,
) -> Result<(TokenStream, TokenStream)> {
    let mut pat_vec = Vec::with_capacity(fields.len());
    let mut expr_vec = Vec::with_capacity(fields.len());
    let mut ty_vec = Vec::with_capacity(fields.len());
    for Field { attrs, ident, ty, .. } in fields {
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            impl_unpin.push(ty);
            expr_vec.push(quote!(#ident: ::core::pin::Pin::new_unchecked(#ident)));
            ty_vec.push(quote!(#ident: ::core::pin::Pin<&#lifetime mut #ty>));
        } else {
            expr_vec.push(quote!(#ident: #ident));
            ty_vec.push(quote!(#ident: &#lifetime mut #ty));
        }
        pat_vec.push(ident);
    }

    let proj_arm = quote! {
        #original::#variant_ident { #(#pat_vec),* } => #projected::#variant_ident { #(#expr_vec),* }
    };
    let proj_ty = quote!(#variant_ident { #(#ty_vec),* });
    Ok((proj_arm, proj_ty))
}

fn unnamed(
    Context { original, projected, lifetime, impl_unpin, .. }: &mut Context,
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
    variant_ident: &Ident,
) -> Result<(TokenStream, TokenStream)> {
    let mut pat_vec = Vec::with_capacity(fields.len());
    let mut expr_vec = Vec::with_capacity(fields.len());
    let mut ty_vec = Vec::with_capacity(fields.len());
    for (i, Field { attrs, ty, .. }) in fields.iter_mut().enumerate() {
        let x = format_ident!("_x{}", i);
        if let Some(attr) = attrs.find_remove(PIN) {
            let _: Nothing = syn::parse2(attr.tokens)?;
            impl_unpin.push(ty);
            expr_vec.push(quote!(::core::pin::Pin::new_unchecked(#x)));
            ty_vec.push(quote!(::core::pin::Pin<&#lifetime mut #ty>));
        } else {
            expr_vec.push(quote!(#x));
            ty_vec.push(quote!(&#lifetime mut #ty));
        }
        pat_vec.push(x);
    }

    let proj_arm = quote! {
        #original::#variant_ident(#(#pat_vec),*) => #projected::#variant_ident(#(#expr_vec),*)
    };
    let proj_ty = quote!(#variant_ident(#(#ty_vec),*));
    Ok((proj_arm, proj_ty))
}

fn unit(
    Context { original, projected, .. }: &mut Context,
    variant_ident: &Ident,
) -> (TokenStream, TokenStream) {
    let proj_arm = quote! {
        #original::#variant_ident => #projected::#variant_ident
    };
    let proj_ty = quote!(#variant_ident);
    (proj_arm, proj_ty)
}
