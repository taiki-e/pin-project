use pin_project::pin_project;
#[pin(__private())]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
struct __StructProjection<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::core::pin::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[allow(dead_code)]
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
                let Struct { pinned, unpinned } = self.get_unchecked_mut();
                __StructProjection {
                    pinned: ::core::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        fn project_ref<'pin>(
            self: ::core::pin::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Struct { pinned, unpinned } = self.get_ref();
                __StructProjectionRef {
                    pinned: ::core::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
    }
    struct __Struct<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::core::marker::Unpin for Struct<T, U> where
        __Struct<'pin, T, U>: ::core::marker::Unpin
    {
    }
    trait StructMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::core::ops::Drop> StructMustNotImplDrop for T {}
    #[allow(single_use_lifetimes)]
    impl<T, U> StructMustNotImplDrop for Struct<T, U> {}
    #[allow(single_use_lifetimes)]
    impl<T, U> ::pin_project::__private::PinnedDrop for Struct<T, U> {
        unsafe fn drop(self: ::core::pin::Pin<&mut Self>) {}
    }
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &Struct<T, U>) {
        &val.pinned;
        &val.unpinned;
    }
};
fn main() {}
