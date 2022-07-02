use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;
#[pin(__private(PinnedDrop))]
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
    struct __Struct<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<
            'pin,
            (
                ::pin_project::__private::PhantomData<T>,
                ::pin_project::__private::PhantomData<U>,
            ),
        >,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__private::Unpin for Struct<T, U>
    where
        __Struct<'pin, T, U>: ::pin_project::__private::Unpin,
    {}
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for Struct<T, U> {}
    impl<T, U> ::pin_project::__private::Drop for Struct<T, U> {
        fn drop(&mut self) {
            let pinned_self = unsafe {
                ::pin_project::__private::Pin::new_unchecked(self)
            };
            unsafe {
                ::pin_project::__private::PinnedDrop::drop(pinned_self);
            }
        }
    }
    #[forbid(unaligned_references, safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &Struct<T, U>) {
        let _ = &this.pinned;
        let _ = &this.unpinned;
    }
};
impl<T, U> ::pin_project::__private::PinnedDrop for Struct<T, U> {
    unsafe fn drop(self: Pin<&mut Self>) {
        #[allow(clippy::needless_pass_by_value)]
        fn __drop_inner<T, U>(__self: Pin<&mut Struct<T, U>>) {
            fn __drop_inner() {}
            let _this = __self;
        }
        __drop_inner(self);
    }
}
fn main() {}
