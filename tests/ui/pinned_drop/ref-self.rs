use std::pin::Pin;

pub struct Foo {
    field: u8,
}

impl Foo {
    fn method_ref(ref self: Pin<&mut Self>) {} //~ ERROR expected identifier, found keyword `self`
    fn method_ref_mut(ref mut self: Pin<&mut Self>) {} //~ ERROR expected identifier, found keyword `self`
}

fn main() {}
