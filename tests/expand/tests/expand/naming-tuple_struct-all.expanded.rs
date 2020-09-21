use pin_project::pin_project;
# [pin (__private (project = Proj , project_ref = ProjRef , project_replace = ProjOwn))]
struct TupleStruct<T, U>(#[pin] T, U);
#[allow(dead_code)]
#[allow(clippy::mut_mut)]
#[allow(clippy::type_repetition_in_bounds)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
struct Proj<'pin, T, U>(::pin_project::__private::Pin<&'pin mut (T)>, &'pin mut (U))
where
    TupleStruct<T, U>: 'pin;
#[allow(dead_code)]
#[allow(clippy::type_repetition_in_bounds)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
struct ProjRef<'pin, T, U>(::pin_project::__private::Pin<&'pin (T)>, &'pin (U))
where
    TupleStruct<T, U>: 'pin;
#[allow(dead_code)]
#[allow(unreachable_pub)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
struct ProjOwn<T, U>(::pin_project::__private::PhantomData<T>, U);
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(clippy::used_underscore_binding)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
const _: () = {
    impl<T, U> TupleStruct<T, U> {
        fn project<'pin>(self: ::pin_project::__private::Pin<&'pin mut Self>) -> Proj<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    TupleStruct(_0, _1) => {
                        Proj(::pin_project::__private::Pin::new_unchecked(_0), _1)
                    }
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__private::Pin<&'pin Self>,
        ) -> ProjRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    TupleStruct(_0, _1) => {
                        ProjRef(::pin_project::__private::Pin::new_unchecked(_0), _1)
                    }
                }
            }
        }
        fn project_replace(
            self: ::pin_project::__private::Pin<&mut Self>,
            __replacement: Self,
        ) -> ProjOwn<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                match &mut *__self_ptr {
                    TupleStruct(_0, _1) => {
                        let __result = ProjOwn(
                            ::pin_project::__private::PhantomData,
                            ::pin_project::__private::ptr::read(_1),
                        );
                        let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                            target: __self_ptr,
                            value: ::pin_project::__private::ManuallyDrop::new(__replacement),
                        };
                        {
                            let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(_0);
                        }
                        __result
                    }
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
    trait TupleStructMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::pin_project::__private::Drop> TupleStructMustNotImplDrop for T {}
    impl<T, U> TupleStructMustNotImplDrop for TupleStruct<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for TupleStruct<T, U> {
        unsafe fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
    }
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &TupleStruct<T, U>) {
        &val.0;
        &val.1;
    }
};
fn main() {}
