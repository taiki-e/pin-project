// compile-fail

#![deny(warnings, unsafe_code)]

use pin_project::{pin_project, project};
use std::pin::Pin;

#[pin_project]
enum Enum<A, B> {
    A(#[pin] A),
    B(B),
}

struct Struct<T>(T);

#[project]
fn foo() {
    let mut foo: Enum<bool, bool> = Enum::A(true);
    let mut foo = Pin::new(&mut foo);

    #[project]
    let Struct(x) = match foo.project() {
        Enum::A(_) => Struct(true),
        Enum::B(_) => unreachable!(),
    };
    assert!(x);
}

fn main() {}
