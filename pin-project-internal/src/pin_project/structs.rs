use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Nothing, Field, Fields, FieldsNamed, FieldsUnnamed, Index, ItemStruct, Result};

use crate::utils::VecExt;

use super::{Context, ProjTraitGenerics, PIN};

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

    let Context { proj_ident, proj_trait, orig_ident, lifetime, .. } = &cx;
    let proj_generics = cx.proj_generics();
    let proj_ty_generics = proj_generics.split_for_impl().1;


    /*let mut orig_generics_for_impl = item.generics.clone();
    crate::utils::proj_generics(&mut orig_generics_for_impl, lifetime.clone());
    let (impl_generics, modified_ty_generics, _) = orig_generics_for_impl.split_for_impl();

    let (_, ty_generics, where_clause) = item.generics.split_for_impl();*/

    let ProjTraitGenerics { impl_generics, ty_generics, where_clause, orig_ty_generics }
        = cx.proj_trait_generics();

    let mut proj_items = quote! {
        #[allow(dead_code)]
        struct #proj_ident #proj_generics #where_clause #proj_fields
    };

    let crate_path = crate::utils::crate_path();

    proj_items.extend(quote! {
        impl #impl_generics #proj_trait #ty_generics for ::core::pin::Pin<&#lifetime mut #orig_ident #orig_ty_generics> #where_clause {
            fn project(&mut self) -> #proj_ident #proj_ty_generics #where_clause {
                unsafe {
                    use #crate_path::ProjectThrough;
                    let this = self.proj_through().get_unchecked_mut();
                    #proj_ident #proj_init
                }
            }
        }
    });

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
