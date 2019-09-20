// Original code (./pinned_drop.rs):
//
// ```rust
// #![allow(dead_code, unused_imports)]
//
// use pin_project::{pin_project, pinned_drop};
// use std::pin::Pin;
//
// #[pin_project(PinnedDrop)]
// pub struct Foo<'a, T> {
//     was_dropped: &'a mut bool,
//     #[pin]
//     field: T,
// }
//
// #[pinned_drop]
// fn drop_foo<T>(mut this: Pin<&mut Foo<'_, T>>) {
//     **this.project().was_dropped = true;
// }
//
// fn main() {}
// ```

#![allow(dead_code, unused_imports)]

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

pub struct Foo<'a, T> {
    was_dropped: &'a mut bool,
    field: T,
}

#[allow(clippy::mut_mut)]
#[allow(dead_code)]
pub(crate) struct __FooProjection<'_pin, 'a, T> {
    was_dropped: &'_pin mut &'a mut bool,
    field: ::core::pin::Pin<&'_pin mut T>,
}

#[allow(dead_code)]
pub(crate) struct __FooProjectionRef<'_pin, 'a, T> {
    was_dropped: &'_pin &'a mut bool,
    field: ::core::pin::Pin<&'_pin T>,
}

impl<'a, T> Foo<'a, T> {
    pub(crate) fn project<'_pin>(
        self: ::core::pin::Pin<&'_pin mut Self>,
    ) -> __FooProjection<'_pin, 'a, T> {
        unsafe {
            let Foo { was_dropped, field } = self.get_unchecked_mut();
            __FooProjection {
                was_dropped: was_dropped,
                field: ::core::pin::Pin::new_unchecked(field),
            }
        }
    }
    pub(crate) fn project_ref<'_pin>(
        self: ::core::pin::Pin<&'_pin Self>,
    ) -> __FooProjectionRef<'_pin, 'a, T> {
        unsafe {
            let Foo { was_dropped, field } = self.get_ref();
            __FooProjectionRef {
                was_dropped: was_dropped,
                field: ::core::pin::Pin::new_unchecked(field),
            }
        }
    }
}

#[allow(single_use_lifetimes)]
impl<'a, T> ::core::ops::Drop for Foo<'a, T> {
    fn drop(&mut self) {
        // Safety - we're in 'drop', so we know that 'self' will
        // never move again.
        let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
        // We call `pinned_drop` only once. Since `PinnedDrop::drop`
        // is an unsafe function and a private API, it is never called again in safe
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
impl<T> ::pin_project::__private::PinnedDrop for Foo<'_, T> {
    // Since calling it twice on the same object would be UB,
    // this method is unsafe.
    unsafe fn drop(self: Pin<&mut Self>) {
        **self.project().was_dropped = true;
    }
}

// Automatically create the appropriate conditional `Unpin` implementation.
//
// See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/53.
// for details.
#[allow(non_snake_case)]
fn __unpin_scope_Foo() {
    struct AlwaysUnpinFoo<T: ?Sized> {
        val: ::core::marker::PhantomData<T>,
    }
    impl<T: ?Sized> ::core::marker::Unpin for AlwaysUnpinFoo<T> {}
    #[allow(dead_code)]
    #[doc(hidden)]
    pub struct __UnpinStructFoo<'a, T> {
        __pin_project_use_generics: AlwaysUnpinFoo<(T)>,
        __field0: T,
        __lifetime0: &'a (),
    }
    impl<'a, T> ::core::marker::Unpin for Foo<'a, T> where __UnpinStructFoo<'a, T>: ::core::marker::Unpin
    {}
}

// Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
//
// See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/34
// for details.
#[allow(single_use_lifetimes)]
#[allow(non_snake_case)]
#[deny(safe_packed_borrows)]
fn __pin_project_assert_not_repr_packed_Foo<'a, T>(val: Foo<'a, T>) {
    {
        &val.was_dropped;
    }
    {
        &val.field;
    }
}

fn main() {}
