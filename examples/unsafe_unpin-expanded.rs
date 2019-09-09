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
    pinned: T,
    unpinned: U,
}

#[allow(clippy::mut_mut)]
#[allow(dead_code)]
struct __FooProjection<'_pin, T, U> {
    pinned: ::core::pin::Pin<&'_pin mut T>,
    unpinned: &'_pin mut U,
}

impl<'_outer_pin, T, U> __FooProjectionTrait<'_outer_pin, T, U>
    for ::core::pin::Pin<&'_outer_pin mut Foo<T, U>>
{
    fn project<'_pin>(&'_pin mut self) -> __FooProjection<'_pin, T, U> {
        unsafe {
            let Foo { pinned, unpinned } = self.as_mut().get_unchecked_mut();
            __FooProjection { pinned: ::core::pin::Pin::new_unchecked(pinned), unpinned: unpinned }
        }
    }
    fn project_into(self) -> __FooProjection<'_outer_pin, T, U> {
        unsafe {
            let Foo { pinned, unpinned } = self.get_unchecked_mut();
            __FooProjection { pinned: ::core::pin::Pin::new_unchecked(pinned), unpinned: unpinned }
        }
    }
}

trait __FooProjectionTrait<'_outer_pin, T, U> {
    fn project<'_pin>(&'_pin mut self) -> __FooProjection<'_pin, T, U>;
    fn project_into(self) -> __FooProjection<'_outer_pin, T, U>;
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {}

impl<T, U> ::core::marker::Unpin for Foo<T, U> where
    ::pin_project::__private::Wrapper<Self>: ::pin_project::UnsafeUnpin
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
