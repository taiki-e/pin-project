use quote::{format_ident, quote_spanned};
use syn::{
    punctuated::Punctuated,
    token::{self, Comma},
    *,
};

pub(crate) const DEFAULT_LIFETIME_NAME: &str = "'_pin";

pub(crate) type Variants = Punctuated<Variant, token::Comma>;

pub(crate) use Mutability::{Immutable, Mutable};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Mutability {
    Mutable,
    Immutable,
}

/// Creates the ident of projected type from the ident of the original type.
pub(crate) fn proj_ident(ident: &Ident, mutability: Mutability) -> Ident {
    if mutability == Mutable {
        format_ident!("__{}Projection", ident)
    } else {
        format_ident!("__{}ProjectionRef", ident)
    }
}

/// Determines the lifetime names. Ensure it doesn't overlap with any existing lifetime names.
pub(crate) fn proj_lifetime_name(
    lifetime_name: &mut String,
    generics: &Punctuated<GenericParam, Comma>,
) {
    let existing_lifetimes: Vec<String> = generics
        .iter()
        .filter_map(|param| {
            if let GenericParam::Lifetime(LifetimeDef { lifetime, .. }) = param {
                Some(lifetime.to_string())
            } else {
                None
            }
        })
        .collect();
    while existing_lifetimes.iter().any(|name| name.starts_with(&**lifetime_name)) {
        lifetime_name.push('_');
    }
}

/// Creates the generics of projected type from the generics of the original type.
pub(crate) fn proj_generics(generics: &mut Generics, lifetime: Lifetime) {
    if generics.lt_token.is_none() {
        generics.lt_token = Some(token::Lt::default())
    }
    if generics.gt_token.is_none() {
        generics.gt_token = Some(token::Gt::default())
    }

    generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeDef {
            attrs: Vec::new(),
            lifetime,
            colon_token: None,
            bounds: Punctuated::new(),
        }),
    );
}

/// Determines the visibility of the projected type and projection method.
pub(crate) fn determine_visibility(vis: &Visibility) -> Visibility {
    if let Visibility::Public(token) = vis {
        syn::parse2(quote_spanned! { token.pub_token.span =>
            pub(crate)
        })
        .unwrap()
    } else {
        vis.clone()
    }
}

pub(crate) fn collect_cfg(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs.iter().filter(|attr| attr.path.is_ident("cfg")).cloned().collect()
}

pub(crate) trait VecExt {
    fn position(&self, ident: &str) -> Option<usize>;
    fn find_remove(&mut self, ident: &str) -> Option<Attribute>;
}

impl VecExt for Vec<Attribute> {
    fn position(&self, ident: &str) -> Option<usize> {
        self.iter().position(|attr| attr.path.is_ident(ident))
    }
    fn find_remove(&mut self, ident: &str) -> Option<Attribute> {
        self.position(ident).map(|i| self.remove(i))
    }
}

macro_rules! error {
    ($span:expr, $msg:expr) => {
        syn::Error::new_spanned(&$span, $msg)
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}
