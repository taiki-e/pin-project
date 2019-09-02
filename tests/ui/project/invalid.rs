// compile-fail

use pin_project::{pin_project, project};

#[pin_project]
struct A<T> {
    #[pin]
    future: T,
}

#[project]
fn foo() {
    let x = A { future: 0 };
    #[project(foo)] //~ ERROR unexpected token
    let A { future } = x.project();
}

fn main() {}
