use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;
#[pin(__private(PinnedDrop))]
pub struct Foo<'a, T> {
    was_dropped: &'a mut bool,
    #[pin]
    field: T,
}
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
pub(crate) struct __FooProjection<'pin, 'a, T>
where
    Foo<'a, T>: 'pin,
{
    was_dropped: &'pin mut (&'a mut bool),
    field: ::core::pin::Pin<&'pin mut (T)>,
}
#[allow(dead_code)]
pub(crate) struct __FooProjectionRef<'pin, 'a, T>
where
    Foo<'a, T>: 'pin,
{
    was_dropped: &'pin (&'a mut bool),
    field: ::core::pin::Pin<&'pin (T)>,
}
#[allow(non_upper_case_globals)]
const __SCOPE_Foo: () = {
    impl<'a, T> Foo<'a, T> {
        pub(crate) fn project<'pin>(
            self: ::core::pin::Pin<&'pin mut Self>,
        ) -> __FooProjection<'pin, 'a, T> {
            unsafe {
                let Foo { was_dropped, field } = self.get_unchecked_mut();
                __FooProjection {
                    was_dropped,
                    field: ::core::pin::Pin::new_unchecked(field),
                }
            }
        }
        pub(crate) fn project_ref<'pin>(
            self: ::core::pin::Pin<&'pin Self>,
        ) -> __FooProjectionRef<'pin, 'a, T> {
            unsafe {
                let Foo { was_dropped, field } = self.get_ref();
                __FooProjectionRef {
                    was_dropped,
                    field: ::core::pin::Pin::new_unchecked(field),
                }
            }
        }
    }
    pub struct __Foo<'pin, 'a, T> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T)>,
        __field0: T,
        __lifetime0: &'a (),
    }
    impl<'pin, 'a, T> ::core::marker::Unpin for Foo<'a, T> where
        __Foo<'pin, 'a, T>: ::core::marker::Unpin
    {
    }
    #[allow(single_use_lifetimes)]
    impl<'a, T> ::core::ops::Drop for Foo<'a, T> {
        fn drop(&mut self) {
            let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
            unsafe {
                ::pin_project::__private::PinnedDrop::drop(pinned_self);
            }
        }
    }
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<'a, T>(val: &Foo<'a, T>) {
        &val.was_dropped;
        &val.field;
    }
};
impl<T> ::pin_project::__private::PinnedDrop for Foo<'_, T> {
    unsafe fn drop(self: Pin<&mut Self>) {
        fn __drop_inner<T>(__self: Pin<&mut Foo<'_, T>>) {
            **__self.project().was_dropped = true;
        }
        __drop_inner(self);
    }
}
fn main() {}
