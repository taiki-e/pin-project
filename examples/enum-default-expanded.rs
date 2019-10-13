// Original code (./enum-default.rs):
//
// ```rust
// #![allow(dead_code, unused_imports)]
//
// use pin_project::pin_project;
//
// #[pin_project]
// enum Enum<T, U> {
//     Pinned(#[pin] T),
//     Unpinned(U),
// }
//
// fn main() {}
// ```

#![allow(dead_code, unused_imports)]

use pin_project::pin_project;

enum Enum<T, U> {
    Pinned(/* #[pin] */ T),
    Unpinned(U),
}

#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
enum __EnumProjection<'_pin, T, U> {
    Pinned(::core::pin::Pin<&'_pin mut T>),
    Unpinned(&'_pin mut U),
}

#[allow(dead_code)] // This lint warns unused fields/variants.
enum __EnumProjectionRef<'_pin, T, U> {
    Pinned(::core::pin::Pin<&'_pin T>),
    Unpinned(&'_pin U),
}

impl<T, U> Enum<T, U> {
    fn project<'_pin>(self: ::core::pin::Pin<&'_pin mut Self>) -> __EnumProjection<'_pin, T, U> {
        unsafe {
            match self.get_unchecked_mut() {
                Enum::Pinned(_0) => __EnumProjection::Pinned(::core::pin::Pin::new_unchecked(_0)),
                Enum::Unpinned(_0) => __EnumProjection::Unpinned(_0),
            }
        }
    }
    fn project_ref<'_pin>(self: ::core::pin::Pin<&'_pin Self>) -> __EnumProjectionRef<'_pin, T, U> {
        unsafe {
            match self.get_ref() {
                Enum::Pinned(_0) => {
                    __EnumProjectionRef::Pinned(::core::pin::Pin::new_unchecked(_0))
                }
                Enum::Unpinned(_0) => __EnumProjectionRef::Unpinned(_0),
            }
        }
    }
}

// Automatically create the appropriate conditional `Unpin` implementation.
//
// See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/53.
// for details.
#[allow(non_snake_case)]
fn __unpin_scope_Enum() {
    #[allow(dead_code)]
    #[doc(hidden)]
    struct __UnpinStructEnum<'_pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'_pin, (T, U)>,
        __field0: T,
    }
    impl<'_pin, T, U> ::core::marker::Unpin for Enum<T, U> where
        __UnpinStructEnum<'_pin, T, U>: ::core::marker::Unpin
    {
    }
}

// Ensure that enum does not implement `Drop`.
//
// See ./struct-default-expanded.rs for details.
trait EnumMustNotImplDrop {}
#[allow(clippy::drop_bounds)]
impl<T: Drop> EnumMustNotImplDrop for T {}
#[allow(single_use_lifetimes)]
impl<T, U> EnumMustNotImplDrop for Enum<T, U> {}

// We don't need to check for '#[repr(packed)]',
// since it does not apply to enums.

fn main() {}
