// Original code (./not_unpin.rs):
//
// ```rust
// #![allow(dead_code)]
//
// use pin_project::pin_project;
//
// #[pin_project(!Unpin)]
// pub struct Struct<T, U> {
//     #[pin]
//     pinned: T,
//     unpinned: U,
// }
//
// fn main() {
//     fn _is_unpin<T: Unpin>() {}
//     // _is_unpin::<Struct<(), ()>>(); //~ ERROR `std::marker::PhantomPinned` cannot be unpinned
// }
// ```

#![allow(dead_code, unused_imports, unused_parens)]
#![allow(clippy::no_effect)]

use pin_project::pin_project;

pub struct Struct<T, U> {
    // #[pin]
    pinned: T,
    unpinned: U,
}

#[doc(hidden)]
#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
#[allow(single_use_lifetimes)]
pub(crate) struct __StructProjection<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__reexport::pin::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[doc(hidden)]
#[allow(dead_code)] // This lint warns unused fields/variants.
#[allow(single_use_lifetimes)]
pub(crate) struct __StructProjectionRef<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__reexport::pin::Pin<&'pin (T)>,
    unpinned: &'pin (U),
}

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
const __SCOPE_Struct: () = {
    impl<T, U> Struct<T, U> {
        pub(crate) fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                __StructProjection {
                    pinned: ::pin_project::__reexport::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        pub(crate) fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_ref();
                __StructProjectionRef {
                    pinned: ::pin_project::__reexport::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
    }

    // Create `Unpin` impl that has trivial `Unpin` bounds.
    //
    // See https://github.com/taiki-e/pin-project/issues/102#issuecomment-540472282
    // for details.
    impl<'pin, T, U> ::pin_project::__reexport::marker::Unpin for Struct<T, U> where
        ::pin_project::__private::Wrapper<'pin, ::pin_project::__reexport::marker::PhantomPinned>:
            ::pin_project::__reexport::marker::Unpin
    {
    }
    // A dummy impl of `UnsafeUnpin`, to ensure that the user cannot implement it.
    //
    // To ensure that users don't accidentally write a non-functional `UnsafeUnpin`
    // impls, we emit one ourselves. If the user ends up writing a `UnsafeUnpin` impl,
    // they'll get a "conflicting implementations of trait" error when coherence
    // checks are run.
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for Struct<T, U> {}

    // Ensure that struct does not implement `Drop`.
    //
    // See ./struct-default-expanded.rs for details.
    trait StructMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::pin_project::__reexport::ops::Drop> StructMustNotImplDrop for T {}
    impl<T, U> StructMustNotImplDrop for Struct<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for Struct<T, U> {
        unsafe fn drop(self: ::pin_project::__reexport::pin::Pin<&mut Self>) {}
    }

    // Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
    //
    // See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/34
    // for details.
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &Struct<T, U>) {
        &val.pinned;
        &val.unpinned;
    }
};

fn main() {
    fn _is_unpin<T: Unpin>() {}
    // _is_unpin::<Struct<(), ()>>(); //~ ERROR `std::marker::PhantomPinned` cannot be unpinned
}
