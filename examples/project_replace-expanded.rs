// Original code (./struct-default.rs):
//
// ```rust
// #![allow(dead_code)]
//
// use pin_project::pin_project;
//
// #[pin_project(Replace)]
// struct Struct<T, U> {
//     #[pin]
//     pinned: T,
//     unpinned: U,
// }
//
// fn main() {}
// ```

#![allow(dead_code, unused_imports, unused_parens)]
#![allow(clippy::no_effect)]

use pin_project::pin_project;

struct Struct<T, U> {
    // #[pin]
    pinned: T,
    unpinned: U,
}

#[doc(hidden)]
#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
#[allow(single_use_lifetimes)]
struct __StructProjection<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__reexport::pin::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[doc(hidden)]
#[allow(dead_code)] // This lint warns unused fields/variants.
#[allow(single_use_lifetimes)]
struct __StructProjectionRef<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__reexport::pin::Pin<&'pin (T)>,
    unpinned: &'pin (U),
}

#[doc(hidden)]
#[allow(dead_code)] // This lint warns unused fields/variants.
#[allow(single_use_lifetimes)]
struct __StructProjectionOwned<T, U> {
    pinned: ::pin_project::__reexport::marker::PhantomData<T>,
    unpinned: U,
}

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
const __SCOPE_Struct: () = {
    impl<T, U> Struct<T, U> {
        fn project<'pin>(
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
        fn project_ref<'pin>(
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
        fn project_replace(
            self: ::pin_project::__reexport::pin::Pin<&mut Self>,
            __replacement: Self,
        ) -> __StructProjectionOwned<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let Self { pinned, unpinned } = &mut *__self_ptr;

                // First, extract all the unpinned fields
                let __result = __StructProjectionOwned {
                    pinned: ::pin_project::__reexport::marker::PhantomData,
                    unpinned: ::pin_project::__reexport::ptr::read(unpinned),
                };

                // Destructors will run in reverse order, so next create a guard to overwrite
                // `self` with the replacement value without calling destructors.
                let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                    target: __self_ptr,
                    value: ::pin_project::__reexport::mem::ManuallyDrop::new(__replacement),
                };

                // Now create guards to drop all the pinned fields
                //
                // Due to a compiler bug (https://github.com/rust-lang/rust/issues/47949)
                // this must be in its own scope, or else `__result` will not be dropped
                // if any of the destructors panic.
                {
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(pinned);
                }

                // Finally, return the result
                __result
            }
        }
    }

    // Automatically create the appropriate conditional `Unpin` implementation.
    //
    // See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/53.
    // for details.
    struct __Struct<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__reexport::marker::Unpin for Struct<T, U> where
        __Struct<'pin, T, U>: ::pin_project::__reexport::marker::Unpin
    {
    }
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

fn main() {}
