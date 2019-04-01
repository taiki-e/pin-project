mod enums;
mod structs;

use std::convert::identity;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, Generics, Item, Type};

use crate::utils::{failed, Result};

/// The attribute name.
const NAME: &str = "unsafe_project";
/// The annotation for pinned type.
const PIN: &str = "pin";

pub(super) fn attribute(args: &str, input: TokenStream) -> TokenStream {
    match syn::parse2(input) {
        Ok(Item::Struct(item)) => structs::parse(args, item),
        Ok(Item::Enum(item)) => enums::parse(args, item),
        _ => failed(NAME, "may only be used on structs or enums"),
    }
    .unwrap_or_else(identity)
}

#[inline(never)]
fn parse_failed<T>(msg: &str) -> Result<T> {
    failed(NAME, &format!("cannot be implemented for {}", msg))
}

/// Makes the generics of projected type from the reference of the original generics.
fn proj_generics(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!('__a));
    generics
}

// =================================================================================================
// conditional Unpin implementation

struct ImplUnpin(Option<Generics>);

impl ImplUnpin {
    /// Parses attribute arguments.
    fn parse(args: &str, generics: &Generics) -> Result<Self> {
        match args {
            "" => Ok(Self(None)),
            "Unpin" => Ok(Self(Some(generics.clone()))),
            _ => failed(NAME, "an invalid argument was passed"),
        }
    }

    fn push(&mut self, ty: &Type) {
        if let Some(generics) = &mut self.0 {
            generics.make_where_clause().predicates.push(parse_quote!(#ty: ::core::marker::Unpin));
        }
    }

    /// Creates `Unpin` implementation.
    fn build(self, ident: &Ident) -> TokenStream {
        self.0
            .map(|generics| {
                let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
                quote! {
                    impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
                }
            })
            .unwrap_or_default()
    }
}
