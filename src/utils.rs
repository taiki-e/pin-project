use proc_macro2::{Ident, Span};
use syn::Attribute;

/// Makes the ident of projected type from the reference of the original ident.
pub(crate) fn proj_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("__{}Projection", ident), Span::call_site())
}

pub(crate) trait VecExt {
    fn find_remove(&mut self, ident: &str) -> bool;
}

impl VecExt for Vec<Attribute> {
    fn find_remove(&mut self, ident: &str) -> bool {
        self.iter()
            .position(|Attribute { path, tokens, .. }| path.is_ident(ident) && tokens.is_empty())
            .map(|i| self.remove(i))
            .is_some()
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
