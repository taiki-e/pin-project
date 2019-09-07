// run-pass

// Refs: https://github.com/rust-lang/rust/issues/48214

#![feature(trivial_bounds)]

use core::marker::PhantomPinned;
use pin_project::pin_project;

struct Inner(PhantomPinned);

#[pin_project]
struct Foo(#[pin] Inner);

fn main() {}
