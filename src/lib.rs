//! An attribute that would create a projection struct covering all the fields.
//!
//! ## Examples
//!
//! For structs:
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
//! // Automatically create the appropriate conditional Unpin implementation.
//! // impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl
//! ```
//!
//! For enums:
//!
//! ```rust
//! # #[cfg(feature = "project_attr")]
//! use pin_project::{project, unsafe_project};
//! # #[cfg(feature = "project_attr")]
//! use std::pin::Pin;
//!
//! # #[cfg(feature = "project_attr")]
//! #[unsafe_project(Unpin)]
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
//! // Automatically create the appropriate conditional Unpin implementation.
//! // impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
//! ```
//!
//! See [`unsafe_project`] and [`project`] for more details.
//!
//! [`unsafe_project`]: ./attr.unsafe_project.html
//! [`project`]: ./attr.project.html
//!

#![crate_type = "proc-macro"]
#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project/0.1.8")]

extern crate proc_macro;

mod enums;
mod structs;
mod utils;

#[cfg(feature = "project_attr")]
mod macros;

#[cfg(feature = "unsafe_fields")]
mod fields;
#[cfg(feature = "unsafe_variants")]
mod variants;

mod compile_fail;

use proc_macro::TokenStream;
use syn::Item;

/// An attribute that would create a projection struct covering all the fields.
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
/// // impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
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
/// [`Unpin`]: core::marker::Unpin
/// [`drop`]: Drop::drop
#[proc_macro_attribute]
pub fn unsafe_project(args: TokenStream, input: TokenStream) -> TokenStream {
    match syn::parse(input) {
        Ok(Item::Struct(item)) => structs::unsafe_project(args, item),
        Ok(Item::Enum(item)) => enums::unsafe_project(args, item),
        _ => utils::compile_err("`unsafe_project` may only be used on structs or enums"),
    }
}

/// An attribute that would create projections for each struct fields.
///
/// This is similar to [`unsafe_project`], but it is compatible with
/// [pin-utils].
///
/// *This attribute is available if pin-project is built with the
/// "unsafe_fields" feature.*
///
/// This attribute creates methods according to the following rules:
///
/// - For the field that uses `#[pin]` attribute, the method that makes the
///   pinned reference to that field is created. This is the same as
///   [`pin_utils::unsafe_pinned`].
/// - For the field that uses `#[skip]` attribute, the method referencing that
///   field is not created.
/// - For the other fields, the method that makes the unpinned reference to that
///   field is created. This is the same as [`pin_utils::unsafe_unpinned`].
///
/// ## Safety
///
/// For the field that uses `#[pin]` attribute, three things need to be ensured:
///
/// - If the struct implements [`Drop`], the [`drop`] method is not allowed to
///   move the value of the field.
/// - If the struct wants to implement [`Unpin`], it has to do so conditionally:
///   The struct can only implement [`Unpin`] if the field's type is [`Unpin`].
///   If you use `#[unsafe_fields(Unpin)]`, you do not need to ensure this
///   because an appropriate [`Unpin`] implementation will be generated.
/// - The struct must not be `#[repr(packed)]`.
///
/// For the other fields, need to be ensured that the contained value not pinned
/// in the current context.
///
/// ## Examples
///
/// Using `#[unsafe_fields(Unpin)]` will automatically create the appropriate
/// [`Unpin`] implementation:
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
/// // Automatically create the appropriate conditional Unpin implementation.
/// // impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
/// ```
///
/// If you want to implement [`Unpin`] manually:
///
/// ```rust
/// use pin_project::unsafe_fields;
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
#[cfg(feature = "unsafe_fields")]
#[proc_macro_attribute]
pub fn unsafe_fields(args: TokenStream, input: TokenStream) -> TokenStream {
    fields::unsafe_fields(args, input)
}

/// An attribute that would create projections for each enum variants.
///
/// *This attribute is available if pin-project is built with the
/// "unsafe_variants" feature.*
///
/// This attribute creates methods according to the following rules:
///
/// - For the field that uses `#[pin]` attribute, the method that makes the
///   pinned reference to that field is created.
/// - For the variant or field that uses `#[skip]` attribute, the method referencing that
///   variant or field is not created.
/// - For the unit variant, the method referencing that variant is not created.
/// - For the other fields, the method that makes the unpinned reference to that
///   field is created.
///
/// ## Safety
///
/// For the field that uses `#[pin]` attribute, three things need to be ensured:
///
/// - If the enum implements [`Drop`], the [`drop`] method is not allowed to
///   move the value of the field.
/// - If the enum wants to implement [`Unpin`], it has to do so conditionally:
///   The enum can only implement [`Unpin`] if the field's type is [`Unpin`].
///   If you use `#[unsafe_variants(Unpin)]`, you do not need to ensure this
///   because an appropriate [`Unpin`] implementation will be generated.
/// - The enum must not be `#[repr(packed)]`.
///
/// For the other fields, need to be ensured that the contained value not pinned
/// in the current context.
///
/// ## Examples
///
/// Using `#[unsafe_variants(Unpin)]` will automatically create the appropriate
/// [`Unpin`] implementation:
///
/// ```rust
/// use pin_project::unsafe_variants;
/// use std::pin::Pin;
///
/// #[unsafe_variants(Unpin)]
/// enum Foo<A, B, C> {
///     Variant1(#[pin] A, B),
///     Variant2(C),
/// }
///
/// impl<A, B, C> Foo<A, B, C> {
///     fn baz(mut self: Pin<&mut Self>) {
///         let _: Pin<&mut A> = self.as_mut().variant1().unwrap().0; // Pinned reference to the field
///         let _: &mut B = self.as_mut().variant1().unwrap().1; // Normal reference to the field
///         let _: Option<&mut C> = self.as_mut().variant2();
///     }
/// }
///
/// // Automatically create the appropriate conditional Unpin implementation.
/// // impl<A, B, C> Unpin for Foo<A, B, C> where A: Unpin {} // Conditional Unpin impl
/// ```
///
/// If you want to implement [`Unpin`] manually:
///
/// ```rust
/// use pin_project::unsafe_variants;
/// use std::pin::Pin;
///
/// #[unsafe_variants]
/// enum Foo<A, B, C> {
///     Variant1(#[pin] A, B),
///     Variant2(C),
/// }
///
/// impl<A, B, C> Foo<A, B, C> {
///     fn baz(mut self: Pin<&mut Self>) {
///         let _: Pin<&mut A> = self.as_mut().variant1().unwrap().0; // Pinned reference to the field
///         let _: &mut B = self.as_mut().variant1().unwrap().1; // Normal reference to the field
///         let _: Option<&mut C> = self.as_mut().variant2();
///     }
/// }
///
/// impl<A, B, C> Unpin for Foo<A, B, C> where A: Unpin {} // Conditional Unpin impl
/// ```
///
/// Note that borrowing the variant multiple times requires using `.as_mut()` to
/// avoid consuming the `Pin`.
///
/// [`Unpin`]: core::marker::Unpin
/// [`drop`]: Drop::drop
#[cfg(feature = "unsafe_variants")]
#[proc_macro_attribute]
pub fn unsafe_variants(args: TokenStream, input: TokenStream) -> TokenStream {
    variants::unsafe_variants(args, input)
}

/// An attribute to support pattern matching.
///
/// *This attribute is available if pin-project is built with the
/// "project_attr" feature (it is enabled by default).*
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
#[cfg(feature = "project_attr")]
#[proc_macro_attribute]
pub fn project(args: TokenStream, input: TokenStream) -> TokenStream {
    assert!(args.is_empty());
    macros::project(input)
}
