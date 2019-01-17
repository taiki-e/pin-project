//! An attribute that would create a projection struct covering all the fields.
//!
//! ## Examples
//!
//! ```rust
//! use pin_project::unsafe_project;
//! use std::pin::Pin;
//!
//! #[unsafe_project(Unpin)]
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
//!
//! // You do not need to implement this manually.
//! // impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl
//! ```
//!
//! See [`unsafe_project`] for more details.
//!
//! [`unsafe_project`]: ./attr.unsafe_project.html
//!
//! ## Rust Version
//!
//! The current version of pin-project requires Rust nightly 2018-12-26 or later.
//!

#![crate_type = "proc-macro"]
#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project/0.1.4")]

extern crate proc_macro;

mod fields;
mod variants {}
mod enums {}
mod structs;
mod utils;

mod compile_fail;

use proc_macro::TokenStream;

/// An attribute that would create a projection struct covering all the fields.
///
/// This attribute creates a struct according to the following rules:
///
/// - For the field that uses `#[pin]` attribute, makes the pinned reference to
/// the field.
/// - For the other fields, makes the unpinned reference to the field.
///
/// ## Safety
///
/// For the field that uses `#[pin]` attribute, three things need to be ensured:
///
/// - If the struct implements [`Drop`], the [`drop`] method is not allowed to
///   move the value of the field.
/// - If the struct wants to implement [`Unpin`], it has to do so conditionally:
///   The struct can only implement [`Unpin`] if the field's type is [`Unpin`].
///   If you use `#[unsafe_project(Unpin)]`, you do not need to ensure this because
///   an appropriate [`Unpin`] implementation will be generated.
/// - The struct must not be `#[repr(packed)]`.
///
/// For the other fields, need to be ensured that the contained value not pinned
/// in the current context.
///
/// ## Examples
///
/// Using `#[unsafe_project(Unpin)]` will automatically create the appropriate [`Unpin`]
/// implementation:
///
/// ```rust
/// use pin_project::unsafe_project;
/// use std::pin::Pin;
///
/// #[unsafe_project(Unpin)]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     fn baz(mut self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.future; // Pinned reference to the field
///         let _: &mut U = this.field; // Normal reference to the field
///     }
/// }
///
/// // You do not need to implement this manually.
/// // impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
/// ```
///
/// If you want to implement [`Unpin`] manually:
///
/// ```rust
/// use pin_project::unsafe_project;
/// use std::marker::Unpin;
/// use std::pin::Pin;
///
/// #[unsafe_project]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     fn baz(mut self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.future; // Pinned reference to the field
///         let _: &mut U = this.field; // Normal reference to the field
///     }
/// }
///
/// impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl
/// ```
///
/// Note that borrowing the field where `#[pin]` attribute is used multiple
/// times requires using `.as_mut()` to avoid consuming the `Pin`.
///
/// [`Unpin`]: core::marker::Unpin
/// [`drop`]: Drop::drop
#[proc_macro_attribute]
pub fn unsafe_project(args: TokenStream, input: TokenStream) -> TokenStream {
    structs::unsafe_project(args, input)
}

/// An attribute that would create projections for each struct fields.
///
/// This is similar to [`unsafe_project`], but it is compatible with
/// [pin-utils].
///
/// This attribute creates methods according to the following rules:
///
/// - For the field that uses `#[pin]` attribute, the method that makes the pinned
/// reference to that field is created. This is the same as
/// [`pin_utils::unsafe_pinned`].
/// - For the field that uses `#[skip]` attribute, the method referencing that
/// field is not created.
/// - For the other fields, the method that makes the unpinned reference to that
/// field is created.This is the same as [`pin_utils::unsafe_unpinned`].
///
/// ## Safety
///
/// For the field that uses `#[pin]` attribute, three things need to be ensured:
///
/// - If the struct implements [`Drop`], the [`drop`] method is not allowed to
///   move the value of the field.
/// - If the struct wants to implement [`Unpin`], it has to do so conditionally:
///   The struct can only implement [`Unpin`] if the field's type is [`Unpin`].
///   If you use `#[unsafe_fields(Unpin)]`, you do not need to ensure this because
///   an appropriate [`Unpin`] implementation will be generated.
/// - The struct must not be `#[repr(packed)]`.
///
/// For the other fields, need to be ensured that the contained value not pinned
/// in the current context.
///
/// ## Examples
///
/// Using `#[unsafe_fields(Unpin)]` will automatically create the appropriate [`Unpin`]
/// implementation:
///
/// ```rust
/// use pin_project::unsafe_fields;
/// use std::pin::Pin;
///
/// #[unsafe_fields(Unpin)]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     fn baz(mut self: Pin<&mut Self>) {
///         let _: Pin<&mut T> = self.as_mut().future(); // Pinned reference to the field
///         let _: &mut U = self.as_mut().field(); // Normal reference to the field
///     }
/// }
///
/// // You do not need to implement this manually.
/// // impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
/// ```
///
/// If you want to implement [`Unpin`] manually:
///
/// ```rust
/// use pin_project::unsafe_fields;
/// use std::marker::Unpin;
/// use std::pin::Pin;
///
/// #[unsafe_fields]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     fn baz(mut self: Pin<&mut Self>) {
///         let _: Pin<&mut T> = self.as_mut().future(); // Pinned reference to the field
///         let _: &mut U = self.as_mut().field(); // Normal reference to the field
///     }
/// }
///
/// impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl
/// ```
///
/// Note that borrowing the field multiple times requires using `.as_mut()` to
/// avoid consuming the `Pin`.
///
/// [`unsafe_project`]: ./attr.unsafe_project.html
/// [`Unpin`]: core::marker::Unpin
/// [`drop`]: Drop::drop
/// [pin-utils]: https://github.com/rust-lang-nursery/pin-utils
/// [`pin_utils::unsafe_pinned`]: https://docs.rs/pin-utils/0.1.0-alpha/pin_utils/macro.unsafe_pinned.html
/// [`pin_utils::unsafe_unpinned`]: https://docs.rs/pin-utils/0.1.0-alpha/pin_utils/macro.unsafe_unpinned.html
#[proc_macro_attribute]
pub fn unsafe_fields(args: TokenStream, input: TokenStream) -> TokenStream {
    fields::unsafe_fields(args, input)
}
