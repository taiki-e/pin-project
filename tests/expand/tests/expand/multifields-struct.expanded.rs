use pin_project::pin_project;
#[pin(__private(project_replace))]
struct Struct<T, U> {
    #[pin]
    pinned1: T,
    #[pin]
    pinned2: T,
    unpinned1: U,
    unpinned2: U,
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
    pinned1: ::pin_project::__private::Pin<&'pin mut (T)>,
    pinned2: ::pin_project::__private::Pin<&'pin mut (T)>,
    unpinned1: &'pin mut (U),
    unpinned2: &'pin mut (U),
}
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::type_repetition_in_bounds)]
struct __StructProjectionRef<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned1: ::pin_project::__private::Pin<&'pin (T)>,
    pinned2: ::pin_project::__private::Pin<&'pin (T)>,
    unpinned1: &'pin (U),
    unpinned2: &'pin (U),
}
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(unreachable_pub)]
struct __StructProjectionOwned<T, U> {
    pinned1: ::pin_project::__private::PhantomData<T>,
    pinned2: ::pin_project::__private::PhantomData<T>,
    unpinned1: U,
    unpinned2: U,
}
#[doc(hidden)]
#[allow(non_upper_case_globals)]
#[allow(single_use_lifetimes)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl<T, U> Struct<T, U> {
        fn project<'pin>(
            self: ::pin_project::__private::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, T, U> {
            unsafe {
                let Self {
                    pinned1,
                    pinned2,
                    unpinned1,
                    unpinned2,
                } = self.get_unchecked_mut();
                __StructProjection {
                    pinned1: ::pin_project::__private::Pin::new_unchecked(pinned1),
                    pinned2: ::pin_project::__private::Pin::new_unchecked(pinned2),
                    unpinned1,
                    unpinned2,
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__private::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Self {
                    pinned1,
                    pinned2,
                    unpinned1,
                    unpinned2,
                } = self.get_ref();
                __StructProjectionRef {
                    pinned1: ::pin_project::__private::Pin::new_unchecked(pinned1),
                    pinned2: ::pin_project::__private::Pin::new_unchecked(pinned2),
                    unpinned1,
                    unpinned2,
                }
            }
        }
        fn project_replace(
            self: ::pin_project::__private::Pin<&mut Self>,
            __replacement: Self,
        ) -> __StructProjectionOwned<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let Self {
                    pinned1,
                    pinned2,
                    unpinned1,
                    unpinned2,
                } = &mut *__self_ptr;
                let __result = __StructProjectionOwned {
                    pinned1: ::pin_project::__private::PhantomData,
                    pinned2: ::pin_project::__private::PhantomData,
                    unpinned1: ::pin_project::__private::ptr::read(unpinned1),
                    unpinned2: ::pin_project::__private::ptr::read(unpinned2),
                };
                let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                    target: __self_ptr,
                    value: ::pin_project::__private::ManuallyDrop::new(__replacement),
                };
                {
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(pinned2);
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(pinned1);
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
        __field1: T,
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
        let _ = &this.pinned1;
        let _ = &this.pinned2;
        let _ = &this.unpinned1;
        let _ = &this.unpinned2;
    }
};
fn main() {}
