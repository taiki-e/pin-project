use std::convert::identity;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Field, Fields, FieldsUnnamed, ItemEnum, Variant};

use crate::utils::*;

pub(super) fn unsafe_variants(args: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse(input)
        .map_err(|_| compile_err("`unsafe_variants` may only be used on structs"))
        .and_then(|item| Enum::parse(args, item))
        .map(|parsed| TokenStream::from(parsed.proj_impl()))
        .unwrap_or_else(identity)
}

struct Enum {
    item: ItemEnum,
    impl_unpin: ImplUnpin,
}

impl Enum {
    fn parse(args: TokenStream, item: ItemEnum) -> Result<Self> {
        if item.variants.is_empty() {
            parse_failed("unsafe_variants", "enums without variants")?;
        }

        item.variants.iter().try_for_each(|v| match &v.fields {
            _ if v.discriminant.is_some() => {
                parse_failed("unsafe_variants", "enums with discriminants")
            }
            Fields::Named(_) => parse_failed("unsafe_variants", "enums with named fields"),
            _ => Ok(()),
        })?;

        Ok(Self {
            impl_unpin: ImplUnpin::parse(args, &item.generics, "unsafe_variants")?,
            item,
        })
    }

    fn proj_impl(self) -> TokenStream2 {
        let Self {
            mut item,
            mut impl_unpin,
        } = self;

        let ItemEnum {
            variants,
            ident: enum_ident,
            ..
        } = item;

        let mut proj_methods = Vec::with_capacity(variants.len());
        variants.iter_mut().for_each(|variant| {
            if find_remove(&mut variant.attrs, "skip").is_some() {
                return;
            }

            let method = match &variant.fields {
                Fields::Unnamed(_) => unnamed(variant, enum_ident, impl_unpin),
                Fields::Unit => return,
                _ => unreachable!(),
            };

            if let Some(method) = method {
                proj_methods.push(method);
            }
        });

        let ident = &item.ident;
        let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
        let proj_impl = quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#proj_methods)*
            }
        };

        let impl_unpin = impl_unpin.build(impl_generics, ident, ty_generics);
        let mut item = item.into_token_stream();
        item.extend(proj_impl);
        item.extend(impl_unpin);
        item
    }
}

fn unnamed(
    variant: &mut Variant,
    ident: &Ident,
    impl_unpin: &mut ImplUnpin,
) -> Option<TokenStream2> {
    let Variant {
        fields,
        ident: variant,
        ..
    } = variant;

    let fields = match fields {
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
        _ => unreachable!(),
    };

    let pin = pin();
    let (pat, expr, ty) = {
        let mut pat_vec = Vec::with_capacity(fields.len());
        let mut expr_vec = Vec::with_capacity(fields.len());
        let mut ty_vec = Vec::with_capacity(fields.len());

        fields
            .iter_mut()
            .enumerate()
            .for_each(|(i, Field { attrs, ty, .. })| {
                if find_remove(attrs, "skip").is_none() {
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
                } else {
                    pat_vec.push(Ident::new("_", Span::call_site()));
                }
            });

        match expr_vec.len() {
            0 => None?,
            1 => (
                quote!(#(#pat_vec),*),
                expr_vec.into_iter().next().into_token_stream(),
                ty_vec.into_iter().next().into_token_stream(),
            ),
            _ => (
                quote!(#(#pat_vec),*),
                quote!((#(#expr_vec),*)),
                quote!((#(#ty_vec),*)),
            ),
        }
    };

    let option = option();
    let method = Ident::new(&variant.to_string().to_lowercase(), Span::call_site());
    Some(quote! {
        fn #method<'__a>(self: #pin<&'__a mut Self>) -> #option<#ty> {
            unsafe {
                match #pin::get_unchecked_mut(self) {
                    #ident::#variant(#pat) => #option::Some(#expr),
                    _ => #option::None,
                }
            }
        }
    })
}

fn option() -> TokenStream2 {
    quote!(::core::option::Option)
}
