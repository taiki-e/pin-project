#![warn(rust_2018_idioms, single_use_lifetimes)]

// See https://github.com/rust-lang/pin-utils/pull/26#discussion_r344491597
//
// Note: If the proc-macro does not depend on its own items, it may be preferable not to
//       support overwriting the name of core/std crate for compatibility with reexport.
#[allow(unused_extern_crates)]
extern crate pin_project as core;

// Dummy module to check that the expansion refers to the crate.
mod pin_project {}

// This works in 2018 edition, but in 2015 edition it gives an error: `pin` is ambiguous (derive helper attribute vs any other name)
#[allow(unused_imports)]
use ::pin_project as pin;

use ::pin_project::{pin_project, pinned_drop, UnsafeUnpin};
use std::pin::Pin;

include!("include/basic.rs");
