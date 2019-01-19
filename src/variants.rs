use std::mem;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse_quote, Field, Fields, FieldsUnnamed, Generics, ItemEnum, Variant};

use crate::utils::*;

pub(super) fn unsafe_variants(args: TokenStream, input: TokenStream) -> TokenStream {
    Enum::parse(args, input)
        .map(|parsed| TokenStream::from(parsed.proj_impl()))
        .unwrap_or_else(|e| e)
}

struct Enum {
    item: ItemEnum,
    impl_unpin: Option<Generics>,
}

impl Enum {
    fn parse(args: TokenStream, input: TokenStream) -> Result<Self> {
        let item: ItemEnum = match syn::parse(input) {
            Err(_) => Err(compile_err("`unsafe_variants` may only be used on structs"))?,
            Ok(item) => item,
        };

        let impl_unpin = parse_args(args, &item.generics, "unsafe_variants")?;
        item.variants.iter().try_for_each(|v| match &v.fields {
            _ if v.discriminant.is_some() => parse_failed("discriminants"),
            Fields::Named(_) => parse_failed("named fields"),
            _ => Ok(()),
        })?;

        if item.variants.is_empty() {
            Err(compile_err(
                "`unsafe_variants` cannot be implemented for zero-variant enums",
            ))?;
        }

        Ok(Self { item, impl_unpin })
    }

    fn proj_impl(mut self) -> TokenStream2 {
        let proj_methods = self.proj_methods();
        let unpin = unpin();
        let ident = &self.item.ident;
        let (impl_generics, ty_generics, where_clause) = self.item.generics.split_for_impl();
        let proj_impl = quote! {
            impl #impl_generics #ident #ty_generics #where_clause {
                #(#proj_methods)*
            }
        };

        let impl_unpin = self
            .impl_unpin
            .as_ref()
            .map(|generics| {
                let where_clause = generics.split_for_impl().2;
                quote! {
                    impl #impl_generics #unpin for #ident #ty_generics #where_clause {}
                }
            })
            .unwrap_or_default();

        let mut item = self.item.into_token_stream();
        item.extend(proj_impl);
        item.extend(impl_unpin);
        item
    }

    fn proj_methods(&mut self) -> Vec<TokenStream2> {
        let ItemEnum {
            variants, ident, ..
        } = &mut self.item;

        let mut proj_methods = Vec::with_capacity(variants.len());
        let mut impl_unpin = self.impl_unpin.take();
        variants.iter_mut().for_each(|variant| {
            if find_remove(&mut variant.attrs, "skip").is_some() {
                return;
            }

            let method = match &variant.fields {
                Fields::Unnamed(_) => unnamed(variant, ident, impl_unpin.as_mut()),
                Fields::Unit => return,
                _ => unreachable!(),
            };

            if let Some(method) = method {
                proj_methods.push(method);
            }
        });

        mem::replace(&mut self.impl_unpin, impl_unpin);
        proj_methods
    }
}

fn unnamed(
    variant: &mut Variant,
    ident: &Ident,
    mut impl_unpin: Option<&mut Generics>,
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

    let option = option();
    let pin = pin();
    let (pat, expr, ty) = {
        let unpin = unpin();
        let mut pat_vec = Vec::new();
        let mut expr_vec = Vec::new();
        let mut ty_vec = Vec::new();

        fields
            .iter_mut()
            .enumerate()
            .for_each(|(i, Field { attrs, ty, .. })| {
                if find_remove(attrs, "skip").is_none() {
                    let x = Ident::new(&format!("_x{}", i), Span::call_site());

                    if find_remove(attrs, "pin").is_some() {
                        if let Some(generics) = &mut impl_unpin {
                            generics
                                .make_where_clause()
                                .predicates
                                .push(parse_quote!(#ty: #unpin));
                        }

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

#[inline(never)]
fn parse_failed(msg: &str) -> Result<()> {
    Err(compile_err(&format!(
        "`unsafe_variants` cannot be implemented for enums with {}",
        msg
    )))
}
