use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, Result, Variant};

use crate::utils::{proj_ident, VecExt};

use super::*;

pub(super) fn parse(
    args: TokenStream,
    mut item: ItemEnum,
    pinned_drop: Option<ItemFn>,
) -> Result<TokenStream> {
    let impl_drop = ImplDrop::new(item.generics.clone(), pinned_drop)?;
    let mut impl_unpin = ImplUnpin::new(args, &item.generics)?;

    if item.variants.is_empty() {
        return Err(error!(item, "cannot be implemented for enums without variants"));
    } else if let Some(e) = item.variants.iter().find_map(|v| {
        v.discriminant
            .as_ref()
            .map(|(_, e)| error!(e, "cannot be implemented for enums with discriminants"))
    }) {
        return Err(e);
    }

    let proj_ident = proj_ident(&item.ident);
    let (proj_item_body, proj_arms) = variants(&mut item, &proj_ident, &mut impl_unpin);

    let ident = &item.ident;
    let proj_generics = proj_generics(&item.generics);
    let proj_ty_generics = proj_generics.split_for_impl().1;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut proj_items = quote! {
        enum #proj_ident #proj_generics #where_clause #proj_item_body
    };

    proj_items.extend(impl_drop.build(ident));
    proj_items.extend(impl_unpin.build(ident));
    proj_items.extend(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            fn project<'__a>(self: ::core::pin::Pin<&'__a mut Self>) -> #proj_ident #proj_ty_generics {
                unsafe {
                    match ::core::pin::Pin::get_unchecked_mut(self) {
                        #proj_arms
                    }
                }
            }
        }
    });

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}

fn variants(
    ItemEnum { variants, ident: enum_ident, .. }: &mut ItemEnum,
    proj_ident: &Ident,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream, TokenStream) {
    let mut arm_vec = Vec::with_capacity(variants.len());
    let mut ty_vec = Vec::with_capacity(variants.len());
    variants.iter_mut().for_each(|Variant { fields, ident, .. }| {
        let (proj_arm, proj_ty) = match fields {
            Fields::Unnamed(fields) => unnamed(fields, ident, enum_ident, proj_ident, impl_unpin),
            Fields::Named(fields) => named(fields, ident, enum_ident, proj_ident, impl_unpin),
            Fields::Unit => unit(ident, enum_ident, proj_ident),
        };

        arm_vec.push(proj_arm);
        ty_vec.push(proj_ty);
    });

    let proj_item_body = quote!({ #(#ty_vec,)* });
    let proj_arms = quote!(#(#arm_vec,)*);

    (proj_item_body, proj_arms)
}

fn named(
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    variant_ident: &Ident,
    enum_ident: &Ident,
    proj_ident: &Ident,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream, TokenStream) {
    let mut pat_vec = Vec::with_capacity(fields.len());
    let mut expr_vec = Vec::with_capacity(fields.len());
    let mut ty_vec = Vec::with_capacity(fields.len());
    fields.iter_mut().for_each(|Field { attrs, ident, ty, .. }| {
        if attrs.find_remove(PIN) {
            impl_unpin.push(ty);
            expr_vec.push(quote!(#ident: ::core::pin::Pin::new_unchecked(#ident)));
            ty_vec.push(quote!(#ident: ::core::pin::Pin<&'__a mut #ty>));
        } else {
            expr_vec.push(quote!(#ident: #ident));
            ty_vec.push(quote!(#ident: &'__a mut #ty));
        }

        pat_vec.push(ident);
    });

    let proj_arm = quote! {
        #enum_ident::#variant_ident { #(#pat_vec),* } => #proj_ident::#variant_ident { #(#expr_vec),* }
    };
    let proj_ty = quote!(#variant_ident { #(#ty_vec),* });

    (proj_arm, proj_ty)
}

fn unnamed(
    FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
    variant_ident: &Ident,
    enum_ident: &Ident,
    proj_ident: &Ident,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream, TokenStream) {
    let mut pat_vec = Vec::with_capacity(fields.len());
    let mut expr_vec = Vec::with_capacity(fields.len());
    let mut ty_vec = Vec::with_capacity(fields.len());
    fields.iter_mut().enumerate().for_each(|(i, Field { attrs, ty, .. })| {
        let x = Ident::new(&format!("_x{}", i), Span::call_site());

        if attrs.find_remove(PIN) {
            impl_unpin.push(ty);
            expr_vec.push(quote!(::core::pin::Pin::new_unchecked(#x)));
            ty_vec.push(quote!(::core::pin::Pin<&'__a mut #ty>));
        } else {
            expr_vec.push(quote!(#x));
            ty_vec.push(quote!(&'__a mut #ty));
        }

        pat_vec.push(x);
    });

    let proj_arm = quote! {
        #enum_ident::#variant_ident(#(#pat_vec),*) => #proj_ident::#variant_ident(#(#expr_vec),*)
    };
    let proj_ty = quote!(#variant_ident(#(#ty_vec),*));

    (proj_arm, proj_ty)
}

fn unit(
    variant_ident: &Ident,
    enum_ident: &Ident,
    proj_ident: &Ident,
) -> (TokenStream, TokenStream) {
    let proj_arm = quote! {
        #enum_ident::#variant_ident => #proj_ident::#variant_ident
    };
    let proj_ty = quote!(#variant_ident);

    (proj_arm, proj_ty)
}
