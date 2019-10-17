#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;

#[test]
fn safe_project() {
    #[pin_project(PinnedDrop)]
    pub struct Foo<'a> {
        was_dropped: &'a mut bool,
        #[pin]
        field: u8,
    }

    #[pinned_drop]
    impl PinnedDrop for Foo<'_> {
        fn drop(self: Pin<&mut Self>) {
            **self.project().was_dropped = true;
        }
    }

    let mut was_dropped = false;
    drop(Foo { was_dropped: &mut was_dropped, field: 42 });
    assert!(was_dropped);
}

#[test]
fn test_mut_argument() {
    #[pin_project(PinnedDrop)]
    struct Struct {
        data: usize,
    }

    #[pinned_drop]
    impl PinnedDrop for Struct {
        fn drop(mut self: Pin<&mut Self>) {
            let _: &mut _ = &mut self.data;
        }
    }
}

#[test]
fn test_self_in_vec() {
    #[pin_project(PinnedDrop)]
    struct Struct {
        data: usize,
    }

    #[pinned_drop]
    impl PinnedDrop for Struct {
        fn drop(self: Pin<&mut Self>) {
            let _: Vec<_> = vec![self.data];
        }
    }
}

#[test]
fn test_self_in_macro_containing_fn() {
    #[pin_project(PinnedDrop)]
    pub struct Struct {
        data: usize,
    }

    macro_rules! emit {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }

    #[pinned_drop]
    impl PinnedDrop for Struct {
        fn drop(self: Pin<&mut Self>) {
            let _ = emit!({
                impl Struct {
                    pub fn f(self) {}
                }
            });
            self.data;
        }
    }
}

#[test]
fn test_call_self() {
    #[pin_project(PinnedDrop)]
    pub struct Struct {
        data: usize,
    }

    trait Trait {
        fn self_ref(&self) {}
        fn self_pin_ref(self: Pin<&Self>) {}
        fn self_mut(&mut self) {}
        fn self_pin_mut(self: Pin<&mut Self>) {}
        fn assoc_fn(_this: Pin<&mut Self>) {}
    }

    impl Trait for Struct {}

    #[pinned_drop]
    impl PinnedDrop for Struct {
        fn drop(mut self: Pin<&mut Self>) {
            self.self_ref();
            self.as_ref().self_pin_ref();
            self.self_mut();
            self.as_mut().self_pin_mut();
            Self::assoc_fn(self.as_mut());
            <Self>::assoc_fn(self.as_mut());
        }
    }
}

#[test]
fn test_self_match() {
    #[pin_project(PinnedDrop)]
    pub struct TupleStruct(usize);

    #[pinned_drop]
    impl PinnedDrop for TupleStruct {
        #[allow(irrefutable_let_patterns)]
        fn drop(mut self: Pin<&mut Self>) {
            match *self {
                Self(_) => {}
            }
            if let Self(_) = *self {}
            let _: Self = Self(0);
        }
    }
}
