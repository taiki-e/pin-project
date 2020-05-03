use pin_project::pin_project;
#[pin(__private(Replace))]
enum Enum<T, U> {
    V {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    None,
}
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
enum __EnumProjection<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    V {
        pinned: ::core::pin::Pin<&'pin mut (T)>,
        unpinned: &'pin mut (U),
    },
    None,
}
#[allow(dead_code)]
enum __EnumProjectionRef<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    V {
        pinned: ::core::pin::Pin<&'pin (T)>,
        unpinned: &'pin (U),
    },
    None,
}
#[allow(dead_code)]
enum __EnumProjectionOwned<T, U> {
    V {
        pinned: ::core::marker::PhantomData<T>,
        unpinned: U,
    },
    None,
}
#[allow(non_upper_case_globals)]
const __SCOPE_Enum: () = {
    impl<T, U> Enum<T, U> {
        fn project<'pin>(self: ::core::pin::Pin<&'pin mut Self>) -> __EnumProjection<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Enum::V { pinned, unpinned } => __EnumProjection::V {
                        pinned: ::core::pin::Pin::new_unchecked(pinned),
                        unpinned,
                    },
                    Enum::None => __EnumProjection::None,
                }
            }
        }
        fn project_ref<'pin>(
            self: ::core::pin::Pin<&'pin Self>,
        ) -> __EnumProjectionRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    Enum::V { pinned, unpinned } => __EnumProjectionRef::V {
                        pinned: ::core::pin::Pin::new_unchecked(pinned),
                        unpinned,
                    },
                    Enum::None => __EnumProjectionRef::None,
                }
            }
        }
        fn project_replace(
            self: ::core::pin::Pin<&mut Self>,
            __replacement: Self,
        ) -> __EnumProjectionOwned<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                match &mut *__self_ptr {
                    Enum::V { pinned, unpinned } => {
                        let __result = __EnumProjectionOwned::V {
                            pinned: ::core::marker::PhantomData,
                            unpinned: ::core::ptr::read(unpinned),
                        };
                        let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                            target: __self_ptr,
                            value: ::core::mem::ManuallyDrop::new(__replacement),
                        };
                        {
                            let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(pinned);
                        }
                        __result
                    }
                    Enum::None => {
                        let __result = __EnumProjectionOwned::None;
                        let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                            target: __self_ptr,
                            value: ::core::mem::ManuallyDrop::new(__replacement),
                        };
                        {}
                        __result
                    }
                }
            }
        }
    }
    struct __Enum<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::core::marker::Unpin for Enum<T, U> where __Enum<'pin, T, U>: ::core::marker::Unpin
    {}
    trait EnumMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::core::ops::Drop> EnumMustNotImplDrop for T {}
    #[allow(single_use_lifetimes)]
    impl<T, U> EnumMustNotImplDrop for Enum<T, U> {}
    #[allow(single_use_lifetimes)]
    impl<T, U> ::pin_project::__private::PinnedDrop for Enum<T, U> {
        unsafe fn drop(self: ::core::pin::Pin<&mut Self>) {}
    }
};
fn main() {}
