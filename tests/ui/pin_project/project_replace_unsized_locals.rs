#![feature(unsized_locals)]

use pin_project::pin_project;

#[pin_project(Replace)] //~ ERROR E0277
struct Struct<T: ?Sized> {
    x: T,
}

#[pin_project(Replace)] //~ ERROR E0277
struct TupleStruct<T: ?Sized>(T);

fn main() {}
