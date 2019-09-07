#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

// Refs: https://doc.rust-lang.org/reference/attributes.html

use core::marker::PhantomPinned;
use pin_project::pin_project;

#[cfg(all(unix, target_os = "macos"))]
pub struct MacOS;
#[cfg(all(unix, not(target_os = "macos")))]
pub struct Linux;
#[cfg(windows)]
pub struct Windows;

// Using `PhantomPinned: Unpin` without #![feature(trivial_bounds)] will result in an error.
// Use this type to check that `cfg(any())` is working properly.
pub struct Any(PhantomPinned);

#[test]
fn struct_() {
    #[pin_project]
    pub struct SameName {
        #[cfg(all(unix, target_os = "macos"))]
        #[pin]
        inner: MacOS,
        #[cfg(all(unix, not(target_os = "macos")))]
        #[pin]
        inner: Linux,
        #[cfg(windows)]
        #[pin]
        inner: Windows,
        #[cfg(any())]
        #[pin]
        any: Any,
    }

    #[cfg(all(unix, target_os = "macos"))]
    let _x = SameName { inner: MacOS };
    #[cfg(all(unix, not(target_os = "macos")))]
    let _x = SameName { inner: Linux };
    #[cfg(windows)]
    let _x = SameName { inner: Windows };

    #[pin_project]
    pub struct DifferentName {
        #[cfg(all(unix, target_os = "macos"))]
        #[pin]
        m: MacOS,
        #[cfg(all(unix, not(target_os = "macos")))]
        #[pin]
        l: Linux,
        #[cfg(windows)]
        #[pin]
        w: Windows,
        #[cfg(any())]
        #[pin]
        a: Any,
    }

    #[cfg(all(unix, target_os = "macos"))]
    let _x = DifferentName { m: MacOS };
    #[cfg(all(unix, not(target_os = "macos")))]
    let _x = DifferentName { l: Linux };
    #[cfg(windows)]
    let _x = DifferentName { w: Windows };
}

#[test]
fn enum_() {
    #[pin_project]
    pub enum Variant {
        #[cfg(all(unix, target_os = "macos"))]
        Inner(#[pin] MacOS),
        #[cfg(all(unix, not(target_os = "macos")))]
        Inner(#[pin] Linux),
        #[cfg(windows)]
        Inner(#[pin] Windows),

        #[cfg(all(unix, target_os = "macos"))]
        MacOS(#[pin] MacOS),
        #[cfg(all(unix, not(target_os = "macos")))]
        Linux(#[pin] Linux),
        #[cfg(windows)]
        Windows(#[pin] Windows),
        #[cfg(any())]
        Any(#[pin] Any),
    }

    #[cfg(all(unix, target_os = "macos"))]
    let _x = Variant::Inner(MacOS);
    #[cfg(all(unix, not(target_os = "macos")))]
    let _x = Variant::Inner(Linux);
    #[cfg(windows)]
    let _x = Variant::Inner(Windows);

    #[cfg(all(unix, target_os = "macos"))]
    let _x = Variant::MacOS(MacOS);
    #[cfg(all(unix, not(target_os = "macos")))]
    let _x = Variant::Linux(Linux);
    #[cfg(windows)]
    let _x = Variant::Windows(Windows);

    #[pin_project]
    pub enum Field {
        SameName {
            #[cfg(all(unix, target_os = "macos"))]
            #[pin]
            inner: MacOS,
            #[cfg(all(unix, not(target_os = "macos")))]
            #[pin]
            inner: Linux,
            #[cfg(windows)]
            #[pin]
            inner: Windows,
            #[cfg(any())]
            #[pin]
            any: Any,
        },
        DifferentName {
            #[cfg(all(unix, target_os = "macos"))]
            #[pin]
            m: MacOS,
            #[cfg(all(unix, not(target_os = "macos")))]
            #[pin]
            l: Linux,
            #[cfg(windows)]
            #[pin]
            w: Windows,
            #[cfg(any())]
            #[pin]
            any: Any,
        },
    }

    #[cfg(all(unix, target_os = "macos"))]
    let _x = Field::SameName { inner: MacOS };
    #[cfg(all(unix, not(target_os = "macos")))]
    let _x = Field::SameName { inner: Linux };
    #[cfg(windows)]
    let _x = Field::SameName { inner: Windows };

    #[cfg(all(unix, target_os = "macos"))]
    let _x = Field::DifferentName { m: MacOS };
    #[cfg(all(unix, not(target_os = "macos")))]
    let _x = Field::DifferentName { l: Linux };
    #[cfg(windows)]
    let _x = Field::DifferentName { w: Windows };
}
