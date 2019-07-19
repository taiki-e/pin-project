#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project/0.3.3")]
#![doc(test(attr(deny(warnings), allow(dead_code, unused_assignments, unused_variables))))]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]
#![warn(single_use_lifetimes)]
#![warn(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![feature(proc_macro_hygiene)]

extern crate proc_macro;

#[macro_use]
mod utils;

#[cfg(feature = "project_attr")]
mod project;
mod pin_projectable;

use proc_macro::TokenStream;
use lazy_static::lazy_static;


#[cfg(feature = "project_attr")]
#[proc_macro_attribute]
pub fn project(args: TokenStream, input: TokenStream) -> TokenStream {
    assert!(args.is_empty());
    TokenStream::from(project::attribute(input.into()))
}


/// This is a doc comment from the defining crate!
#[proc_macro]
pub fn pin_project(input: TokenStream) -> TokenStream {
	TokenStream::from(pin_projectable::pin_project(input.into()).unwrap_or_else(|e| e.to_compile_error()))
}

#[proc_macro_attribute]
pub fn pin_projectable(args: TokenStream, input: TokenStream) -> TokenStream {
	TokenStream::from(pin_projectable::attribute(args.into(), input.into()).unwrap_or_else(|e| e.to_compile_error()))
}

lazy_static! {
    pub(crate) static ref PIN_PROJECT_CRATE: String = {
        let crate_name = proc_macro_crate::crate_name("pin-projectable")
            .expect("pin-project-internal was used without pin-project!");
        crate_name
    };
}
