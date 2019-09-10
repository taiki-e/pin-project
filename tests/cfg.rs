#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

// Refs: https://doc.rust-lang.org/reference/attributes.html

use core::marker::PhantomPinned;
use pin_project::pin_project;

#[cfg(target_os = "linux")]
pub struct Linux;
#[cfg(not(target_os = "linux"))]
pub struct Other;

// Using `PhantomPinned: Unpin` without #![feature(trivial_bounds)] will result in an error.
// Use this type to check that `cfg(any())` is working properly.
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

    #[cfg(target_os = "linux")]
    let _x = Field::SameName { inner: Linux };
    #[cfg(not(target_os = "linux"))]
    let _x = Field::SameName { inner: Other };

    #[cfg(target_os = "linux")]
    let _x = Field::DifferentName { l: Linux };
    #[cfg(not(target_os = "linux"))]
    let _x = Field::DifferentName { w: Other };
}
