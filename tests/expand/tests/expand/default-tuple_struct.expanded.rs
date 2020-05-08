use pin_project::pin_project;
#[pin(__private())]
struct TupleStruct<T, U>(#[pin] T, U);
#[doc(hidden)]
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
struct __TupleStructProjection<'pin, T, U>(
    ::pin_project::__reexport::pin::Pin<&'pin mut (T)>,
    &'pin mut (U),
)
where
    TupleStruct<T, U>: 'pin;
#[doc(hidden)]
#[allow(dead_code)]
struct __TupleStructProjectionRef<'pin, T, U>(
    ::pin_project::__reexport::pin::Pin<&'pin (T)>,
    &'pin (U),
)
where
    TupleStruct<T, U>: 'pin;
#[doc(hidden)]
#[allow(non_upper_case_globals)]
const __SCOPE_TupleStruct: () = {
    impl<T, U> TupleStruct<T, U> {
        fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> __TupleStructProjection<'pin, T, U> {
            unsafe {
                let Self(_0, _1) = self.get_unchecked_mut();
                __TupleStructProjection(::pin_project::__reexport::pin::Pin::new_unchecked(_0), _1)
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> __TupleStructProjectionRef<'pin, T, U> {
            unsafe {
                let Self(_0, _1) = self.get_ref();
                __TupleStructProjectionRef(
                    ::pin_project::__reexport::pin::Pin::new_unchecked(_0),
                    _1,
                )
            }
        }
    }
    struct __TupleStruct<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__reexport::marker::Unpin for TupleStruct<T, U> where
        __TupleStruct<'pin, T, U>: ::pin_project::__reexport::marker::Unpin
    {
    }
    trait TupleStructMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::pin_project::__reexport::ops::Drop> TupleStructMustNotImplDrop for T {}
    #[allow(single_use_lifetimes)]
    impl<T, U> TupleStructMustNotImplDrop for TupleStruct<T, U> {}
    #[allow(single_use_lifetimes)]
    impl<T, U> ::pin_project::__private::PinnedDrop for TupleStruct<T, U> {
        unsafe fn drop(self: ::pin_project::__reexport::pin::Pin<&mut Self>) {}
    }
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &TupleStruct<T, U>) {
        &val.0;
        &val.1;
    }
};
fn main() {}
