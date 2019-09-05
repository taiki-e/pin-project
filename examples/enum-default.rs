//!
//! Original code:
//!
//! ```rust
//! #![allow(dead_code)]
//!
//! #[pin_project]
//! enum Enum<T, U> {
//!     Pinned(#[pin] T),
//!     Unpinned(U),
//! }
//!
//! fn main() {}
//! ```

#![allow(dead_code)]

enum Enum<T, U> {
    Pinned(T),
    Unpinned(U),
}

#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
enum __EnumProjection<'_pin, T, U> {
    Pinned(::core::pin::Pin<&'_pin mut T>),
    Unpinned(&'_pin mut U),
}

trait __EnumProjectionTrait<'_outer_pin, T, U> {
    fn project<'_pin>(&'_pin mut self) -> __EnumProjection<'_pin, T, U>;
    fn project_into(self) -> __EnumProjection<'_outer_pin, T, U>;
}

impl<'_outer_pin, T, U> __EnumProjectionTrait<'_outer_pin, T, U>
    for ::core::pin::Pin<&'_outer_pin mut Enum<T, U>>
{
    fn project<'_pin>(&'_pin mut self) -> __EnumProjection<'_pin, T, U> {
        unsafe {
            match self.as_mut().get_unchecked_mut() {
                Enum::Pinned(_x0) => __EnumProjection::Pinned(::core::pin::Pin::new_unchecked(_x0)),
                Enum::Unpinned(_x0) => __EnumProjection::Unpinned(_x0),
            }
        }
    }
    fn project_into(self) -> __EnumProjection<'_outer_pin, T, U> {
        unsafe {
            match self.get_unchecked_mut() {
                Enum::Pinned(_x0) => __EnumProjection::Pinned(::core::pin::Pin::new_unchecked(_x0)),
                Enum::Unpinned(_x0) => __EnumProjection::Unpinned(_x0),
            }
        }
    }
}

// Automatically create the appropriate conditional Unpin implementation.
impl<T, U> Unpin for Enum<T, U> where T: Unpin {}

// Ensure that enum does not implement `Drop`.
// There are two possible cases:
// 1. The user type does not implement Drop. In this case,
// the first blanked impl will not apply to it. This code
// will compile, as there is only one impl of MustNotImplDrop for the user type
// 2. The user type does impl Drop. This will make the blanket impl applicable,
// which will then comflict with the explicit MustNotImplDrop impl below.
// This will result in a compilation error, which is exactly what we want.
trait EnumMustNotImplDrop {}
#[allow(clippy::drop_bounds)]
impl<T: Drop> EnumMustNotImplDrop for T {}
#[allow(single_use_lifetimes)]
impl<T, U> EnumMustNotImplDrop for Enum<T, U> {}

// We don't need to check for '#[repr(packed)]',
// since it does not apply to enums.

fn main() {}
