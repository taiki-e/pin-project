use pin_project::pin_project;
# [ pin ( __private ( Replace , project = Proj , project_ref = ProjRef , project_replace = ProjOwn ) ) ]
struct TupleStruct<T, U>(#[pin] T, U);
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
struct Proj<'pin, T, U>(
    ::pin_project::__reexport::pin::Pin<&'pin mut (T)>,
    &'pin mut (U),
)
where
    TupleStruct<T, U>: 'pin;
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
struct ProjRef<'pin, T, U>(::pin_project::__reexport::pin::Pin<&'pin (T)>, &'pin (U))
where
    TupleStruct<T, U>: 'pin;
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
struct ProjOwn<T, U>(::pin_project::__reexport::marker::PhantomData<T>, U);
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
const __SCOPE_TupleStruct: () = {
    impl<T, U> TupleStruct<T, U> {
        fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> Proj<'pin, T, U> {
            unsafe {
                let Self(_0, _1) = self.get_unchecked_mut();
                Proj(::pin_project::__reexport::pin::Pin::new_unchecked(_0), _1)
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> ProjRef<'pin, T, U> {
            unsafe {
                let Self(_0, _1) = self.get_ref();
                ProjRef(::pin_project::__reexport::pin::Pin::new_unchecked(_0), _1)
            }
        }
        fn project_replace(
            self: ::pin_project::__reexport::pin::Pin<&mut Self>,
            __replacement: Self,
        ) -> ProjOwn<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let Self(_0, _1) = &mut *__self_ptr;
                let __result = ProjOwn(
                    ::pin_project::__reexport::marker::PhantomData,
                    ::pin_project::__reexport::ptr::read(_1),
                );
                let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                    target: __self_ptr,
                    value: ::pin_project::__reexport::mem::ManuallyDrop::new(__replacement),
                };
                {
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(_0);
                }
                __result
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
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for TupleStruct<T, U> {}
    trait TupleStructMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::pin_project::__reexport::ops::Drop> TupleStructMustNotImplDrop for T {}
    impl<T, U> TupleStructMustNotImplDrop for TupleStruct<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for TupleStruct<T, U> {
        unsafe fn drop(self: ::pin_project::__reexport::pin::Pin<&mut Self>) {}
    }
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &TupleStruct<T, U>) {
        &val.0;
        &val.1;
    }
};
fn main() {}
