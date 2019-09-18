// compile-fail

use pin_project::{pin_project, project};
use std::pin::Pin;

#[pin_project]
struct A<T> {
    #[pin]
    future: T,
}

#[project]
fn foo() {
    let mut x = A { future: 0 };
    #[project(foo)] //~ ERROR unexpected token
    let A { future } = Pin::new(&mut x).project();
}

fn main() {}
