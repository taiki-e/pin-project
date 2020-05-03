// Original code (./struct-default.rs):
//
// ```rust
// #![allow(dead_code)]
//
// use pin_project::pin_project;
//
// #[pin_project]
// struct Struct<T, U> {
//     #[pin]
//     pinned: T,
//     unpinned: U,
// }
//
// fn main() {}
// ```

#![allow(dead_code, unused_imports, unused_parens)]

use pin_project::pin_project;

struct Struct<T, U> {
    // #[pin]
    pinned: T,
    unpinned: U,
}

#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
struct __StructProjection<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::core::pin::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[allow(dead_code)] // This lint warns unused fields/variants.
struct __StructProjectionRef<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::core::pin::Pin<&'pin (T)>,
    unpinned: &'pin (U),
}

#[allow(non_upper_case_globals)]
const __SCOPE_Struct: () = {
    impl<T, U> Struct<T, U> {
        fn project<'pin>(self: ::core::pin::Pin<&'pin mut Self>) -> __StructProjection<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                __StructProjection { pinned: ::core::pin::Pin::new_unchecked(pinned), unpinned }
            }
        }
        fn project_ref<'pin>(
            self: ::core::pin::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_ref();
                __StructProjectionRef { pinned: ::core::pin::Pin::new_unchecked(pinned), unpinned }
            }
        }
    }

    // Automatically create the appropriate conditional `Unpin` implementation.
    //
    // Basically this is equivalent to the following code:
    //
    // ```rust
    // impl<T, U> Unpin for Struct<T, U> where T: Unpin {}
    // ```
    //
    // However, if struct is public and there is a private type field,
    // this would cause an E0446 (private type in public interface).
    //
    // When RFC 2145 is implemented (rust-lang/rust#48054),
    // this will become a lint, rather then a hard error.
    //
    // As a workaround for this, we generate a new struct, containing all of the pinned
    // fields from our #[pin_project] type. This struct is declared within
    // a function, which makes it impossible to be named by user code.
    // This guarantees that it will use the default auto-trait impl for Unpin -
    // that is, it will implement Unpin iff all of its fields implement Unpin.
    // This type can be safely declared as 'public', satisfying the privacy
    // checker without actually allowing user code to access it.
    //
    // This allows users to apply the #[pin_project] attribute to types
    // regardless of the privacy of the types of their fields.
    //
    // See also https://github.com/taiki-e/pin-project/pull/53.
    struct __Struct<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::core::marker::Unpin for Struct<T, U> where
        __Struct<'pin, T, U>: ::core::marker::Unpin
    {
    }

    // Ensure that struct does not implement `Drop`.
    //
    // If you attempt to provide an Drop impl, the blanket impl will
    // then apply to your type, causing a compile-time error due to
    // the conflict with the second impl.
    trait StructMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::core::ops::Drop> StructMustNotImplDrop for T {}
    #[allow(single_use_lifetimes)]
    impl<T, U> StructMustNotImplDrop for Struct<T, U> {}
    // A dummy impl of `PinnedDrop`, to ensure that users don't accidentally
    // write a non-functional `PinnedDrop` impls.
    #[allow(single_use_lifetimes)]
    impl<T, U> ::pin_project::__private::PinnedDrop for Struct<T, U> {
        unsafe fn drop(self: ::core::pin::Pin<&mut Self>) {}
    }

    // Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
    //
    // Taking a reference to a packed field is unsafe, and applying
    // #[deny(safe_packed_borrows)] makes sure that doing this without
    // an 'unsafe' block (which we deliberately do not generate)
    // is a hard error.
    //
    // If the struct ends up having #[repr(packed)] applied somehow,
    // this will generate an (unfriendly) error message. Under all reasonable
    // circumstances, we'll detect the #[repr(packed)] attribute, and generate
    // a much nicer error above.
    //
    // See https://github.com/taiki-e/pin-project/pull/34 for more details.
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &Struct<T, U>) {
        &val.pinned;
        &val.unpinned;
    }
};

fn main() {}
