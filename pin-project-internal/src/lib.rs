//! An interal crate to support pin_project - **do not use directly**

#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project-internal/0.4.0-alpha.11")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub, single_use_lifetimes)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::use_self)]
#![cfg_attr(proc_macro_def_site, feature(proc_macro_def_site))]

extern crate proc_macro;

#[macro_use]
mod utils;

mod pin_project;
mod pinned_drop;
#[cfg(feature = "project_attr")]
mod project;

use proc_macro::TokenStream;
use syn::parse::Nothing;

// TODO: Move this doc into pin-project crate when https://github.com/rust-lang/rust/pull/62855 merged.
/// An attribute that creates a projection struct covering all the fields.
///
/// This attribute creates a projection struct according to the following rules:
///
/// - For the field that uses `#[pin]` attribute, makes the pinned reference to
/// the field.
/// - For the other fields, makes the unpinned reference to the field.
///
/// The following methods are implemented on the original `#[pin_project]` type:
///
/// ```
/// # #![feature(arbitrary_self_types)]
/// # use std::pin::Pin;
/// # type ProjectedType = ();
/// # trait ProjectionTrait {
/// fn project(self: &mut Pin<&mut Self>) -> ProjectedType;
/// fn project_into(self: Pin<&mut Self>) -> ProjectedType;
/// # }
/// ```
///
/// The `project` method takes a mutable reference to a pinned
/// type, and returns a projection struct. This is the method
/// you'll usually want to use - since it takes a mutable reference,
/// it can be called multiple times, and allows you to use
/// the original Pin type later on (e.g. to call [`Pin::set`]).
///
/// The `project_into` type takes a pinned type by value (consuming it),
/// and returns a projection struct. The difference between this and the `project`
/// method lies in the lifetime. While the type returned by `project` only lives
/// as long as the 'outer' mutable reference, the type returned by this method
/// lives for as long as the original Pin. This can be useful when returning a pin
/// projection from a method:
///
/// ```
/// # use pin_project::pin_project;
/// # use std::pin::Pin;
/// # #[pin_project]
/// # struct Struct<T> {
/// #     #[pin]
/// #     pinned: T,
/// # }
/// # impl<T> Struct<T> {
/// fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
///     self.project_into().pinned
/// }
/// # }
/// ```
///
/// ## Safety
///
/// This attribute is completely safe. In the absence of other `unsafe` code *that you write*,
/// it is impossible to cause undefined behavior with this attribute.
///
/// This is accomplished by enforcing the four requirements for pin projection
/// stated in [the Rust documentation](https://doc.rust-lang.org/beta/std/pin/index.html#projections-and-structural-pinning):
///
/// 1. The struct must only be Unpin if all the structural fields are Unpin.
///
///	   To enforce this, this attribute will automatically generate an `Unpin` implementation
///    for you, which will require that all structurally pinned fields be `Unpin`
///    If you wish to provide an manual `Unpin` impl, you can do so via the
///    `UnsafeUnpin` argument.
///
/// 2. The destructor of the struct must not move structural fields out of its argument.
///
///    To enforce this, this attribute will generate code like this:
///
///    ```rust
///    struct MyStruct {}
///    trait MyStructMustNotImplDrop {}
///    impl<T: Drop> MyStructMustNotImplDrop for T {}
///    impl MyStructMustNotImplDrop for MyStruct {}
///    ```
///
///    If you attempt to provide an Drop impl, the blanket impl will
///    then apply to your type, causing a compile-time error due to
///    the conflict with the second impl.
///
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
///    the memory that contains the content is not overwritten or deallocated without calling the content's destructors.
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
/// Using `#[pin_project]` will automatically create the appropriate
/// conditional [`Unpin`] implementation:
///
/// ```rust
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project]
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
/// ```
///
/// If you want to implement [`Unpin`] manually, you must use the `UnsafeUnpin`
/// argument to `#[pin_project]`.
///
/// ```rust
/// use pin_project::{pin_project, UnsafeUnpin};
/// use std::pin::Pin;
///
/// #[pin_project(UnsafeUnpin)]
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
/// unsafe impl<T: Unpin, U> UnsafeUnpin for Foo<T, U> {} // Conditional Unpin impl
/// ```
///
/// Note the usage of the unsafe [`UnsafeUnpin`] trait, instead of the usual
/// [`Unpin`] trait. [`UnsafeUnpin`] behaves exactly like [`Unpin`], except that is
/// unsafe to implement. This unsafety comes from the fact that pin projections
/// are being used. If you implement [`UnsafeUnpin`], you must ensure that it is
/// only implemented when all pin-projected fields implement [`Unpin`].
///
/// Note that borrowing the field where `#[pin]` attribute is used multiple
/// times requires using [`.as_mut()`][`Pin::as_mut`] to avoid
/// consuming the `Pin`.
///
/// See also [`UnsafeUnpin`] trait.
///
/// ### `#[pinned_drop]`
///
/// In order to correctly implement pin projections, a type's `Drop` impl must
/// not move out of any stucturally pinned fields. Unfortunately, [`Drop::drop`]
/// takes `&mut Self`, not `Pin<&mut Self>`.
///
/// To ensure that this requirement is upheld, the `#[pin_project]` attribute will
/// provide a [`Drop`] impl for you. This `Drop` impl will delegate to an impl
/// block annotated with `#[pinned_drop]` if you use the `PinnedDrop` argument
/// to `#[pin_project]`. This impl block acts just like a normal [`Drop`] impl,
/// except for the following two:
///
/// * `drop` method takes `Pin<&mut Self>`
/// * Name of the trait is `PinnedDrop`.
///
/// `#[pin_project]` implements the actual [`Drop`] trait via `PinnedDrop` you
/// implemented. To drop a type that implements `PinnedDrop`, use the [`drop`]
/// function just like dropping a type that directly implements [`Drop`].
///
/// In particular, it will never be called more than once, just like [`Drop::drop`].
///
/// For example:
///
/// ```rust
/// use pin_project::{pin_project, pinned_drop};
/// use std::fmt::Debug;
/// use std::pin::Pin;
///
/// #[pin_project(PinnedDrop)]
/// pub struct Foo<T: Debug, U: Debug> {
///     #[pin] pinned_field: T,
///     unpin_field: U
/// }
///
/// #[pinned_drop]
/// impl<T: Debug, U: Debug> PinnedDrop for Foo<T, U> {
///     fn drop(self: Pin<&mut Self>) {
///         println!("Dropping pinned field: {:?}", self.pinned_field);
///         println!("Dropping unpin field: {:?}", self.unpin_field);
///     }
/// }
///
/// fn main() {
///     Foo { pinned_field: true, unpin_field: 40 };
/// }
/// ```
///
/// See also [`pinned_drop`] attribute.
///
/// ## Supported Items
///
/// The current pin-project supports the following types of items.
///
/// ### Structs (structs with named fields):
///
/// ```rust
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project]
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
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project]
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
/// `pin_project` also supports enums, but to use it ergonomically, you need
/// to use the [`project`] attribute.
///
/// *This attribute is only available if pin-project is built
/// with the `"project_attr"` feature.*
///
/// The attribute at the expression position is not stable, so you need to use
/// a dummy `#[project]` attribute for the function.
///
/// ```rust
/// # #[cfg(feature = "project_attr")]
/// use pin_project::{project, pin_project};
/// # #[cfg(feature = "project_attr")]
/// use std::pin::Pin;
///
/// # #[cfg(feature = "project_attr")]
/// #[pin_project]
/// enum Foo<A, B, C> {
///     Tuple(#[pin] A, B),
///     Struct { field: C },
///     Unit,
/// }
///
/// # #[cfg(feature = "project_attr")]
/// impl<A, B, C> Foo<A, B, C> {
///     #[project] // Nightly does not need a dummy attribute to the function.
///     fn baz(mut self: Pin<&mut Self>) {
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
/// See also [`project`] attribute.
///
/// [`Pin::as_mut`]: core::pin::Pin::as_mut
/// [`Pin::set`]: core::pin::Pin::set
/// [`drop`]: Drop::drop
/// [`UnsafeUnpin`]: https://docs.rs/pin-project/0.4.0-alpha.11/pin_project/trait.UnsafeUnpin.html
/// [`project`]: ./attr.project.html
/// [`pinned_drop`]: ./attr.pinned_drop.html
#[proc_macro_attribute]
pub fn pin_project(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    pin_project::attribute(args.into(), input).into()
}

// TODO: Move this doc into pin-project crate when https://github.com/rust-lang/rust/pull/62855 merged.
/// An attribute for annotating an impl block that implements [`Drop`].
///
/// This attribute is only needed when you wish to provide a [`Drop`]
/// impl for your type. The impl block annotated with `#[pinned_drop]` acts just
/// like a normal [`Drop`] impl, except for the fact that `drop` method takes
/// `Pin<&mut Self>`. In particular, it will never be called more than once,
/// just like [`Drop::drop`].
///
/// ## Example
///
/// ```rust
/// use pin_project::{pin_project, pinned_drop};
/// use std::pin::Pin;
///
/// #[pin_project(PinnedDrop)]
/// struct Foo {
///     #[pin] field: u8
/// }
///
/// #[pinned_drop]
/// impl PinnedDrop for Foo {
///     fn drop(self: Pin<&mut Self>) {
///         println!("Dropping: {}", self.field);
///     }
/// }
///
/// fn main() {
///     Foo { field: 50 };
/// }
/// ```
///
/// See ["pinned-drop" section of `pin_project` attribute][pinned-drop] for more details.
///
/// [pinned-drop]: ./attr.pin_project.html#pinned_drop
#[proc_macro_attribute]
pub fn pinned_drop(args: TokenStream, input: TokenStream) -> TokenStream {
    let _: Nothing = syn::parse_macro_input!(args);
    let input = syn::parse_macro_input!(input);
    pinned_drop::attribute(input).into()
}

// TODO: Move this doc into pin-project crate when https://github.com/rust-lang/rust/pull/62855 merged.
/// An attribute to provide way to refer to the projected type.
///
/// *This attribute is available if pin-project is built with the
/// `"project_attr"` feature.*
///
/// The following three syntaxes are supported.
///
/// ## `impl` blocks
///
/// All methods (and associated functions) in `#[project] impl` block become
/// methods of the projected type. If you want to implement methods on the
/// original type, you need to create another (non-`#[project]`) `impl` block.
///
/// To call a method implemented in `#[project] impl` block, you need to first
/// get the projected-type with `let this = self.project();`.
///
/// ### Examples
///
/// ```rust
/// use pin_project::{pin_project, project};
/// use std::pin::Pin;
///
/// #[pin_project]
/// struct Foo<T, U> {
///     #[pin]
///     future: T,
///     field: U,
/// }
///
/// // impl for the original type
/// impl<T, U> Foo<T, U> {
///     fn bar(mut self: Pin<&mut Self>) {
///         self.project().baz()
///     }
/// }
///
/// // impl for the projected type
/// #[project]
/// impl<T, U> Foo<T, U> {
///     fn baz(self) {
///         let Self { future, field } = self;
///
///         let _: Pin<&mut T> = future;
///         let _: &mut U = field;
///     }
/// }
/// ```
///
/// ## `let` bindings
///
/// *The attribute at the expression position is not stable, so you need to use
/// a dummy `#[project]` attribute for the function.*
///
/// ### Examples
///
/// ```rust
/// use pin_project::{pin_project, project};
/// use std::pin::Pin;
///
/// #[pin_project]
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
/// ## `match` expressions
///
/// *The attribute at the expression position is not stable, so you need to use
/// a dummy `#[project]` attribute for the function.*
///
/// ### Examples
///
/// ```rust
/// use pin_project::{project, pin_project};
/// use std::pin::Pin;
///
/// #[pin_project]
/// enum Foo<A, B, C> {
///     Tuple(#[pin] A, B),
///     Struct { field: C },
///     Unit,
/// }
///
/// impl<A, B, C> Foo<A, B, C> {
///     #[project] // Nightly does not need a dummy attribute to the function.
///     fn baz(mut self: Pin<&mut Self>) {
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
#[cfg(feature = "project_attr")]
#[proc_macro_attribute]
pub fn project(args: TokenStream, input: TokenStream) -> TokenStream {
    let _: Nothing = syn::parse_macro_input!(args);
    let input = syn::parse_macro_input!(input);
    project::attribute(input).into()
}

#[doc(hidden)]
#[proc_macro_derive(__PinProjectAutoImplUnpin, attributes(pin))]
pub fn derive_unpin(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    pin_project::derive(input).into()
}

#[cfg(feature = "renamed")]
lazy_static::lazy_static! {
    pub(crate) static ref PIN_PROJECT_CRATE: String = {
        proc_macro_crate::crate_name("pin-project")
            .expect("pin-project-internal was used without pin-project!")
    };
}
