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

#![allow(dead_code, unused_imports, unused_parens, unknown_lints, renamed_and_removed_lints)]
#![allow(clippy::just_underscores_and_digits)]

use pin_project::pin_project;

// #[pin_project(project = EnumProj)]
enum Enum<T, U> {
    Pinned(/* #[pin] */ T),
    Unpinned(U),
}

#[allow(dead_code)]
#[allow(clippy::mut_mut)]
#[allow(clippy::type_repetition_in_bounds)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
enum EnumProj<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Pinned(::pin_project::__private::Pin<&'pin mut (T)>),
    Unpinned(&'pin mut (U)),
}

#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(clippy::used_underscore_binding)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
const _: () = {
    // When `#[pin_project]` is used on enums, only named projection types and
    // methods are generated because there is no way to access variants of
    // projected types without naming it.
    // (When `#[pin_project]` is used on structs, both methods are always generated.)

    impl<T, U> Enum<T, U> {
        fn project<'pin>(
            self: ::pin_project::__private::Pin<&'pin mut Self>,
        ) -> EnumProj<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Enum::Pinned(_0) => {
                        EnumProj::Pinned(::pin_project::__private::Pin::new_unchecked(_0))
                    }
                    Enum::Unpinned(_0) => EnumProj::Unpinned(_0),
                }
            }
        }
    }

    // Automatically create the appropriate conditional `Unpin` implementation.
    //
    // See ./struct-default-expanded.rs and https://github.com/taiki-e/pin-project/pull/53.
    // for details.
    struct __Enum<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<
            'pin,
            (::pin_project::__private::PhantomData<T>, ::pin_project::__private::PhantomData<U>),
        >,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__private::Unpin for Enum<T, U> where
        __Enum<'pin, T, U>: ::pin_project::__private::Unpin
    {
    }
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for Enum<T, U> {}

    // Ensure that enum does not implement `Drop`.
    //
    // See ./struct-default-expanded.rs for details.
    trait EnumMustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project::__private::Drop> EnumMustNotImplDrop for T {}
    impl<T, U> EnumMustNotImplDrop for Enum<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for Enum<T, U> {
        unsafe fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
    }

    // We don't need to check for `#[repr(packed)]`,
    // since it does not apply to enums.
};

fn main() {}
