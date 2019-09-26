// Original code (./unsafe_unpin.rs):
//
// ```rust
// #![allow(dead_code, unused_imports)]
//
// use pin_project::{pin_project, UnsafeUnpin};
//
// #[pin_project(UnsafeUnpin)]
// pub struct Foo<T, U> {
//     #[pin]
//     pinned: T,
//     unpinned: U,
// }
//
// unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {}
//
// fn main() {}
// ```

#![allow(dead_code, unused_imports)]

use pin_project::{pin_project, UnsafeUnpin};

pub struct Foo<T, U> {
    // #[pin]
    pinned: T,
    unpinned: U,
}

#[allow(clippy::mut_mut)]
#[allow(dead_code)]
pub(crate) struct __FooProjection<'_pin, T, U> {
    pinned: ::core::pin::Pin<&'_pin mut T>,
    unpinned: &'_pin mut U,
}
#[allow(dead_code)]
pub(crate) struct __FooProjectionRef<'_pin, T, U> {
    pinned: ::core::pin::Pin<&'_pin T>,
    unpinned: &'_pin U,
}

impl<T, U> Foo<T, U> {
    pub(crate) fn project<'_pin>(
        self: ::core::pin::Pin<&'_pin mut Self>,
    ) -> __FooProjection<'_pin, T, U> {
        unsafe {
            let Foo { pinned, unpinned } = self.get_unchecked_mut();
            __FooProjection { pinned: ::core::pin::Pin::new_unchecked(pinned), unpinned: unpinned }
        }
    }
    pub(crate) fn project_ref<'_pin>(
        self: ::core::pin::Pin<&'_pin Self>,
    ) -> __FooProjectionRef<'_pin, T, U> {
        unsafe {
            let Foo { pinned, unpinned } = self.get_ref();
            __FooProjectionRef {
                pinned: ::core::pin::Pin::new_unchecked(pinned),
                unpinned: unpinned,
            }
        }
    }
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {}

#[allow(single_use_lifetimes)]
impl<'_pin, T, U> ::core::marker::Unpin for Foo<T, U> where
    ::pin_project::__private::Wrapper<'_pin, Self>: ::pin_project::UnsafeUnpin
{
}

// Ensure that struct does not implement `Drop`.
//
// See ./struct-default-expanded.rs for details.
trait FooMustNotImplDrop {}
#[allow(clippy::drop_bounds)]
impl<T: ::core::ops::Drop> FooMustNotImplDrop for T {}
#[allow(single_use_lifetimes)]
impl<T, U> FooMustNotImplDrop for Foo<T, U> {}

// Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
//
// See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/34
// for details.
#[allow(single_use_lifetimes)]
#[allow(non_snake_case)]
#[deny(safe_packed_borrows)]
fn __pin_project_assert_not_repr_packed_Foo<T, U>(val: Foo<T, U>) {
    {
        &val.pinned;
    }
    {
        &val.unpinned;
    }
}

fn main() {}
