#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

// Refs: https://doc.rust-lang.org/nightly/reference/attributes.html

use pin_project::pin_project;
use std::marker::PhantomPinned;

fn is_unpin<T: Unpin>() {}

#[cfg(target_os = "linux")]
pub struct Linux;
#[cfg(not(target_os = "linux"))]
pub struct Other;

// Use this type to check that `cfg(any())` is working properly.
// If `cfg(any())` is not working properly, `is_unpin` will fail.
pub struct Any(PhantomPinned);

#[test]
fn struct_() {
    #[pin_project]
    pub struct SameName {
        #[cfg(target_os = "linux")]
        #[pin]
        inner: Linux,
        #[cfg(not(target_os = "linux"))]
        #[pin]
        inner: Other,
        #[cfg(any())]
        #[pin]
        any: Any,
    }

    is_unpin::<SameName>();

    #[cfg(target_os = "linux")]
    let _x = SameName { inner: Linux };
    #[cfg(not(target_os = "linux"))]
    let _x = SameName { inner: Other };

    #[pin_project]
    pub struct DifferentName {
        #[cfg(target_os = "linux")]
        #[pin]
        l: Linux,
        #[cfg(not(target_os = "linux"))]
        #[pin]
        o: Other,
        #[cfg(any())]
        #[pin]
        a: Any,
    }

    is_unpin::<DifferentName>();

    #[cfg(target_os = "linux")]
    let _x = DifferentName { l: Linux };
    #[cfg(not(target_os = "linux"))]
    let _x = DifferentName { o: Other };
}

#[test]
fn enum_() {
    #[pin_project]
    pub enum Variant {
        #[cfg(target_os = "linux")]
        Inner(#[pin] Linux),
        #[cfg(not(target_os = "linux"))]
        Inner(#[pin] Other),

        #[cfg(target_os = "linux")]
        Linux(#[pin] Linux),
        #[cfg(not(target_os = "linux"))]
        Other(#[pin] Other),
        #[cfg(any())]
        Any(#[pin] Any),
    }

    is_unpin::<Variant>();

    #[cfg(target_os = "linux")]
    let _x = Variant::Inner(Linux);
    #[cfg(not(target_os = "linux"))]
    let _x = Variant::Inner(Other);

    #[cfg(target_os = "linux")]
    let _x = Variant::Linux(Linux);
    #[cfg(not(target_os = "linux"))]
    let _x = Variant::Other(Other);

    #[pin_project]
    pub enum Field {
        SameName {
            #[cfg(target_os = "linux")]
            #[pin]
            inner: Linux,
            #[cfg(not(target_os = "linux"))]
            #[pin]
            inner: Other,
            #[cfg(any())]
            #[pin]
            any: Any,
        },
        DifferentName {
            #[cfg(target_os = "linux")]
            #[pin]
            l: Linux,
            #[cfg(not(target_os = "linux"))]
            #[pin]
            w: Other,
            #[cfg(any())]
            #[pin]
            any: Any,
        },
    }

    is_unpin::<Field>();

    #[cfg(target_os = "linux")]
    let _x = Field::SameName { inner: Linux };
    #[cfg(not(target_os = "linux"))]
    let _x = Field::SameName { inner: Other };

    #[cfg(target_os = "linux")]
    let _x = Field::DifferentName { l: Linux };
    #[cfg(not(target_os = "linux"))]
    let _x = Field::DifferentName { w: Other };
}
