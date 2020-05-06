use pin_project::pin_project;
#[pin(__private(Replace))]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
struct __StructProjection<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__reexport::pin::Pin<&'pin mut (T)>,
    unpinned: &'pin mut (U),
}
#[allow(dead_code)]
struct __StructProjectionRef<'pin, T, U>
where
    Struct<T, U>: 'pin,
{
    pinned: ::pin_project::__reexport::pin::Pin<&'pin (T)>,
    unpinned: &'pin (U),
}
#[allow(dead_code)]
struct __StructProjectionOwned<T, U> {
    pinned: ::pin_project::__reexport::marker::PhantomData<T>,
    unpinned: U,
}
#[allow(non_upper_case_globals)]
const __SCOPE_Struct: () = {
    impl<T, U> Struct<T, U> {
        fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                __StructProjection {
                    pinned: ::pin_project::__reexport::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_ref();
                __StructProjectionRef {
                    pinned: ::pin_project::__reexport::pin::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        fn project_replace(
            self: ::pin_project::__reexport::pin::Pin<&mut Self>,
            __replacement: Self,
        ) -> __StructProjectionOwned<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let Self { pinned, unpinned } = &mut *__self_ptr;
                let __result = __StructProjectionOwned {
                    pinned: ::pin_project::__reexport::marker::PhantomData,
                    unpinned: ::pin_project::__reexport::ptr::read(unpinned),
                };
                let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                    target: __self_ptr,
                    value: ::pin_project::__reexport::mem::ManuallyDrop::new(__replacement),
                };
                {
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(pinned);
                }
                __result
            }
        }
    }
    struct __Struct<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__reexport::marker::Unpin for Struct<T, U> where
        __Struct<'pin, T, U>: ::pin_project::__reexport::marker::Unpin
    {
    }
    trait StructMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::pin_project::__reexport::ops::Drop> StructMustNotImplDrop for T {}
    #[allow(single_use_lifetimes)]
    impl<T, U> StructMustNotImplDrop for Struct<T, U> {}
    #[allow(single_use_lifetimes)]
    impl<T, U> ::pin_project::__private::PinnedDrop for Struct<T, U> {
        unsafe fn drop(self: ::pin_project::__reexport::pin::Pin<&mut Self>) {}
    }
    #[allow(single_use_lifetimes)]
    #[deny(safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(val: &Struct<T, U>) {
        &val.pinned;
        &val.unpinned;
    }
};
fn main() {}
