// Original code (./pinned_drop.rs):
//
// ```rust
// #![allow(dead_code)]
//
// use pin_project::{pin_project, pinned_drop};
// use std::pin::Pin;
//
// #[pin_project(PinnedDrop)]
// pub struct Struct<'a, T> {
//     was_dropped: &'a mut bool,
//     #[pin]
//     field: T,
// }
//
// #[pinned_drop]
// fn drop_Struct<T>(mut this: Pin<&mut Struct<'_, T>>) {
//     **this.project().was_dropped = true;
// }
//
// fn main() {}
// ```

#![allow(dead_code, unused_imports, unused_parens)]

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

pub struct Struct<'a, T> {
    was_dropped: &'a mut bool,
    // #[pin]
    field: T,
}

#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
pub(crate) struct __StructProjection<'pin, 'a, T>
where
    Struct<'a, T>: 'pin,
{
    was_dropped: &'pin mut (&'a mut bool),
    field: ::pin_project::__reexport::pin::Pin<&'pin mut (T)>,
}
#[allow(dead_code)] // This lint warns unused fields/variants.
pub(crate) struct __StructProjectionRef<'pin, 'a, T>
where
    Struct<'a, T>: 'pin,
{
    was_dropped: &'pin (&'a mut bool),
    field: ::pin_project::__reexport::pin::Pin<&'pin (T)>,
}

#[allow(non_upper_case_globals)]
const __SCOPE_Struct: () = {
    impl<'a, T> Struct<'a, T> {
        pub(crate) fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, 'a, T> {
            unsafe {
                let Self { was_dropped, field } = self.get_unchecked_mut();
                __StructProjection {
                    was_dropped,
                    field: ::pin_project::__reexport::pin::Pin::new_unchecked(field),
                }
            }
        }
        pub(crate) fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, 'a, T> {
            unsafe {
                let Self { was_dropped, field } = self.get_ref();
                __StructProjectionRef {
                    was_dropped,
                    field: ::pin_project::__reexport::pin::Pin::new_unchecked(field),
                }
            }
        }
    }

    #[allow(single_use_lifetimes)]
    impl<'a, T> ::pin_project::__reexport::ops::Drop for Struct<'a, T> {
        fn drop(&mut self) {
            // Safety - we're in 'drop', so we know that 'self' will
            // never move again.
            let pinned_self = unsafe { ::pin_project::__reexport::pin::Pin::new_unchecked(self) };
            // We call `pinned_drop` only once. Since `PinnedDrop::drop`
            // is an unsafe method and a private API, it is never called again in safe
            // code *unless the user uses a maliciously crafted macro*.
            unsafe {
                ::pin_project::__private::PinnedDrop::drop(pinned_self);
            }
        }
    }

    // It is safe to implement PinnedDrop::drop, but it is not safe to call it.
    // This is because destructors can be called multiple times (double dropping
    // is unsound: rust-lang/rust#62360).
    //
    // Ideally, it would be desirable to be able to prohibit manual calls in the
    // same way as Drop::drop, but the library cannot. So, by using macros and
    // replacing them with private traits, we prevent users from calling
    // PinnedDrop::drop.
    //
    // Users can implement `Drop` safely using `#[pinned_drop]`.
    // **Do not call or implement this trait directly.**
    impl<T> ::pin_project::__private::PinnedDrop for Struct<'_, T> {
        // Since calling it twice on the same object would be UB,
        // this method is unsafe.
        unsafe fn drop(self: Pin<&mut Self>) {
            fn __drop_inner<T>(__self: Pin<&mut Struct<'_, T>>) {
                **__self.project().was_dropped = true;
            }
            __drop_inner(self);
        }
    }

    // Automatically create the appropriate conditional `Unpin` implementation.
    //
    // See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/53.
    // for details.
    pub struct __Struct<'pin, 'a, T> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T)>,
        __field0: T,
        __lifetime0: &'a (),
    }
    impl<'pin, 'a, T> ::pin_project::__reexport::marker::Unpin for Struct<'a, T> where
        __Struct<'pin, 'a, T>: ::pin_project::__reexport::marker::Unpin
    {
    }

    // Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
    //
    // See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/34
    // for details.
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<'a, T>(val: &Struct<'a, T>) {
        &val.was_dropped;
        &val.field;
    }
};

fn main() {}
