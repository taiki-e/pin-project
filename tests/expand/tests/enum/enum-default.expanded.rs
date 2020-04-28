use pin_project::pin_project;
#[pin(__private())]
enum Enum<T, U> {
    Pinned(#[pin] T),
    Unpinned(U),
}
#[allow(clippy::mut_mut)]
#[allow(dead_code)]
enum __EnumProjection<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Pinned(::core::pin::Pin<&'pin mut (T)>),
    Unpinned(&'pin mut (U)),
}
#[allow(dead_code)]
enum __EnumProjectionRef<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Pinned(::core::pin::Pin<&'pin (T)>),
    Unpinned(&'pin (U)),
}
#[allow(non_upper_case_globals)]
const __SCOPE_Enum: () = {
    impl<T, U> Enum<T, U> {
        fn project<'pin>(self: ::core::pin::Pin<&'pin mut Self>) -> __EnumProjection<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Enum::Pinned(_0) => {
                        __EnumProjection::Pinned(::core::pin::Pin::new_unchecked(_0))
                    }
                    Enum::Unpinned(_0) => __EnumProjection::Unpinned(_0),
                }
            }
        }
        fn project_ref<'pin>(
            self: ::core::pin::Pin<&'pin Self>,
        ) -> __EnumProjectionRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    Enum::Pinned(_0) => {
                        __EnumProjectionRef::Pinned(::core::pin::Pin::new_unchecked(_0))
                    }
                    Enum::Unpinned(_0) => __EnumProjectionRef::Unpinned(_0),
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
