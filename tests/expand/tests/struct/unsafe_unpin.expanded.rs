use pin_project::{pin_project, UnsafeUnpin};
#[pin(__private(UnsafeUnpin))]
pub struct Foo<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
pub(crate) struct __FooProjection<'pin, T, U>
where
    Foo<T, U>: 'pin,
{
    pinned: ::core::pin::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[allow(dead_code)]
pub(crate) struct __FooProjectionRef<'pin, T, U>
where
    Foo<T, U>: 'pin,
{
    pinned: ::core::pin::Pin<&'pin (T)>,
    unpinned: &'pin (U),
}
#[allow(non_upper_case_globals)]
const __SCOPE_Foo: () = {
    impl<T, U> Foo<T, U> {
        pub(crate) fn project<'pin>(
            self: ::core::pin::Pin<&'pin mut Self>,
        ) -> __FooProjection<'pin, T, U> {
            unsafe {
                let Foo { pinned, unpinned } = self.get_unchecked_mut();
                __FooProjection {
                    pinned: ::core::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        pub(crate) fn project_ref<'pin>(
            self: ::core::pin::Pin<&'pin Self>,
        ) -> __FooProjectionRef<'pin, T, U> {
            unsafe {
                let Foo { pinned, unpinned } = self.get_ref();
                __FooProjectionRef {
                    pinned: ::core::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
    }
    #[allow(single_use_lifetimes)]
    impl<'pin, T, U> ::core::marker::Unpin for Foo<T, U> where
        ::pin_project::__private::Wrapper<'pin, Self>: ::pin_project::UnsafeUnpin
    {
    }
    trait FooMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::core::ops::Drop> FooMustNotImplDrop for T {}
    #[allow(single_use_lifetimes)]
    impl<T, U> FooMustNotImplDrop for Foo<T, U> {}
    #[allow(single_use_lifetimes)]
    impl<T, U> ::pin_project::__private::PinnedDrop for Foo<T, U> {
        unsafe fn drop(self: ::core::pin::Pin<&mut Self>) {}
    }
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &Foo<T, U>) {
        &val.pinned;
        &val.unpinned;
    }
};
unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {}
fn main() {}
