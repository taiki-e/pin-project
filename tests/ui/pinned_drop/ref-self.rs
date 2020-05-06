// `ref (mut) self` are rejected by rustc.

use std::pin::Pin;

pub struct Struct {
    field: u8,
}

impl Struct {
    fn take_ref_self(ref self: Pin<&mut Self>) {} //~ ERROR expected identifier, found keyword `self`
    fn take_ref_mut_self(ref mut self: Pin<&mut Self>) {} //~ ERROR expected identifier, found keyword `self`
}

fn main() {}
