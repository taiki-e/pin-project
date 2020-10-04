use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;
# [pin (__private (PinnedDrop , project = EnumProj , project_ref = EnumProjRef))]
enum Enum<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
    Unit,
}
#[allow(dead_code)]
#[allow(clippy::mut_mut)]
#[allow(clippy::type_repetition_in_bounds)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
#[allow(clippy::redundant_pub_crate)]
enum EnumProj<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Struct {
        pinned: ::pin_project::__private::Pin<&'pin mut (T)>,
        unpinned: &'pin mut (U),
    },
    Tuple(::pin_project::__private::Pin<&'pin mut (T)>, &'pin mut (U)),
    Unit,
}
#[allow(dead_code)]
#[allow(clippy::type_repetition_in_bounds)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
#[allow(clippy::redundant_pub_crate)]
enum EnumProjRef<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Struct {
        pinned: ::pin_project::__private::Pin<&'pin (T)>,
        unpinned: &'pin (U),
    },
    Tuple(::pin_project::__private::Pin<&'pin (T)>, &'pin (U)),
    Unit,
}
#[doc(hidden)]
#[allow(clippy::used_underscore_binding)]
#[allow(box_pointers)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::pattern_type_mismatch)]
#[allow(clippy::redundant_pub_crate)]
const _: () = {
    impl<T, U> Enum<T, U> {
        fn project<'pin>(
            self: ::pin_project::__private::Pin<&'pin mut Self>,
        ) -> EnumProj<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Enum::Struct { pinned, unpinned } => EnumProj::Struct {
                        pinned: ::pin_project::__private::Pin::new_unchecked(pinned),
                        unpinned,
                    },
                    Enum::Tuple(_0, _1) => {
                        EnumProj::Tuple(::pin_project::__private::Pin::new_unchecked(_0), _1)
                    }
                    Enum::Unit => EnumProj::Unit,
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__private::Pin<&'pin Self>,
        ) -> EnumProjRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    Enum::Struct { pinned, unpinned } => EnumProjRef::Struct {
                        pinned: ::pin_project::__private::Pin::new_unchecked(pinned),
                        unpinned,
                    },
                    Enum::Tuple(_0, _1) => {
                        EnumProjRef::Tuple(::pin_project::__private::Pin::new_unchecked(_0), _1)
                    }
                    Enum::Unit => EnumProjRef::Unit,
                }
            }
        }
    }
    struct __Enum<'pin, T, U> {
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
    impl<'pin, T, U> ::pin_project::__private::Unpin for Enum<T, U> where
        __Enum<'pin, T, U>: ::pin_project::__private::Unpin
    {
    }
    #[doc(hidden)]
    unsafe impl<'pin, T, U> ::pin_project::UnsafeUnpin for Enum<T, U> where
        __Enum<'pin, T, U>: ::pin_project::__private::Unpin
    {
    }
    impl<T, U> ::pin_project::__private::Drop for Enum<T, U> {
        fn drop(&mut self) {
            unsafe {
                let __pinned_self = ::pin_project::__private::Pin::new_unchecked(self);
                ::pin_project::__private::PinnedDrop::drop(__pinned_self);
            }
        }
    }
};
impl<T, U> ::pin_project::__private::PinnedDrop for Enum<T, U> {
    unsafe fn drop(self: Pin<&mut Self>) {
        #[allow(clippy::needless_pass_by_value)]
        fn __drop_inner<T, U>(__self: Pin<&mut Enum<T, U>>) {
            fn __drop_inner() {}
            let _this = __self;
        }
        __drop_inner(self);
    }
}
fn main() {}
