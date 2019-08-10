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

/// If the 'renamed' feature is enabled, we detect
/// the actual name of the 'pin-project' crate in the consumer's Cargo.toml
#[cfg(feature = "renamed")]
pub(crate) fn crate_path() -> Ident {
    // This is fairly subtle.
    // Normally, you would use `env!("CARGO_PKG_NAME")` to get the name of the package,
    // since it's set at compile time.
    // However, we're in a proc macro, which runs while *another* crate is being compiled.
    // By retreiving the runtime value of `CARGO_PKG_NAME`, we can figure out the name
    // of the crate that's calling us.
    let cur_crate = std::env::var("CARGO_PKG_NAME")
        .expect("Could not find CARGO_PKG_NAME environemnt variable");
    Ident::new(
        if cur_crate == "pin-project" { "pin_project" } else { crate::PIN_PROJECT_CRATE.as_str() },
        Span::call_site(),
    )
}

/// If the 'renamed' feature is not enabled, we just
/// assume that the 'pin-project' dependency has not been renamed
#[cfg(not(feature = "renamed"))]
pub(crate) fn crate_path() -> Ident {
    Ident::new("pin_project", Span::call_site())
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
