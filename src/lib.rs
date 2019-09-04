//! A crate for safe and ergonomic pin-projection.
//!
//! This crate provides the following attribute macros:
//!
//! * [`pin_project`] - An attribute that creates a projection struct covering all the fields.
//! * [`pinned_drop`] - An attribute for annotating a function that implements `Drop`.
//! * [`project`] - An attribute to support pattern matching.
//!
//! NOTE: While this crate supports stable Rust, it currently requires
//! nightly Rust in order for rustdoc to correctly document auto-generated
//! `Unpin` impls. This does not affect the runtime functionality of this crate,
//! nor does it affect the safety of the api provided by this crate.
//!
//!
//! ## Examples
//!
//! [`pin_project`] attribute creates a projection struct covering all the fields.
//!
//! ```rust
//! use pin_project::pin_project;
//! use std::pin::Pin;
//!
//! #[pin_project]
//! struct Foo<T, U> {
//!     #[pin]
//!     future: T,
//!     field: U,
//! }
//!
//! impl<T, U> Foo<T, U> {
//!     fn baz(mut self: Pin<&mut Self>) {
//!         let this = self.project();
//!         let _: Pin<&mut T> = this.future; // Pinned reference to the field
//!         let _: &mut U = this.field; // Normal reference to the field
//!     }
//! }
//! ```
//!
//! <details>
//! <summary>Code like this will be generated:</summary>
//!
//! ```rust
//! struct Foo<T, U> {
//!     future: T,
//!     field: U,
//! }
//!
//! struct __FooProjection<'__a, T, U> {
//!     future: ::core::pin::Pin<&'__a mut T>,
//!     field: &'__a mut U,
//! }
//!
//! impl<T, U> Foo<T, U> {
//!     fn project<'__a>(self: ::core::pin::Pin<&'__a mut Self>) -> __FooProjection<'__a, T, U> {
//!         unsafe {
//!             let this = ::core::pin::Pin::get_unchecked_mut(self);
//!             __FooProjection {
//!                 future: ::core::pin::Pin::new_unchecked(&mut this.future),
//!                 field: &mut this.field,
//!             }
//!         }
//!     }
//! }
//!
//! // Automatically create the Drop implementation.
//! impl<T, U> Drop for Foo<T, U> {
//!     fn drop(&mut self) {
//!         // Do nothing. The precense of this Drop
//!         // impl ensures that the user can't provide one of their own
//!     }
//! }
//!
//! // Automatically create the appropriate conditional Unpin implementation.
//! impl<T, U> Unpin for Foo<T, U> where T: Unpin {}
//! ```
//!
//! </details>
//!
//! <br>
//!
//! See [`pin_project`] attribute for more details.
//!
//! [`pin_project`]: https://docs.rs/pin-project-internal/0.4.0-alpha.8/pin_project_internal/attr.pin_project.html
//! [`pinned_drop`]: https://docs.rs/pin-project-internal/0.4.0-alpha.8/pin_project_internal/attr.pinned_drop.html
//! [`project`]: https://docs.rs/pin-project-internal/0.4.0-alpha.8/pin_project_internal/attr.project.html

#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project/0.4.0-alpha.8")]
#![doc(test(attr(deny(warnings), allow(dead_code, unused_assignments, unused_variables))))]
#![no_std]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]
#![warn(single_use_lifetimes)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::use_self)]

#[doc(hidden)]
pub use pin_project_internal::pin_project;

#[doc(hidden)]
pub use pin_project_internal::pinned_drop;

#[cfg(feature = "project_attr")]
#[doc(hidden)]
pub use pin_project_internal::project;

/// A trait used for custom implementations of [`Unpin`].
/// This trait is used in conjunction with the `UnsafeUnpin`
/// argument to [`pin_project`]
///
/// The Rust [`Unpin`] trait is safe to implement - by itself,
/// implementing it cannot lead to undefined behavior. Undefined
/// behavior can only occur when other unsafe code is used.
///
/// It turns out that using pin projections, which requires unsafe code,
/// imposes additional requirements on an [`Unpin`] impl. Normally, all of this
/// unsafety is contained within this crate, ensuring that it's impossible for
/// you to violate any of the guarnatees required by pin projection.
///
/// However, things change if you want to provide a custom [`Unpin`] impl
/// for your `#[pin_project]` type. As stated in [the Rust
/// documentation](https://doc.rust-lang.org/beta/std/pin/index.html#projections-and-structural-pinning),
/// you must be sure to only implement [`Unpin`] when all of your `#[pin]` fields (i.e. struturally
/// pinend fields) are also [`Unpin`].
///
/// To help highlight this unsafety, the `UnsafeUnpin` trait is provided.
/// Implementing this trait is logically equivalent to implemnting [`Unpin`] -
/// this crate will generate an [`Unpin`] impl for your type that 'forwards' to
/// your `UnsafeUnpin` impl. However, this trait is `unsafe` - since your type
/// uses structural pinning (otherwise, you wouldn't be using this crate!),
/// you must be sure that your `UnsafeUnpin` impls follows all of
/// the requirements for an [`Unpin`] impl of a structurally-pinned type.
///
/// Note that if you specify `#[pin_project(UnsafeUnpin)]`, but do *not*
/// provide an impl of `UnsafeUnpin`, your type will never implement [`Unpin`].
/// This is effectly the same thing as adding a [`PhantomPinned`] to your type
///
/// Since this trait is `unsafe`, impls of it will be detected by the `unsafe_code` lint,
/// and by tools like `cargo geiger`.
///
/// ## Examples
///
/// An `UnsafeUnpin` impl which, in addition to requiring that structually pinned
/// fields be [`Unpin`], imposes an additional requirement:
///
/// ```rust
/// use pin_project::{pin_project, UnsafeUnpin};
///
/// #[pin_project(UnsafeUnpin)]
/// struct Foo<K, V> {
///     #[pin]
///     field_1: K,
///     field_2: V
/// }
///
/// unsafe impl<K, V> UnsafeUnpin for Foo<K, V> where K: Unpin + Clone {}
/// ```
///
/// [`PhantomPinned`]: core::marker::PhantomPinned
/// [`pin_project`]: https://docs.rs/pin-project-internal/0.4.0-alpha.8/pin_project_internal/attr.pin_project.html
#[allow(unsafe_code)]
pub unsafe trait UnsafeUnpin {}

use core::pin::Pin;

/// A helper trait to allow projecting through a mutable reference,
/// while preserving lifetimes.
///
/// Normally, `Pin::as_mut` can be used to convert a '&mut Pin' to a owned 'Pin'
/// However, since `Pin::as_mut'`needs to work with generic `DerefMut` impls,
/// it loses lifetime information. Specifically, it returns a `Pin<&'a mut P::Target>`,
/// where 'a is tied to the lifetime of the original '&mut Pin'. If you started
/// with a `&mut Pin<'pin &mut T>`, the 'pin lifetime is lost.
///
/// This can cause issues when trying to return the result of a pin projection
/// from a method - e.g. `fn foo<'a>(mut self: Pin<&'a mut Self>) -> Pin<&'a mut T>`
///
/// The `ProjectThrough` trait was introduced to solve this issue.
/// Normally, you'll never need to interact with this type
/// directly - it's used internally by pin-project to allow expressing
/// the proper lifetime.
pub trait ProjectThrough<'a> {
    type Target;
    fn proj_through(&mut self) -> Pin<&'a mut Self::Target>;
}

impl<'a, T> ProjectThrough<'a> for Pin<&'a mut T> {
    type Target = T;
    fn proj_through(&mut self) -> Pin<&'a mut Self::Target> {
        // This method is fairly tricky. We start out with
        // a `&'p mut Pin<&'a mut T>`, which we want
        // to convert into a `Pin<&'a mut T>`
        //
        // Effectively, we want to discard the outer '&mut'
        // reference, and just work with the inner 'Pin'
        // We accomplish this in four steps:
        //
        // First, call Pin::as_mut(self). This
        // converts our '&'p mut Pin<&'a mut T>` to
        // a `Pin<&mut T>`. This is *almost* what we want -
        // however, the lifetime is wrong. Since 'as_mut'
        // uses the `DerefMut` impl of `&mut T`, it loses
        // the lifetime associated with the `&mut T`.
        // However, we know that we're dealing with a
        // `Pin<&mut T>`, so we can soundly recover the original
        // lifetime.
        //
        // This relies on the following property:
        // The lifetime of the reference returned by <&'a mut T as DerefMut>::deref_mut
        // is the same as 'a. In order words, calling deref_mut on a mutable reference
        // is a no-op.
        let as_mut: Pin<&mut T> = Pin::as_mut(self);

        // Next, we unwrap the Pin<&mut T> by calling 'Pin::get_unchecked_mut'.
        // This gives us access to the underlying mutable reference
        #[allow(unsafe_code)]
        let raw_mut: &mut T = unsafe { Pin::get_unchecked_mut(as_mut) };

        // NExt, we transmute the mutable reference, changing its lifetime to 'a.
        // Here, we rely on the behavior of DerefMut for mutable references
        // described above.
        //
        // This is the core of this method - everything else is just to deconstruct
        // and reconstrut a Pin
        #[allow(unsafe_code)]
        let raw_transmuted: &'a mut T = unsafe { core::mem::transmute(raw_mut) };

        // Finally, we construct a Pin from our 'upgraded' mutable reference
        #[allow(unsafe_code)]
        unsafe {
            Pin::new_unchecked(raw_transmuted)
        }
    }
}

#[doc(hidden)]
pub mod __private {
    use super::UnsafeUnpin;
    use core::pin::Pin;

    // This is an internal helper trait used by `pin-project-internal`.
    // This allows us to force an error if the user tries to provide
    // a regular `Drop` impl when they specify the `PinnedDrop` argument.
    //
    // Users can implement `Drop` safely using `#[pinned_drop]`.
    // **Do not call or implement this trait directly.**
    #[allow(unsafe_code)]
    #[doc(hidden)]
    pub unsafe trait UnsafePinnedDrop {
        // Since calling it twice on the same object would be UB,
        // this method is unsafe.
        #[doc(hidden)]
        unsafe fn pinned_drop(self: Pin<&mut Self>);
    }

    // This is an internal helper struct used by `pin-project-internal`.
    // This allows us to force an error if the user tries to provide
    // a regular `Unpin` impl when they specify the `UnsafeUnpin` argument.
    // This is why we need Wrapper:
    //
    // Supposed we have the following code:
    //
    // #[pin_project(UnsafeUnpin)]
    // struct MyStruct<T> {
    //     #[pin] field: T
    // }
    //
    // impl<T> Unpin for MyStruct<T> where MyStruct<T>: UnsafeUnpin {} // generated by pin-project-internal
    // impl<T> Unpin for MyStruct<T> where T: Copy // written by the user
    //
    // We want this code to be rejected - the user is completely bypassing `UnsafeUnpin`,
    // and providing an unsound Unpin impl in safe code!
    //
    // Unfortunately, the Rust compiler will accept the above code.
    // Because MyStruct is declared in the same crate as the user-provided impl,
    // the compiler will notice that 'MyStruct<T>: UnsafeUnpin' never holds.
    //
    // The solution is to introduce the 'Wrapper' struct, which is defined
    // in the 'pin-project' crate.
    //
    // We now have code that looks like this:
    //
    // impl<T> Unpin for MyStruct<T> where Wrapper<MyStruct<T>>: UnsafeUnpin {} // generated by pin-project-internal
    // impl<T> Unpin for MyStruct<T> where T: Copy // written by the user
    //
    // We also have 'unsafe impl<T> UnsafeUnpin for Wrapper<T> where T: UnsafeUnpin {}' in the
    // 'pin-project' crate.
    //
    // Now, our generated impl has a bound involving a type defined in another crate - Wrapper.
    // This will cause rust to conservatively assume that 'Wrapper<MyStruct<T>>: UnsafeUnpin'
    // holds, in the interest of preserving forwards compatibility (in case such an impl is added
    // for Wrapper<T> in a new version of the crate).
    //
    // This will cause rust to reject any other Unpin impls for MyStruct<T>, since it will
    // assume that our generated impl could potentially apply in any situation.
    //
    // This acheives the desired effect - when the user writes `#[pin_project(UnsafeUnpin)]`,
    // the user must either provide no impl of `UnsafeUnpin` (which is equivalent
    // to making the type never implement Unpin), or provide an impl of `UnsafeUnpin`.
    // It is impossible for them to provide an impl of `Unpin`
    #[doc(hidden)]
    pub struct Wrapper<T>(T);

    #[allow(unsafe_code)]
    unsafe impl<T> UnsafeUnpin for Wrapper<T> where T: UnsafeUnpin {}
}
