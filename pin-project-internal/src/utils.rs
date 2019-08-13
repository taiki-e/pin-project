use proc_macro2::Ident;
use quote::format_ident;
use syn::Attribute;

/// Makes the ident of projected type from the reference of the original ident.
pub(crate) fn proj_ident(ident: &Ident) -> Ident {
    format_ident!("__{}Projection", ident, span = ident.span())
}

pub(crate) trait VecExt {
    fn find_remove(&mut self, ident: &str) -> Option<Attribute>;
}

impl VecExt for Vec<Attribute> {
    fn find_remove(&mut self, ident: &str) -> Option<Attribute> {
        self.iter().position(|attr| attr.path.is_ident(ident)).map(|i| self.remove(i))
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
    format_ident!(
        "{}",
        if cur_crate == "pin-project" { "pin_project" } else { crate::PIN_PROJECT_CRATE.as_str() },
    )
}

/// If the 'renamed' feature is not enabled, we just
/// assume that the 'pin-project' dependency has not been renamed
#[cfg(not(feature = "renamed"))]
pub(crate) fn crate_path() -> Ident {
    format_ident!("pin_project")
}

macro_rules! error {
    ($span:expr, $msg:expr) => {
        syn::Error::new_spanned($span, $msg)
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}
