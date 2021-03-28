use pin_project::pin_project;
# [pin (__private (project = Proj , project_ref = ProjRef , project_replace = ProjOwn))]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::mut_mut)]
#[allow(clippy::type_repetition_in_bounds)]
struct Proj<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__private::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::type_repetition_in_bounds)]
struct ProjRef<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__private::Pin<&'pin (T)>,
    unpinned: &'pin (U),
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(unreachable_pub)]
struct ProjOwn<T, U> {
    pinned: ::pin_project::__private::PhantomData<T>,
    unpinned: U,
}
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl<T, U> Struct<T, U> {
        fn project<'pin>(self: ::pin_project::__private::Pin<&'pin mut Self>) -> Proj<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                Proj {
                    pinned: ::pin_project::__private::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__private::Pin<&'pin Self>,
        ) -> ProjRef<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_ref();
                ProjRef {
                    pinned: ::pin_project::__private::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        fn project_replace(
            self: ::pin_project::__private::Pin<&mut Self>,
            __replacement: Self,
        ) -> ProjOwn<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let Self { pinned, unpinned } = &mut *__self_ptr;
                let __result = ProjOwn {
                    pinned: ::pin_project::__private::PhantomData,
                    unpinned: ::pin_project::__private::ptr::read(unpinned),
                };
                let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                    target: __self_ptr,
                    value: ::pin_project::__private::ManuallyDrop::new(__replacement),
                };
                {
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(pinned);
                }
                __result
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
    impl<'pin, T, U> ::pin_project::__private::Unpin for Struct<T, U> where
        __Struct<'pin, T, U>: ::pin_project::__private::Unpin
    {
    }
    unsafe impl<T, U> ::pin_project::UnsafeUnpin for Struct<T, U> {}
    trait StructMustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project::__private::Drop> StructMustNotImplDrop for T {}
    impl<T, U> StructMustNotImplDrop for Struct<T, U> {}
    impl<T, U> ::pin_project::__private::PinnedDrop for Struct<T, U> {
        unsafe fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
    }
    #[forbid(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &Struct<T, U>) {
        let _ = &this.pinned;
        let _ = &this.unpinned;
    }
};
fn main() {}
