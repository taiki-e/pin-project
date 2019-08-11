#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project-internal/0.3.3")]
#![doc(test(attr(deny(warnings), allow(dead_code, unused_assignments, unused_variables))))]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]
#![warn(single_use_lifetimes)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::use_self)]

extern crate proc_macro;

#[macro_use]
mod utils;

mod pin_project;
mod pinned_drop;
#[cfg(feature = "project_attr")]
mod project;

use proc_macro::TokenStream;
use utils::Nothing;

#[proc_macro_attribute]
pub fn pin_project(args: TokenStream, input: TokenStream) -> TokenStream {
    pin_project::attribute(args.into(), input.into()).into()
}

#[proc_macro_attribute]
pub fn pinned_drop(args: TokenStream, input: TokenStream) -> TokenStream {
    let _: Nothing = syn::parse_macro_input!(args);
    pinned_drop::attribute(input.into()).into()
}

#[cfg(feature = "project_attr")]
#[proc_macro_attribute]
pub fn project(args: TokenStream, input: TokenStream) -> TokenStream {
    let _: Nothing = syn::parse_macro_input!(args);
    project::attribute(input.into()).into()
}

#[cfg(feature = "renamed")]
lazy_static::lazy_static! {
    pub(crate) static ref PIN_PROJECT_CRATE: String = {
        proc_macro_crate::crate_name("pin-project")
            .expect("pin-project-internal was used without pin-project!")
    };
}
