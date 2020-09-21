use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;
#[pin(__private(PinnedDrop))]
struct TupleStruct<T, U>(#[pin] T, U);
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(clippy::used_underscore_binding)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
const _: () = {
    #[allow(dead_code)]
    #[allow(clippy::mut_mut)]
    #[allow(clippy::type_repetition_in_bounds)]
    #[allow(box_pointers)]
    #[allow(explicit_outlives_requirements)]
    #[allow(single_use_lifetimes)]
    #[allow(clippy::pattern_type_mismatch)]
    struct __TupleStructProjection<'pin, T, U>(
        ::pin_project::__private::Pin<&'pin mut (T)>,
        &'pin mut (U),
    )
    where
        TupleStruct<T, U>: 'pin;
    #[allow(dead_code)]
    #[allow(clippy::type_repetition_in_bounds)]
    #[allow(box_pointers)]
    #[allow(explicit_outlives_requirements)]
    #[allow(single_use_lifetimes)]
    #[allow(clippy::pattern_type_mismatch)]
    struct __TupleStructProjectionRef<'pin, T, U>(
        ::pin_project::__private::Pin<&'pin (T)>,
        &'pin (U),
    )
    where
        TupleStruct<T, U>: 'pin;
    impl<T, U> TupleStruct<T, U> {
        fn project<'pin>(
            self: ::pin_project::__private::Pin<&'pin mut Self>,
        ) -> __TupleStructProjection<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    TupleStruct(_0, _1) => __TupleStructProjection(
                        ::pin_project::__private::Pin::new_unchecked(_0),
                        _1,
                    ),
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__private::Pin<&'pin Self>,
        ) -> __TupleStructProjectionRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    TupleStruct(_0, _1) => __TupleStructProjectionRef(
                        ::pin_project::__private::Pin::new_unchecked(_0),
                        _1,
                    ),
                }
            }
        }
    }
    struct __TupleStruct<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<
            'pin,
            (
                ::pin_project::__private::PhantomData<T>,
                ::pin_project::__private::PhantomData<U>,
            ),
        >,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__private::Unpin for TupleStruct<T, U> where
        __TupleStruct<'pin, T, U>: ::pin_project::__private::Unpin
    {
    }
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for TupleStruct<T, U> {}
    impl<T, U> ::pin_project::__private::Drop for TupleStruct<T, U> {
        fn drop(&mut self) {
            let pinned_self = unsafe { ::pin_project::__private::Pin::new_unchecked(self) };
            unsafe {
                ::pin_project::__private::PinnedDrop::drop(pinned_self);
            }
        }
    }
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &TupleStruct<T, U>) {
        &val.0;
        &val.1;
    }
};
impl<T, U> ::pin_project::__private::PinnedDrop for TupleStruct<T, U> {
    unsafe fn drop(self: Pin<&mut Self>) {
        #[allow(clippy::needless_pass_by_value)]
        fn __drop_inner<T, U>(__self: Pin<&mut TupleStruct<T, U>>) {
            fn __drop_inner() {}
            let _this = __self;
        }
        __drop_inner(self);
    }
}
fn main() {}
