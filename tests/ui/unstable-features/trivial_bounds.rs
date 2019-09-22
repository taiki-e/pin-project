// run-pass

// NB: If you change this test, change 'trivial_bounds-feature-gate.rs' at the same time.

// trivial_bounds
// Tracking issue: https://github.com/rust-lang/rust/issues/48214
#![feature(trivial_bounds)]

use pin_project::pin_project;
use std::marker::PhantomPinned;

struct Inner(PhantomPinned);

#[pin_project]
struct Foo(#[pin] Inner);

fn main() {}
