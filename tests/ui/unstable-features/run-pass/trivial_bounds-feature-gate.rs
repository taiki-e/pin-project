// run-pass

// NB: If you change this test, change 'trivial_bounds.rs' at the same time.

use pin_project::pin_project;
use std::marker::PhantomPinned;

struct Inner(PhantomPinned);

#[pin_project]
struct Foo(#[pin] Inner);

#[pin_project(UnsafeUnpin)]
struct Bar(#[pin] Inner);

fn main() {}
