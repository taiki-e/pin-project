//! An attribute that creates a projection struct covering all the fields.
//!
//! ## Examples
//!
//! [`unsafe_project`] attribute creates a projection struct covering all the fields.
//!
//! ```rust
//! use pin_project::unsafe_project;
//! use std::pin::Pin;
//!
//! #[unsafe_project(Unpin)] // `(Unpin)` is optional (create the appropriate conditional Unpin implementation)
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
//! // Automatically create the appropriate conditional Unpin implementation (optional).
//! // impl<T: Unpin, U> Unpin for Foo<T, U> {}
//! ```
//!
//! <details>
//! <summary>Code like this will be generated:</summary>
//!
//! ```rust
//! # use std::pin::Pin;
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
//! // Automatically create the appropriate conditional Unpin implementation (optional).
//! impl<T, U> Unpin for Foo<T, U> where T: Unpin {}
//! ```
//!
//! </details>
//!
//! [`unsafe_project`] also supports enums, but to use it ergonomically, you need
//! to use the [`project`] attribute.
//!
//! ```rust
//! # #[cfg(feature = "project_attr")]
//! use pin_project::{project, unsafe_project};
//! # #[cfg(feature = "project_attr")]
//! use std::pin::Pin;
//!
//! # #[cfg(feature = "project_attr")]
//! #[unsafe_project(Unpin)] // `(Unpin)` is optional (create the appropriate conditional Unpin implementation)
//! enum Foo<T, U> {
//!     Future(#[pin] T),
//!     Done(U),
//! }
//!
//! # #[cfg(feature = "project_attr")]
//! impl<T, U> Foo<T, U> {
//!     #[project] // Nightly does not need a dummy attribute to the function.
//!     fn baz(mut self: Pin<&mut Self>) {
//!         #[project]
//!         match self.project() {
//!             Foo::Future(future) => {
//!                 let _: Pin<&mut T> = future;
//!             }
//!             Foo::Done(value) => {
//!                 let _: &mut U = value;
//!             }
//!         }
//!     }
//! }
//!
//! // Automatically create the appropriate conditional Unpin implementation (optional).
//! // impl<T, U> Unpin for Foo<T, U> where T: Unpin {}
//! ```
//!
//! <details>
//! <summary>Code like this will be generated:</summary>
//!
//! ```rust
//! # use std::pin::Pin;
//! enum Foo<T, U> {
//!     Future(T),
//!     Done(U),
//! }
//!
//! enum __FooProjection<'__a, T, U> {
//!     Future(::core::pin::Pin<&'__a mut T>),
//!     Done(&'__a mut U),
//! }
//!
//! impl<T, U> Foo<T, U> {
//!     fn project<'__a>(self: ::core::pin::Pin<&'__a mut Self>) -> __FooProjection<'__a, T, U> {
//!         unsafe {
//!             match ::core::pin::Pin::get_unchecked_mut(self) {
//!                 Foo::Future(_x0) => __FooProjection::Future(::core::pin::Pin::new_unchecked(_x0)),
//!                 Foo::Done(_x0) => __FooProjection::Done(_x0),
//!             }
//!         }
//!     }
//! }
//!
//! // Automatically create the appropriate conditional Unpin implementation (optional).
//! impl<T, U> Unpin for Foo<T, U> where T: Unpin {}
//! ```
//!
//! </details>
//!
//! See [`unsafe_project`] and [`project`] for more details.
//!
//! [`unsafe_project`]: ./attr.unsafe_project.html
//! [`project`]: ./attr.project.html
//!

#![crate_type = "proc-macro"]
#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project/0.3.1")]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms, unreachable_pub)]
#![deny(clippy::all, clippy::pedantic)]

extern crate proc_macro;

#[cfg(feature = "project_attr")]
mod project;
mod unsafe_project;
mod utils;

mod compile_fail;

use proc_macro::TokenStream;

/// An attribute that creates a projection struct covering all the fields.
///
/// This attribute creates a projection struct according to the following rules:
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
///   If you use `#[unsafe_project(Unpin)]`, you do not need to ensure this
///   because an appropriate conditional [`Unpin`] implementation will be
///   generated.
/// - The struct must not be `#[repr(packed)]`.
///
/// For the other fields, need to be ensured that the contained value not pinned
/// in the current context.
///
/// ## Examples
///
/// Using `#[unsafe_project(Unpin)]` will automatically create the appropriate
/// conditional [`Unpin`] implementation:
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
/// // Automatically create the appropriate conditional Unpin implementation.
/// // impl<T, U> Unpin for Foo<T, U> where T: Unpin {}
/// ```
///
/// If you want to implement [`Unpin`] manually:
///
/// ```rust
/// use pin_project::unsafe_project;
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
/// ## Supported Items
///
/// The current version of pin-project supports the following types of items.
///
/// ### Structs (structs with named fields):
///
/// ```rust
/// # use pin_project::unsafe_project;
/// # use std::pin::Pin;
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
///         let _: Pin<&mut T> = this.future;
///         let _: &mut U = this.field;
///     }
/// }
/// ```
///
/// ### Tuple structs (structs with unnamed fields):
///
/// ```rust
/// # use pin_project::unsafe_project;
/// # use std::pin::Pin;
/// #[unsafe_project(Unpin)]
/// struct Foo<T, U>(#[pin] T, U);
///
/// impl<T, U> Foo<T, U> {
///     fn baz(mut self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.0;
///         let _: &mut U = this.1;
///     }
/// }
/// ```
///
/// Structs without fields (unit-like struct and zero fields struct) are not
/// supported.
///
/// ### Enums
///
/// `unsafe_project` also supports enums, but to use it ergonomically, you need
/// to use the [`project`] attribute.
///
/// ```rust
/// # #[cfg(feature = "project_attr")]
/// use pin_project::{project, unsafe_project};
/// # #[cfg(feature = "project_attr")]
/// # use std::pin::Pin;
///
/// # #[cfg(feature = "project_attr")]
/// #[unsafe_project(Unpin)]
/// enum Foo<A, B, C> {
///     Tuple(#[pin] A, B),
///     Struct { field: C },
///     Unit,
/// }
///
/// # #[cfg(feature = "project_attr")]
/// impl<A, B, C> Foo<A, B, C> {
///     #[project] // Nightly does not need a dummy attribute to the function.
///     fn baz(self: Pin<&mut Self>) {
///         #[project]
///         match self.project() {
///             Foo::Tuple(x, y) => {
///                 let _: Pin<&mut A> = x;
///                 let _: &mut B = y;
///             }
///             Foo::Struct { field } => {
///                 let _: &mut C = field;
///             }
///             Foo::Unit => {}
///         }
///     }
/// }
/// ```
///
/// Enums without variants (zero-variant enums) are not supported.
///
/// Also see [`project`] attribute.
///
/// [`Unpin`]: core::marker::Unpin
/// [`drop`]: Drop::drop
/// [`project`]: ./attr.project.html
#[proc_macro_attribute]
pub fn unsafe_project(args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(unsafe_project::unsafe_project(&args.into(), input.into()))
}

/// An attribute to support pattern matching.
///
/// *This attribute is available if pin-project is built with the
/// `"project_attr"` feature (it is enabled by default).*
///
/// ## Examples
///
/// ### `let` bindings
///
/// ```rust
/// use pin_project::{project, unsafe_project};
/// # use std::pin::Pin;
///
/// #[unsafe_project(Unpin)]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     #[project] // Nightly does not need a dummy attribute to the function.
///     fn baz(mut self: Pin<&mut Self>) {
///         #[project]
///         let Foo { future, field } = self.project();
///
///         let _: Pin<&mut T> = future;
///         let _: &mut U = field;
///     }
/// }
/// ```
///
/// ### `match` expressions
///
/// ```rust
/// use pin_project::{project, unsafe_project};
/// # use std::pin::Pin;
///
/// #[unsafe_project(Unpin)]
/// enum Foo<A, B, C> {
///     Tuple(#[pin] A, B),
///     Struct { field: C },
///     Unit,
/// }
///
/// impl<A, B, C> Foo<A, B, C> {
///     #[project] // Nightly does not need a dummy attribute to the function.
///     fn baz(self: Pin<&mut Self>) {
///         #[project]
///         match self.project() {
///             Foo::Tuple(x, y) => {
///                 let _: Pin<&mut A> = x;
///                 let _: &mut B = y;
///             }
///             Foo::Struct { field } => {
///                 let _: &mut C = field;
///             }
///             Foo::Unit => {}
///         }
///     }
/// }
/// ```
///
/// ### `if let` expressions
///
/// When used against `if let` expressions, the `#[project]` attribute records
/// the name of the structure destructed with the first `if let`. Destructing
/// different structures in the after second times will not generate wrong code.
///
/// ```rust
/// use pin_project::{project, unsafe_project};
/// # use std::pin::Pin;
///
/// #[unsafe_project(Unpin)]
/// enum Foo<A, B, C> {
///     Tuple(#[pin] A, B),
///     Struct { field: C },
///     Unit,
/// }
///
/// impl<A, B, C> Foo<A, B, C> {
///     #[project] // Nightly does not need a dummy attribute to the function.
///     fn baz(self: Pin<&mut Self>) {
///         #[project]
///         {
///             if let Foo::Tuple(x, y) = self.project() {
///                 let _: Pin<&mut A> = x;
///                 let _: &mut B = y;
///             }
///         }
///     }
/// }
/// ```
#[allow(clippy::needless_pass_by_value)] // https://github.com/rust-lang/rust-clippy/issues/3067
#[cfg(feature = "project_attr")]
#[proc_macro_attribute]
pub fn project(args: TokenStream, input: TokenStream) -> TokenStream {
    assert!(args.is_empty());
    TokenStream::from(project::project(input.into()))
}
