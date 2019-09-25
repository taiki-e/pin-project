// compile-fail

// NB: If you change this test, change 'trivial_bounds.rs' at the same time.

use pin_project::pin_project;
use std::marker::PhantomPinned;

struct Inner(PhantomPinned);

// As a workaround, you need to use `UnsafeUnpin`.
#[pin_project(UnsafeUnpin)] // Ok
struct Foo(#[pin] Inner);

#[pin_project] //~ ERROR E0277
struct Bar(#[pin] Inner);

fn main() {}
