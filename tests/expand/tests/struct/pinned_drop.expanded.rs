use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;
#[pin(__private(PinnedDrop))]
pub struct Struct<'a, T> {
    was_dropped: &'a mut bool,
    #[pin]
    field: T,
}
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
pub(crate) struct __StructProjection<'pin, 'a, T>
where
    Struct<'a, T>: 'pin,
{
    was_dropped: &'pin mut (&'a mut bool),
    field: ::pin_project::__reexport::pin::Pin<&'pin mut (T)>,
}
#[allow(dead_code)]
pub(crate) struct __StructProjectionRef<'pin, 'a, T>
where
    Struct<'a, T>: 'pin,
{
    was_dropped: &'pin (&'a mut bool),
    field: ::pin_project::__reexport::pin::Pin<&'pin (T)>,
}
#[allow(non_upper_case_globals)]
const __SCOPE_Struct: () = {
    impl<'a, T> Struct<'a, T> {
        pub(crate) fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, 'a, T> {
            unsafe {
                let Self { was_dropped, field } = self.get_unchecked_mut();
                __StructProjection {
                    was_dropped,
                    field: ::pin_project::__reexport::pin::Pin::new_unchecked(field),
                }
            }
        }
        pub(crate) fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, 'a, T> {
            unsafe {
                let Self { was_dropped, field } = self.get_ref();
                __StructProjectionRef {
                    was_dropped,
                    field: ::pin_project::__reexport::pin::Pin::new_unchecked(field),
                }
            }
        }
    }
    pub struct __Struct<'pin, 'a, T> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T)>,
        __field0: T,
        __lifetime0: &'a (),
    }
    impl<'pin, 'a, T> ::pin_project::__reexport::marker::Unpin for Struct<'a, T> where
        __Struct<'pin, 'a, T>: ::pin_project::__reexport::marker::Unpin
    {
    }
    #[allow(single_use_lifetimes)]
    impl<'a, T> ::pin_project::__reexport::ops::Drop for Struct<'a, T> {
        fn drop(&mut self) {
            let pinned_self = unsafe { ::pin_project::__reexport::pin::Pin::new_unchecked(self) };
            unsafe {
                ::pin_project::__private::PinnedDrop::drop(pinned_self);
            }
        }
    }
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<'a, T>(val: &Struct<'a, T>) {
        &val.was_dropped;
        &val.field;
    }
};
impl<T> ::pin_project::__private::PinnedDrop for Struct<'_, T> {
    unsafe fn drop(self: Pin<&mut Self>) {
        fn __drop_inner<T>(__self: Pin<&mut Struct<'_, T>>) {
            **__self.project().was_dropped = true;
        }
        __drop_inner(self);
    }
}
fn main() {}
