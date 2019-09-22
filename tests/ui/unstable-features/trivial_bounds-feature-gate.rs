// compile-fail

// NB: If you change this test, change 'trivial_bounds.rs' at the same time.

use pin_project::pin_project;
use std::marker::PhantomPinned;

struct Inner(PhantomPinned);

#[pin_project] //~ ERROR E0277
struct Foo(#[pin] Inner);

fn main() {}
