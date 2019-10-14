use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::{
    parse::{ParseBuffer, ParseStream},
    punctuated::Punctuated,
    token::{self, Comma},
    *,
};

pub(crate) const DEFAULT_LIFETIME_NAME: &str = "'pin";
pub(crate) const CURRENT_PRIVATE_MODULE: &str = "__private";

pub(crate) type Variants = Punctuated<Variant, token::Comma>;

pub(crate) use Mutability::{Immutable, Mutable};

macro_rules! error {
    ($span:expr, $msg:expr) => {
        syn::Error::new_spanned(&$span, $msg)
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}

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
pub(crate) fn determine_lifetime_name(
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

/// Inserts a `lifetime` at position `0` of `generics.params`.
pub(crate) fn insert_lifetime(generics: &mut Generics, lifetime: Lifetime) {
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

/// Check if `tokens` is an empty `TokenStream`.
/// This is almost equivalent to `syn::parse2::<Nothing>()`,
/// but produces a better error message and does not require ownership of `tokens`.
pub(crate) fn parse_as_empty(tokens: &TokenStream) -> Result<()> {
    if tokens.is_empty() { Ok(()) } else { Err(error!(tokens, "unexpected token: {}", tokens)) }
}

pub(crate) trait SliceExt {
    fn position_exact(&self, ident: &str) -> Result<Option<usize>>;
    fn find(&self, ident: &str) -> Option<&Attribute>;
    fn find_exact(&self, ident: &str) -> Result<Option<&Attribute>>;
}

pub(crate) trait VecExt {
    fn find_remove(&mut self, ident: &str) -> Result<Option<Attribute>>;
}

impl SliceExt for [Attribute] {
    fn position_exact(&self, ident: &str) -> Result<Option<usize>> {
        self.iter()
            .try_fold((0, None), |(i, mut prev), attr| {
                if attr.path.is_ident(ident) {
                    if prev.is_some() {
                        return Err(error!(attr, "duplicate #[{}] attribute", ident));
                    }
                    parse_as_empty(&attr.tokens)?;
                    prev = Some(i);
                }
                Ok((i + 1, prev))
            })
            .map(|(_, pos)| pos)
    }
    fn find(&self, ident: &str) -> Option<&Attribute> {
        self.iter().position(|attr| attr.path.is_ident(ident)).and_then(|i| self.get(i))
    }
    fn find_exact(&self, ident: &str) -> Result<Option<&Attribute>> {
        self.position_exact(ident).map(|pos| pos.and_then(|i| self.get(i)))
    }
}

impl VecExt for Vec<Attribute> {
    fn find_remove(&mut self, ident: &str) -> Result<Option<Attribute>> {
        self.position_exact(ident).map(|pos| pos.map(|i| self.remove(i)))
    }
}

pub(crate) trait ParseBufferExt<'a> {
    fn parenthesized(self) -> Result<ParseBuffer<'a>>;
}

impl<'a> ParseBufferExt<'a> for ParseStream<'a> {
    fn parenthesized(self) -> Result<ParseBuffer<'a>> {
        let content;
        let _: token::Paren = syn::parenthesized!(content in self);
        Ok(content)
    }
}

impl<'a> ParseBufferExt<'a> for ParseBuffer<'a> {
    fn parenthesized(self) -> Result<ParseBuffer<'a>> {
        let content;
        let _: token::Paren = syn::parenthesized!(content in self);
        Ok(content)
    }
}
