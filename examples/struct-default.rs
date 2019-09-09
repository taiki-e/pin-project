// See ./struct-default-expanded.rs for generated code.

#![allow(dead_code, unused_imports)]

use pin_project::pin_project;

#[pin_project]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

fn main() {}
