#![warn(rust_2018_idioms, single_use_lifetimes)]
#![warn(unused, unused_results, future_incompatible)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod basic {
    include!("include/basic.rs");
}

pub mod forbid_unsafe {
    #![forbid(unsafe_code)]

    include!("include/basic-safe-part.rs");
}

pub mod clippy {
    use pin_project::pin_project;

    #[pin_project(Replace)]
    pub struct MutMutStruct<'a, T, U> {
        #[pin]
        pub pinned: &'a mut T,
        pub unpinned: &'a mut U,
    }

    #[pin_project(Replace)]
    pub struct MutMutTupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

    #[pin_project(Replace)]
    pub enum MutMutEnum<'a, T, U> {
        Struct {
            #[pin]
            pinned: &'a mut T,
            unpinned: &'a mut U,
        },
        Tuple(#[pin] &'a mut T, &'a mut U),
        Unit,
    }

    #[pin_project(Replace)]
    pub struct TypeRepetitionInBoundsStruct<T, U>
    where
        Self: Sized,
    {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }

    #[pin_project(Replace)]
    pub struct TypeRepetitionInBoundsTupleStruct<T, U>(#[pin] T, U)
    where
        Self: Sized;

    #[pin_project(Replace)]
    pub enum TypeRepetitionInBoundsEnum<T, U>
    where
        Self: Sized,
    {
        Struct {
            #[pin]
            pinned: T,
            unpinned: U,
        },
        Tuple(#[pin] T, U),
        Unit,
    }

    #[pin_project(Replace)]
    pub struct UsedUnderscoreBindingStruct<T, U> {
        #[pin]
        pub _pinned: T,
        pub _unpinned: U,
    }

    #[pin_project(Replace)]
    pub enum UsedUnderscoreBindingEnum<T, U> {
        Struct {
            #[pin]
            _pinned: T,
            _unpinned: U,
        },
    }
}
