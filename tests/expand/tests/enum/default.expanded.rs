use pin_project::pin_project;
#[pin(__private())]
enum Enum<T, U> {
    Pinned(#[pin] T),
    Unpinned(U),
}
#[doc(hidden)]
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
enum __EnumProjection<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Pinned(::pin_project::__reexport::pin::Pin<&'pin mut (T)>),
    Unpinned(&'pin mut (U)),
}
#[doc(hidden)]
#[allow(dead_code)]
enum __EnumProjectionRef<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Pinned(::pin_project::__reexport::pin::Pin<&'pin (T)>),
    Unpinned(&'pin (U)),
}
#[doc(hidden)]
#[allow(non_upper_case_globals)]
const __SCOPE_Enum: () = {
    impl<T, U> Enum<T, U> {
        fn project<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin mut Self>,
        ) -> __EnumProjection<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Enum::Pinned(_0) => __EnumProjection::Pinned(
                        ::pin_project::__reexport::pin::Pin::new_unchecked(_0),
                    ),
                    Enum::Unpinned(_0) => __EnumProjection::Unpinned(_0),
                }
            }
        }
        fn project_ref<'pin>(
            self: ::pin_project::__reexport::pin::Pin<&'pin Self>,
        ) -> __EnumProjectionRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    Enum::Pinned(_0) => __EnumProjectionRef::Pinned(
                        ::pin_project::__reexport::pin::Pin::new_unchecked(_0),
                    ),
                    Enum::Unpinned(_0) => __EnumProjectionRef::Unpinned(_0),
                }
            }
        }
    }
    struct __Enum<'pin, T, U> {
        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<'pin, (T, U)>,
        __field0: T,
    }
    impl<'pin, T, U> ::pin_project::__reexport::marker::Unpin for Enum<T, U> where
        __Enum<'pin, T, U>: ::pin_project::__reexport::marker::Unpin
    {
    }
    trait EnumMustNotImplDrop {}
    #[allow(clippy::drop_bounds)]
    impl<T: ::pin_project::__reexport::ops::Drop> EnumMustNotImplDrop for T {}
    #[allow(single_use_lifetimes)]
    impl<T, U> EnumMustNotImplDrop for Enum<T, U> {}
    #[allow(single_use_lifetimes)]
    impl<T, U> ::pin_project::__private::PinnedDrop for Enum<T, U> {
        unsafe fn drop(self: ::pin_project::__reexport::pin::Pin<&mut Self>) {}
    }
};
fn main() {}
