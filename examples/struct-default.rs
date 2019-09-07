//!
//! Original code:
//!
//! ```rust
//! #![allow(dead_code)]
//!
//! #[pin_project]
//! struct Struct<T, U> {
//!     #[pin]
//!     pinned: T,
//!     unpinned: U,
//! }
//!
//! fn main() {}
//! ```

#![allow(dead_code)]

struct Struct<T, U> {
    pinned: T,
    unpinned: U,
}

#[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
#[allow(dead_code)] // This lint warns unused fields/variants.
struct __StructProjection<'_pin, T, U> {
    pinned: ::core::pin::Pin<&'_pin mut T>,
    unpinned: &'_pin mut U,
}

impl<'_outer_pin, T, U> __StructProjectionTrait<'_outer_pin, T, U>
    for ::core::pin::Pin<&'_outer_pin mut Struct<T, U>>
{
    fn project<'_pin>(&'_pin mut self) -> __StructProjection<'_pin, T, U> {
        unsafe {
            let Struct { pinned, unpinned } = self.as_mut().get_unchecked_mut();
            __StructProjection {
                pinned: ::core::pin::Pin::new_unchecked(pinned),
                unpinned: unpinned,
            }
        }
    }
    fn project_into(self) -> __StructProjection<'_outer_pin, T, U> {
        unsafe {
            let Struct { pinned, unpinned } = self.get_unchecked_mut();
            __StructProjection {
                pinned: ::core::pin::Pin::new_unchecked(pinned),
                unpinned: unpinned,
            }
        }
    }
}

trait __StructProjectionTrait<'_outer_pin, T, U> {
    fn project<'_pin>(&'_pin mut self) -> __StructProjection<'_pin, T, U>;
    fn project_into(self) -> __StructProjection<'_outer_pin, T, U>;
}

// Automatically create the appropriate conditional `Unpin` implementation.
impl<T, U> Unpin for Struct<T, U> where T: Unpin {}

// Ensure that struct does not implement `Drop`.
//
// There are two possible cases:
// 1. The user type does not implement Drop. In this case,
// the first blanked impl will not apply to it. This code
// will compile, as there is only one impl of MustNotImplDrop for the user type
// 2. The user type does impl Drop. This will make the blanket impl applicable,
// which will then comflict with the explicit MustNotImplDrop impl below.
// This will result in a compilation error, which is exactly what we want.
trait StructMustNotImplDrop {}
#[allow(clippy::drop_bounds)]
impl<T: Drop> StructMustNotImplDrop for T {}
#[allow(single_use_lifetimes)]
impl<T, U> StructMustNotImplDrop for Struct<T, U> {}

// Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
//
// Taking a reference to a packed field is unsafe, amd appplying
// #[deny(safe_packed_borrows)] makes sure that doing this without
// an 'unsafe' block (which we deliberately do not generate)
// is a hard error.
//
// If the struct ends up having #[repr(packed)] applied somehow,
// this will generate an (unfriendly) error message. Under all reasonable
// circumstances, we'll detect the #[repr(packed)] attribute, and generate
// a much nicer error above.
//
// See https://github.com/taiki-e/pin-project/pull/34 for for more details.
#[allow(single_use_lifetimes)]
#[allow(non_snake_case)]
#[deny(safe_packed_borrows)]
fn __pin_project_assert_not_repr_packed_Struct<T, U>(val: Struct<T, U>) {
    {
        &val.pinned;
    }
    {
        &val.unpinned;
    }
}

fn main() {}
