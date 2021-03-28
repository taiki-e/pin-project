use pin_project::pin_project;
#[pin(__private(project_replace))]
struct TupleStruct<T, U>(#[pin] T, #[pin] T, U, U);
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::mut_mut)]
#[allow(clippy::type_repetition_in_bounds)]
struct __TupleStructProjection<'pin, T, U>(
    ::pin_project::__private::Pin<&'pin mut (T)>,
    ::pin_project::__private::Pin<&'pin mut (T)>,
    &'pin mut (U),
    &'pin mut (U),
)
where
    TupleStruct<T, U>: 'pin;
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::type_repetition_in_bounds)]
struct __TupleStructProjectionRef<'pin, T, U>(
    ::pin_project::__private::Pin<&'pin (T)>,
    ::pin_project::__private::Pin<&'pin (T)>,
    &'pin (U),
    &'pin (U),
)
where
    TupleStruct<T, U>: 'pin;
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(unreachable_pub)]
struct __TupleStructProjectionOwned<T, U>(
    ::pin_project::__private::PhantomData<T>,
    ::pin_project::__private::PhantomData<T>,
    U,
    U,
);
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl<T, U> TupleStruct<T, U> {
        fn project<'pin>(
            self: ::pin_project::__private::Pin<&'pin mut Self>,
        ) -> __TupleStructProjection<'pin, T, U> {
            unsafe {
                let Self(_0, _1, _2, _3) = self.get_unchecked_mut();
                __TupleStructProjection(
                    ::pin_project::__private::Pin::new_unchecked(_0),
                    ::pin_project::__private::Pin::new_unchecked(_1),
                    _2,
                    _3,
                )
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__private::Pin<&'pin Self>,
        ) -> __TupleStructProjectionRef<'pin, T, U> {
            unsafe {
                let Self(_0, _1, _2, _3) = self.get_ref();
                __TupleStructProjectionRef(
                    ::pin_project::__private::Pin::new_unchecked(_0),
                    ::pin_project::__private::Pin::new_unchecked(_1),
                    _2,
                    _3,
                )
            }
        }
        fn project_replace(
            self: ::pin_project::__private::Pin<&mut Self>,
            __replacement: Self,
        ) -> __TupleStructProjectionOwned<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let Self(_0, _1, _2, _3) = &mut *__self_ptr;
                let __result = __TupleStructProjectionOwned(
                    ::pin_project::__private::PhantomData,
                    ::pin_project::__private::PhantomData,
                    ::pin_project::__private::ptr::read(_2),
                    ::pin_project::__private::ptr::read(_3),
                );
                let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                    target: __self_ptr,
                    value: ::pin_project::__private::ManuallyDrop::new(__replacement),
                };
                {
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(_1);
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(_0);
                }
                __result
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
        __field1: T,
    }
    impl<'pin, T, U> ::pin_project::__private::Unpin for TupleStruct<T, U> where
        __TupleStruct<'pin, T, U>: ::pin_project::__private::Unpin
    {
    }
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for TupleStruct<T, U> {}
    trait TupleStructMustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project::__private::Drop> TupleStructMustNotImplDrop for T {}
    impl<T, U> TupleStructMustNotImplDrop for TupleStruct<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for TupleStruct<T, U> {
        unsafe fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
    }
    #[forbid(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &TupleStruct<T, U>) {
        let _ = &this.0;
        let _ = &this.1;
        let _ = &this.2;
        let _ = &this.3;
    }
};
fn main() {}
