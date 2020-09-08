#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use pin_project::{pin_project, UnsafeUnpin};
use std::{marker::PhantomPinned, pin::Pin};

fn is_unpin<T: Unpin>() {}

#[pin_project(UnsafeUnpin)]
pub struct Blah<T, U> {
    f1: U,
    #[pin]
    f2: T,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Blah<T, U> {}

#[pin_project(UnsafeUnpin)]
pub struct OverlappingLifetimeNames<'pin, T, U> {
    #[pin]
    f1: T,
    f2: U,
    f3: &'pin (),
}

unsafe impl<T: Unpin, U> UnsafeUnpin for OverlappingLifetimeNames<'_, T, U> {}

#[test]
fn unsafe_unpin() {
    is_unpin::<Blah<(), PhantomPinned>>();
    is_unpin::<OverlappingLifetimeNames<'_, (), ()>>();
}

#[test]
fn trivial_bounds() {
    #[pin_project(UnsafeUnpin)]
    pub struct NotImplementUnsafUnpin {
        #[pin]
        f: PhantomPinned,
    }
}

#[test]
fn test() {
    let mut x = OverlappingLifetimeNames { f1: 0, f2: 1, f3: &() };
    let x = Pin::new(&mut x);
    let y = x.as_ref().project_ref();
    let _: Pin<&u8> = y.f1;
    let _: &u8 = y.f2;
    let y = x.project();
    let _: Pin<&mut u8> = y.f1;
    let _: &mut u8 = y.f2;
}
