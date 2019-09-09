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
    Pinned(T),
    Unpinned(U),
}

#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
enum __EnumProjection<'_pin, T, U> {
    Pinned(::core::pin::Pin<&'_pin mut T>),
    Unpinned(&'_pin mut U),
}

impl<'_outer_pin, T, U> __EnumProjectionTrait<'_outer_pin, T, U>
    for ::core::pin::Pin<&'_outer_pin mut Enum<T, U>>
{
    fn project<'_pin>(&'_pin mut self) -> __EnumProjection<'_pin, T, U> {
        unsafe {
            match self.as_mut().get_unchecked_mut() {
                Enum::Pinned(_x0) => __EnumProjection::Pinned(::core::pin::Pin::new_unchecked(_x0)),
                Enum::Unpinned(_x0) => __EnumProjection::Unpinned(_x0),
            }
        }
    }
    fn project_into(self) -> __EnumProjection<'_outer_pin, T, U> {
        unsafe {
            match self.get_unchecked_mut() {
                Enum::Pinned(_x0) => __EnumProjection::Pinned(::core::pin::Pin::new_unchecked(_x0)),
                Enum::Unpinned(_x0) => __EnumProjection::Unpinned(_x0),
            }
        }
    }
}

trait __EnumProjectionTrait<'_outer_pin, T, U> {
    fn project<'_pin>(&'_pin mut self) -> __EnumProjection<'_pin, T, U>;
    fn project_into(self) -> __EnumProjection<'_outer_pin, T, U>;
}

// Automatically create the appropriate conditional `Unpin` implementation.
//
// See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/53.
// for details.
#[allow(non_snake_case)]
fn __unpin_scope_Enum() {
    struct AlwaysUnpinEnum<T: ?Sized> {
        val: ::core::marker::PhantomData<T>,
    }
    impl<T: ?Sized> ::core::marker::Unpin for AlwaysUnpinEnum<T> {}
    #[allow(dead_code)]
    #[doc(hidden)]
    struct __UnpinStructEnum<T, U> {
        __pin_project_use_generics: AlwaysUnpinEnum<(T, U)>,
        __field0: T,
    }
    impl<T, U> ::core::marker::Unpin for Enum<T, U> where __UnpinStructEnum<T, U>: ::core::marker::Unpin {}
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
