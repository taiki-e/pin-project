use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Result,
};

/// Makes the ident of projected type from the reference of the original ident.
pub(crate) fn proj_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("__{}Projection", ident), Span::call_site())
}

pub(crate) trait VecExt {
    fn find_remove(&mut self, ident: &str) -> Option<Attribute>;
}

impl VecExt for Vec<Attribute> {
    fn find_remove(&mut self, ident: &str) -> Option<Attribute> {
        self.iter().position(|attr| attr.path.is_ident(ident)).map(|i| self.remove(i))
    }
}

// See https://github.com/dtolnay/syn/commit/82a3aed7ecfd07fc2f7f322b01d2413ffea6c5e7
/// An empty syntax tree node that consumes no tokens when parsed.
pub(crate) struct Nothing;

impl Parse for Nothing {
    fn parse(_input: ParseStream<'_>) -> Result<Self> {
        Ok(Nothing)
    }
}

macro_rules! span {
    ($expr:expr) => {
        $expr.clone()
    };
}

macro_rules! error {
    ($msg:expr) => {
        syn::Error::new_spanned($msg, $msg)
    };
    ($span:expr, $msg:expr) => {
        syn::Error::new_spanned($span, $msg)
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}
