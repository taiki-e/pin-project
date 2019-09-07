use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Fields, FieldsNamed, FieldsUnnamed, Ident, ItemStruct, Result};

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

    let (proj_pat, proj_body, proj_fields) = match &mut item.fields {
        Fields::Named(fields) => super::enums::named(cx, fields)?,
        Fields::Unnamed(fields) => super::enums::unnamed(cx, fields, true)?,
        Fields::Unit => unreachable!(),
    };

    let Context { orig_ident, proj_ident, .. } = &cx;
    let proj_generics = cx.proj_generics();
    let where_clause = item.generics.split_for_impl().2;

    let mut proj_items = quote! {
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
        #[allow(dead_code)] // This lint warns unused fields/variants.
        struct #proj_ident #proj_generics #where_clause #proj_fields
    };

    let project_body = quote! {
        let #orig_ident #proj_pat = self.as_mut().get_unchecked_mut();
        #proj_ident #proj_body
    };
    let project_into_body = quote! {
        let #orig_ident #proj_pat = self.get_unchecked_mut();
        #proj_ident #proj_body
    };

    proj_items.extend(cx.make_proj_impl(&project_body, &project_into_body));

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    Ok(item)
}
