use pin_project::{pin_project, UnsafeUnpin};
#[pin(__private(UnsafeUnpin))]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::mut_mut)]
#[allow(clippy::type_repetition_in_bounds)]
struct __StructProjection<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__private::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::type_repetition_in_bounds)]
struct __StructProjectionRef<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__private::Pin<&'pin (T)>,
    unpinned: &'pin (U),
}
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl<T, U> Struct<T, U> {
        #[allow(dead_code)]
        fn project<'pin>(
            self: ::pin_project::__private::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                __StructProjection {
                    pinned: ::pin_project::__private::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        #[allow(dead_code)]
        fn project_ref<'pin>(
            self: ::pin_project::__private::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_ref();
                __StructProjectionRef {
                    pinned: ::pin_project::__private::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
    }
    impl<'pin, T, U> ::pin_project::__private::Unpin for Struct<T, U>
    where
        ::pin_project::__private::Wrapper<'pin, Self>: ::pin_project::UnsafeUnpin,
    {}
    trait StructMustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project::__private::Drop> StructMustNotImplDrop for T {}
    impl<T, U> StructMustNotImplDrop for Struct<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for Struct<T, U> {
        unsafe fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
    }
    #[forbid(unaligned_references, safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &Struct<T, U>) {
        let _ = &this.pinned;
        let _ = &this.unpinned;
    }
};
unsafe impl<T: Unpin, U> UnsafeUnpin for Struct<T, U> {}
fn main() {}
