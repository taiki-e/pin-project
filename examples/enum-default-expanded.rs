// Original code (./enum-default.rs):
//
// ```rust
// #![allow(dead_code)]
//
// use pin_project::pin_project;
//
// #[pin_project(project = EnumProj)]
// enum Enum<T, U> {
//     Pinned(#[pin] T),
//     Unpinned(U),
// }
//
// fn main() {}
// ```

#![allow(dead_code, unused_imports, unused_parens)]
#![allow(clippy::no_effect, clippy::just_underscores_and_digits)]

use pin_project::pin_project;

enum Enum<T, U> {
    Pinned(/* #[pin] */ T),
    Unpinned(U),
}

#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
#[allow(single_use_lifetimes)]
enum EnumProj<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Pinned(::pin_project::__reexport::pin::Pin<&'pin mut (T)>),
    Unpinned(&'pin mut (U)),
}
#[doc(hidden)]
#[allow(dead_code)] // This lint warns unused fields/variants.
#[allow(single_use_lifetimes)]
enum __EnumProjectionRef<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Pinned(::pin_project::__reexport::pin::Pin<&'pin (T)>),
    Unpinned(&'pin (U)),
}

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
const __SCOPE_Enum: () = {
    impl<T, U> Enum<T, U> {
        fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> EnumProj<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Enum::Pinned(_0) => {
                        EnumProj::Pinned(::pin_project::__reexport::pin::Pin::new_unchecked(_0))
                    }
                    Enum::Unpinned(_0) => EnumProj::Unpinned(_0),
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> __EnumProjectionRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    Enum::Pinned(_0) => __EnumProjectionRef::Pinned(
                        ::pin_project::__reexport::pin::Pin::new_unchecked(_0),
                    ),
                    Enum::Unpinned(_0) => __EnumProjectionRef::Unpinned(_0),
                }
            }
        }
    }

    // Automatically create the appropriate conditional `Unpin` implementation.
    //
    // See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/53.
    // for details.
    struct __Enum<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__reexport::marker::Unpin for Enum<T, U> where
        __Enum<'pin, T, U>: ::pin_project::__reexport::marker::Unpin
    {
    }
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for Enum<T, U> {}

    // Ensure that enum does not implement `Drop`.
    //
    // See ./struct-default-expanded.rs for details.
    trait EnumMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::pin_project::__reexport::ops::Drop> EnumMustNotImplDrop for T {}
    impl<T, U> EnumMustNotImplDrop for Enum<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for Enum<T, U> {
        unsafe fn drop(self: ::pin_project::__reexport::pin::Pin<&mut Self>) {}
    }

    // We don't need to check for `#[repr(packed)]`,
    // since it does not apply to enums.
};

fn main() {}
