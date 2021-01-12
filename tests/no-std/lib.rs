#![no_std]
#![warn(rust_2018_idioms, single_use_lifetimes)]

// This works in 2018 edition, but in 2015 edition it gives an error:
// ```text
// error[E0659]: `pin` is ambiguous (derive helper attribute vs any other name)
//  --> tests/rust-2015/../include/basic-safe-part.rs:3:1
//   |
// 3 | #[pin_project]
//   | ^^^^^^^^^^^^^^ ambiguous name
//   |
// ```
#[allow(unused_imports)]
use pin_project as pin;

include!("../include/basic.rs");
