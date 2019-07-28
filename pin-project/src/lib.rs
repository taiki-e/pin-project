//! An attribute that creates a projection struct covering all the fields.
//!
//! ## Examples
//!
//! [`pin_projectable`] attribute creates a projection struct covering all the fields.
//!
//! ```rust
//! use pin_project::pin_projectable;
//! use std::pin::Pin;
//!
//! #[pin_projectable]
//! struct Foo<T, U> {
//!     #[pin]
//!     future: T,
//!     field: U,
//! }
//!
//! impl<T, U> Foo<T, U> {
//!     fn baz(self: Pin<&mut Self>) {
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
//! [`pin_projectable`] also supports enums, but to use it ergonomically, you need
//! to use the [`project`] attribute.
//!
//! ```rust
//! # #[cfg(feature = "project_attr")]
//! use pin_project::{project, pin_projectable};
//! # #[cfg(feature = "project_attr")]
//! use std::pin::Pin;
//!
//! # #[cfg(feature = "project_attr")]
//! #[pin_projectable]
//! enum Foo<T, U> {
//!     Future(#[pin] T),
//!     Done(U),
//! }
//!
//! # #[cfg(feature = "project_attr")]
//! impl<T, U> Foo<T, U> {
//!     #[project] // Nightly does not need a dummy attribute to the function.
//!     fn baz(self: Pin<&mut Self>) {
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
//! See [`pin_projectable`] and [`project`] for more details.
//!
//! [`pin_projectable`]: ./attr.pin_projectable.html
//! [`project`]: ./attr.project.html
//!
#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project/0.3.4")]
#![doc(test(attr(deny(warnings), allow(dead_code, unused_assignments, unused_variables))))]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]
#![warn(single_use_lifetimes)]
#![warn(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]

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
/// use pin_project::{pin_projectable, project};
/// # use std::pin::Pin;
///
/// #[pin_projectable]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     #[project] // Nightly does not need a dummy attribute to the function.
///     fn baz(self: Pin<&mut Self>) {
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
/// use pin_project::{project, pin_projectable};
/// # use std::pin::Pin;
///
/// #[pin_projectable]
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
/// use pin_project::{project, pin_projectable};
/// # use std::pin::Pin;
///
/// #[pin_projectable]
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
#[doc(inline)]
pub use pin_project_internal::project;

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
/// This attribute is completely safe. In the absence of other `unsafe` code *that you write*,
/// it is impossible to cause undefined behavior with this attribute.
///
/// This is accomplished by enforcing the four requirements for pin projection
/// stated in [the Rust documentation](https://doc.rust-lang.org/beta/std/pin/index.html#projections-and-structural-pinning):
///
/// 1. The struct must only be Unpin if all the structural fields are Unpin
///
///	   To enforce this, this attribute will automatically generate an `Unpin` implementation
///    for you, which will require that all structurally pinned fields be `Unpin`
///    If you wish to provide an manual `Unpin` impl, you can do so via the
///    `unsafe_Unpin` argument.
///
/// 2. The destructor of the struct must not move structural fields out of its argument.
///
///    To enforce this, this attribute will automatically generate a `Drop` impl.
///    If you wish to provide a custom `Drop` impl, you can annotate a function
///    with `#[pinned_drop]`. This function takes a pinned version of your struct -
///    that is, `Pin<&mut MyStruct>` where `MyStruct` is the type of your struct.
///
///    You can call `project()` on this type as usual, along with any other
///    methods you have defined. Because your code is never provided with
///    a `&mut MyStruct`, it is impossible to move out of pin-projectable
///    fields in safe code in your destructor.
///
/// 3. You must make sure that you uphold the Drop guarantee: once your struct is pinned,
///    the memory that contains the content is not overwritten or deallocated without calling the content's destructors
///
///    Safe code doesn't need to worry about this - the only wait to violate this requirement
///    is to manually deallocate memory (which is `unsafe`), or to overwite a field with something else.
///    Becauese your custom destructor takes `Pin<&mut MyStruct`, it's impossible to obtain
///    a mutable reference to a pin-projected field in safe code.
///
/// 4. You must not offer any other operations that could lead to data being moved out of the structural fields when your type is pinned.
///
///    As with requirement 3, it is impossible for safe code to violate this. This crate ensures that safe code can never
///    obtain a mutable reference to `#[pin]` fields, which prevents you from ever moving out of them in safe code.
///
/// Pin projections are also incompatible with `#[repr(packed)]` structs. Attempting to use this attribute
/// on a `#[repr(packed)]` struct results in a compile-time error.
///
///
/// ## Examples
///
/// Using `#[pin_projectable]` will automatically create the appropriate
/// conditional [`Unpin`] implementation:
///
/// ```rust
/// use pin_project::pin_projectable;
/// use std::pin::Pin;
///
/// #[pin_projectable]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     fn baz(self: Pin<&mut Self>) {
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
/// If you want to implement [`Unpin`] manually,
/// you msut use thw `unsafe_Unpin` argument to
/// `#[pin_projectable]`.
///
/// ```rust
/// use pin_project::{pin_projectable, UnsafeUnpin};
/// use std::pin::Pin;
///
/// #[pin_projectable(unsafe_Unpin)]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     fn baz(self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.future; // Pinned reference to the field
///         let _: &mut U = this.field; // Normal reference to the field
///     }
/// }
///
/// unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {} // Conditional Unpin impl
/// ```
///
/// Note the usage of the unsafe `UnsafeUnpin` trait, instead of the usual
/// `Unpin` trait. `UnsafeUnpin` behaves exactly like `Unpin`, except that is
/// unsafe to implement. This unsafety comes from the fact that pin projections
/// are being used. If you implement `UnsafeUnpin`, you must ensure that it is
/// only implemented when all pin-projected fields implement `Unpin`.
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
/// # use pin_project::pin_projectable;
/// # use std::pin::Pin;
/// #[pin_projectable]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// impl<T, U> Foo<T, U> {
///     fn baz(self: Pin<&mut Self>) {
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
/// # use pin_project::pin_projectable;
/// # use std::pin::Pin;
/// #[pin_projectable]
/// struct Foo<T, U>(#[pin] T, U);
///
/// impl<T, U> Foo<T, U> {
///     fn baz(self: Pin<&mut Self>) {
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
/// `pin_projectable` also supports enums, but to use it ergonomically, you need
/// to use the [`project`] attribute.
///
/// ```rust
/// # #[cfg(feature = "project_attr")]
/// use pin_project::{project, pin_projectable};
/// # #[cfg(feature = "project_attr")]
/// # use std::pin::Pin;
///
/// # #[cfg(feature = "project_attr")]
/// #[pin_projectable]
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
/// ### `#[pinned_drop]`
///
/// In order to correctly implement pin projections, a type's Drop impl must
/// not move out of any stucturally pinned fields. Unfortunately, `Drop::drop` takes
/// `&mut Self`, not `Pin<&mut Self>`.
///
/// To ensure that this requirement is upheld, the `pin_projectable` attribute will
/// provide a `Drop` impl for you. This `Drop` impl will delegate to a function
/// annotated with `#[pinned_drop]`, if present in the same `pin_project!` block.
/// This function acts just like a normal `drop` impl, except for the fact that it
/// takes `Pin<&mut Self>`. In particular, it will never be called more than once,
/// just like `Drop::drop`.
///
/// For example:
///
/// ```rust
/// use std::fmt::Debug;
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// pin_project! {
///     #[pin_projectable]
///     pub struct Foo<T: Debug, U: Debug> {
///         #[pin] pinned_field: T,
///         unpin_field: U
///     }
///
///     #[pinned_drop]
///     fn my_drop_fn<T: Debug, U: Debug>(foo: Pin<&mut Foo<T, U>>) {
///         let foo = foo.project();
///         println!("Dropping pinned field: {:?}", foo.pinned_field);
///         println!("Dropping unpin field: {:?}", foo.unpin_field);
///     }
/// }
///
/// fn main() {
///     Foo { pinned_field: true, unpin_field: 40 };
/// }
/// ```
///
///
///
/// Also see [`project`] attribute.
///
/// [`Unpin`]: core::marker::Unpin
/// [`drop`]: Drop::drop
/// [`project`]: ./attr.project.html
#[doc(inline)]
pub use pin_project_internal::pin_projectable;

/// A helper macro for working with `pin_projectable`.
///
/// This macro is only needed when you wish to provide a `Drop`
/// impl for your type. You may include a single `#[pin_projectable]`
/// type, and (optionally) one `#[pinned_drop]` function. Writing
/// anything else within the `pin_projectable` block is an error.
///
/// Example:
///
/// ```rust
///
/// use std::pin::Pin;
/// use pin_project::pin_project;
///
/// pin_project! {
///
///     #[pin_projectable]
///     struct Foo {
///         #[pin] field: u8
///     }
///
///     #[pinned_drop]
///     fn my_drop(foo: Pin<&mut Foo>) {
///         println!("Dropping: {}", foo.field);
///     }
/// }
///
/// fn main() {
///     Foo { field: 50 };
/// }
///```
#[doc(inline)]
pub use pin_project_internal::pin_project;

/// A trait used for custom implementations of [`Unpin`].
/// This trait is used in conjunction with the `unsafe_Unpin`
/// argument to [`pin_projectable`]
///
/// The Rust [`Unpin`] trait is safe to implement - by itself,
/// implementing it cannot lead to undefined behavior. Undefined
/// behavior can only occur when other unsafe code is used.
///
/// It turns out that using pin projections, which requires unsafe code,
/// imposes additional requirements on an `Unpin` impl. Normally, all of this
/// unsafety is contained within this crate, ensuring that it's impossible for
/// you to violate any of the guarnatees required by pin projection.
///
/// However, things change if you want to provide a custom `Unpin` impl
/// for your `#[pin_projectable]` type. As stated in [the Rust
/// documentation](https://doc.rust-lang.org/beta/std/pin/index.html#projections-and-structural-pinning),
/// you must be sure to only implement `Unpin` when all of your `#[pin]` fields (i.e. struturally
/// pinend fields) are also `Unpin`.
///
/// To help highlight this unsafety, the `UnsafeUnpin` trait is provided.
/// Implementing this trait is logically equivalent to implemnting `Unpin` -
/// this crate will generate an `Unpin` impl for your type that 'forwards' to
/// your `UnsafeUnpin` impl. However, this trait is `unsafe` - since your type
/// uses structural pinning (otherwise, you wouldn't be using this crate!),
/// you must be sure that your `UnsafeUnpinned` impls follows all of
/// the requirements for an `Unpin` impl of a structurally-pinned type.
///
/// Note that if you specify `#[pin_projectable(unsafe_Unpin)]`, but do *not*
/// provide an impl of `UnsafeUnpin`, your type will never implement `Unpin`.
/// This is effectly the same thing as adding a `PhantomUnpin` to your type
///
/// Since this trait is `unsafe`, impls of it will be detected by the `unsafe_code` lint,
/// and by tools like `cargo geiger`.
///
/// ## Examples
///
/// An `UnsafeUnpin` impl which, in addition to requiring that structually pinned
/// fields be `Unpin`, imposes an additional requirement:
///
/// ```rust
/// use pin_project::{pin_projectable, UnsafeUnpin};
///
/// #[pin_projectable(unsafe_Unpin)]
/// struct Foo<K, V> {
///     #[pin]
///     field_1: K,
///     field_2: V
/// }
///
/// unsafe impl<K, V> UnsafeUnpin for Foo<K, V> where K: Unpin + Clone {}
/// ```
///
/// [`pin_projectable`]: ./attr.pin_projectable.html
#[allow(unsafe_code)]
pub unsafe trait UnsafeUnpin {}

#[doc(hidden)]
// This is an internal helper struct used by `pin-project-internal`.
// This allows us to force an error if the user tries to provide
// a regular `Unpin` impl when they specify the `unsafe_Unpin` argument
// This is why we need Wrapper:
//
// Supposed we have the following code:
//
// #[pin_projectable(unsafe_Unpin)]
// struct MyStruct<T> {
//     #[pin] field: T
// }
//
// impl<T> Unpin for MyStruct<T> where MyStruct<T>: UnsafeUnpinned {} // generated by pin-project-internal
// impl<T> Unpin for MyStruct<T> where T: Copy // written by the user
//
// We want this code to be rejected - the user is completely bypassing unsafe_Unpin,
// and providing an unsound Unpin impl in safe code!
//
// Unfortunately, the Rust compiler will accept the above code.
// Because MyStruct is declared in the same crate as the user-provided impl,
// the compiler will notice that 'MyStruct<T>: UnsafeUnpinned' never holds.
//
// The solution is to introduce the 'Wrapper' struct, which is defined
// in the 'pin-project' crate.
//
// We now have code that looks like this:
//
// impl<T> Unpin for MyStruct<T> where Wrapper<MyStruct<T>>: UnsafeUnpinned {} // generated by pin-project-internal
// impl<T> Unpin for MyStruct<T> where T: Copy // written by the user
//
// We also have 'unsafe impl<T> UnsafeUnpin for Wrapper<T> where T: UnsafeUnpin {}' in the
// 'pin-project' crate.
//
// Now, our generated impl has a bound involving a type defined in another crate - Wrapper.
// This will cause rust to conservatively assume that 'Wrapper<MyStruct<T>>: UnsafeUnpinned'
// holds, in the interest of preserving forwards compatibility (in case such an impl is added
// for Wrapper<T> in a new version of the crate).
//
// This will cause rust to reject any other Unpin impls for MyStruct<T>, since it will
// assume that our generated impl could potentially apply in any situation.
//
// This acheives the desired effect - when the user writes `#[pin_projectable(unsafe_Unpin)]`,
// the user must either provide no impl of `UnsafeUnpinned` (which is equivalent
// to making the type never implement Unpin), or provide an impl of `UnsafeUnpin`.
// It is impossible for them to provide an impl of `Unpin`
pub struct Wrapper<T>(T);

#[allow(unsafe_code)]
unsafe impl<T> UnsafeUnpin for Wrapper<T> where T: UnsafeUnpin {}
