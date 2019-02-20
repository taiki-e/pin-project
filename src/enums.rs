use std::convert::identity;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, Variant};

use crate::utils::*;

pub(super) fn unsafe_project(args: TokenStream, item: ItemEnum) -> TokenStream {
    Enum::parse(args, item)
        .map(|parsed| TokenStream::from(parsed.proj_impl()))
        .unwrap_or_else(identity)
}

struct Enum {
    item: ItemEnum,
    impl_unpin: ImplUnpin,
    proj_ident: Ident,
}

impl Enum {
    fn parse(args: TokenStream, item: ItemEnum) -> Result<Self> {
        if item.variants.is_empty() {
            parse_failed("unsafe_project", "enums without variants")?;
        }

        item.variants
            .iter()
            .filter(|v| v.discriminant.is_some())
            .try_for_each(|_| parse_failed("unsafe_project", "enums with discriminants"))?;

        Ok(Self {
            impl_unpin: ImplUnpin::parse(args, &item.generics, "unsafe_project")?,
            proj_ident: proj_ident(&item.ident),
            item,
        })
    }

    fn proj_impl(self) -> TokenStream2 {
        let Self {
            mut item,
            mut impl_unpin,
            proj_ident,
        } = self;

        let ItemEnum {
            variants,
            ident: enum_ident,
            ..
        } = &mut item;

        let mut arm_vec = Vec::with_capacity(variants.len());
        let mut ty_vec = Vec::with_capacity(variants.len());
        variants
            .iter_mut()
            .for_each(|Variant { fields, ident, .. }| {
                let (proj_arm, proj_ty) = match fields {
                    Fields::Unnamed(fields) => {
                        unnamed(fields, ident, enum_ident, &proj_ident, &mut impl_unpin)
                    }
                    Fields::Named(fields) => {
                        named(fields, ident, enum_ident, &proj_ident, &mut impl_unpin)
                    }
                    Fields::Unit => unit(ident, enum_ident, &proj_ident),
                };

                arm_vec.push(proj_arm);
                ty_vec.push(proj_ty);
            });

        let pin = pin();
        let ident = &item.ident;
        let proj_generics = proj_generics(&item.generics);
        let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
        let proj_ty_generics = proj_generics.split_for_impl().1;

        let proj_item = quote! {
            enum #proj_ident #proj_generics #where_clause {
                #(#ty_vec,)*
            }
        };

        let proj_impl = quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                fn project<'__a>(self: #pin<&'__a mut Self>) -> #proj_ident #proj_ty_generics {
                    unsafe {
                        match #pin::get_unchecked_mut(self) {
                            #(#arm_vec,)*
                        }
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
}

fn named(
    FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    variant_ident: &Ident,
    enum_ident: &Ident,
    proj_ident: &Ident,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream2, TokenStream2) {
    let pin = pin();
    let mut pat_vec = Vec::with_capacity(fields.len());
    let mut expr_vec = Vec::with_capacity(fields.len());
    let mut ty_vec = Vec::with_capacity(fields.len());
    fields.iter_mut().for_each(
        |Field {
             attrs, ident, ty, ..
         }| {
            if find_remove(attrs, "pin").is_some() {
                impl_unpin.push(ty);
                expr_vec.push(quote!(#ident: #pin::new_unchecked(#ident)));
                ty_vec.push(quote!(#ident: #pin<&'__a mut #ty>));
            } else {
                expr_vec.push(quote!(#ident: #ident));
                ty_vec.push(quote!(#ident: &'__a mut #ty));
            }

            pat_vec.push(ident);
        },
    );

    let proj_arm = quote! {
        #enum_ident::#variant_ident { #(#pat_vec),* } => #proj_ident::#variant_ident { #(#expr_vec),* }
    };
    let proj_ty = quote!(#variant_ident { #(#ty_vec),* });

    (proj_arm, proj_ty)
}

fn unnamed(
    FieldsUnnamed {
        unnamed: fields, ..
    }: &mut FieldsUnnamed,
    variant_ident: &Ident,
    enum_ident: &Ident,
    proj_ident: &Ident,
    impl_unpin: &mut ImplUnpin,
) -> (TokenStream2, TokenStream2) {
    let pin = pin();
    let mut pat_vec = Vec::with_capacity(fields.len());
    let mut expr_vec = Vec::with_capacity(fields.len());
    let mut ty_vec = Vec::with_capacity(fields.len());
    fields
        .iter_mut()
        .enumerate()
        .for_each(|(i, Field { attrs, ty, .. })| {
            let x = Ident::new(&format!("_x{}", i), Span::call_site());

            if find_remove(attrs, "pin").is_some() {
                impl_unpin.push(ty);
                expr_vec.push(quote!(#pin::new_unchecked(#x)));
                ty_vec.push(quote!(#pin<&'__a mut #ty>));
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
) -> (TokenStream2, TokenStream2) {
    let proj_arm = quote! {
        #enum_ident::#variant_ident => #proj_ident::#variant_ident
    };
    let proj_ty = quote!(#variant_ident);

    (proj_arm, proj_ty)
}
