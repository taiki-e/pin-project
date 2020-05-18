//! An internal crate to support pin_project - **do not use directly**

#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project-internal/0.4.17")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes, unreachable_pub)]
#![warn(clippy::all, clippy::default_trait_access)]
// mem::take and #[non_exhaustive] requires Rust 1.40
#![allow(clippy::mem_replace_with_default, clippy::manual_non_exhaustive)]
#![allow(clippy::needless_doctest_main)]

// older compilers require explicit `extern crate`.
#[allow(unused_extern_crates)]
extern crate proc_macro;

#[macro_use]
mod utils;

mod pin_project;
mod pinned_drop;
mod project;

use proc_macro::TokenStream;

use crate::utils::{Immutable, Mutable, Owned};

/// An attribute that creates a projection type covering all the fields of struct or enum.
///
/// This attribute creates a projection type according to the following rules:
///
/// * For the field that uses `#[pin]` attribute, makes the pinned reference to the field.
/// * For the other fields, makes the unpinned reference to the field.
///
/// And the following methods are implemented on the original `#[pin_project]` type:
///
/// ```
/// # #[rustversion::since(1.36)]
/// # fn dox() {
/// # use std::pin::Pin;
/// # type Projection<'a> = &'a ();
/// # type ProjectionRef<'a> = &'a ();
/// # trait Dox {
/// fn project(self: Pin<&mut Self>) -> Projection<'_>;
/// fn project_ref(self: Pin<&Self>) -> ProjectionRef<'_>;
/// # }
/// # }
/// ```
///
/// By passing an argument with the same name as the method to the attribute,
/// you can name the projection type returned from the method:
///
/// ```rust
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project(project = EnumProj)]
/// enum Enum<T> {
///     Variant(#[pin] T),
/// }
///
/// fn func<T>(x: Pin<&mut Enum<T>>) {
///     match x.project() {
///         EnumProj::Variant(y) => {
///             let _: Pin<&mut T> = y;
///         }
///     }
/// }
/// ```
///
/// The visibility of the projected type and projection method is based on the original type.
/// However, if the visibility of the original type is `pub`, the visibility of the projected type
/// and the projection method is downgraded to `pub(crate)`.
///
/// # Safety
///
/// This attribute is completely safe. In the absence of other `unsafe` code *that you write*,
/// it is impossible to cause [undefined behavior][undefined-behavior] with this attribute.
///
/// This is accomplished by enforcing the four requirements for pin projection
/// stated in [the Rust documentation][pin-projection]:
///
/// 1. The struct must only be [`Unpin`] if all the structural fields are [`Unpin`].
///
///    To enforce this, this attribute will automatically generate an [`Unpin`] implementation
///    for you, which will require that all structurally pinned fields be [`Unpin`]
///    If you wish to provide an manual [`Unpin`] impl, you can do so via the
///    [`UnsafeUnpin`][unsafe-unpin] argument.
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
///    If you attempt to provide an [`Drop`] impl, the blanket impl will
///    then apply to your type, causing a compile-time error due to
///    the conflict with the second impl.
///
///    If you wish to provide a custom [`Drop`] impl, you can annotate an impl
///    with [`#[pinned_drop]`][pinned-drop]. This impl takes a pinned version of your struct -
///    that is, [`Pin`]`<&mut MyStruct>` where `MyStruct` is the type of your struct.
///
///    You can call `project()` on this type as usual, along with any other
///    methods you have defined. Because your code is never provided with
///    a `&mut MyStruct`, it is impossible to move out of pin-projectable
///    fields in safe code in your destructor.
///
/// 3. You must make sure that you uphold the [`Drop` guarantee][drop-guarantee]: once your struct is pinned,
///    the memory that contains the content is not overwritten or deallocated without calling the content's destructors.
///
///    Safe code doesn't need to worry about this - the only wait to violate this requirement
///    is to manually deallocate memory (which is `unsafe`), or to overwrite a field with something else.
///    Because your custom destructor takes [`Pin`]`<&mut MyStruct>`, it's impossible to obtain
///    a mutable reference to a pin-projected field in safe code.
///
/// 4. You must not offer any other operations that could lead to data being moved out of the structural fields when your type is pinned.
///
///    As with requirement 3, it is impossible for safe code to violate this. This crate ensures that safe code can never
///    obtain a mutable reference to `#[pin]` fields, which prevents you from ever moving out of them in safe code.
///
/// Pin projections are also incompatible with [`#[repr(packed)]`][repr-packed] structs. Attempting to use this attribute
/// on a [`#[repr(packed)]`][repr-packed] struct results in a compile-time error.
///
/// # Examples
///
/// Using `#[pin_project]` will automatically create the appropriate
/// conditional [`Unpin`] implementation:
///
/// ```rust
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project]
/// struct Struct<T, U> {
///     #[pin]
///     pinned: T,
///     unpinned: U,
/// }
///
/// impl<T, U> Struct<T, U> {
///     fn method(self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.pinned; // Pinned reference to the field
///         let _: &mut U = this.unpinned; // Normal reference to the field
///     }
/// }
/// ```
///
/// If you want to call the `project()` method multiple times or later use the
/// original [`Pin`] type, it needs to use [`.as_mut()`][`Pin::as_mut`] to avoid
/// consuming the [`Pin`].
///
/// ## Supported Items
///
/// `#[pin_project]` can be used on structs and enums.
///
/// [Structs](https://doc.rust-lang.org/reference/items/structs.html):
///
/// ```rust
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project]
/// struct Struct<T, U> {
///     #[pin]
///     pinned: T,
///     unpinned: U,
/// }
///
/// impl<T, U> Struct<T, U> {
///     fn method(self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.pinned;
///         let _: &mut U = this.unpinned;
///     }
/// }
/// ```
///
/// [Tuple structs](https://doc.rust-lang.org/reference/items/structs.html):
///
/// ```rust
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project]
/// struct TupleStruct<T, U>(#[pin] T, U);
///
/// impl<T, U> TupleStruct<T, U> {
///     fn method(self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.0;
///         let _: &mut U = this.1;
///     }
/// }
/// ```
///
/// [Enums](https://doc.rust-lang.org/reference/items/enumerations.html):
///
/// `#[pin_project]` supports enums, but to use it, you need to name the
/// projection type returned from the method or to use with the [`project`] attribute.
///
/// ```rust
/// use pin_project::pin_project;
/// use std::pin::Pin;
///
/// #[pin_project(project = EnumProj)]
/// enum Enum<T, U> {
///     Tuple(#[pin] T),
///     Struct { field: U },
///     Unit,
/// }
///
/// impl<T, U> Enum<T, U> {
///     fn method(self: Pin<&mut Self>) {
///         match self.project() {
///             EnumProj::Tuple(x) => {
///                 let _: Pin<&mut T> = x;
///             }
///             EnumProj::Struct { field } => {
///                 let _: &mut U = field;
///             }
///             EnumProj::Unit => {}
///         }
///     }
/// }
/// ```
///
/// See also [`project`] and [`project_ref`] attributes.
///
/// ## `!Unpin`
///
/// If you want to ensure that [`Unpin`] is not implemented, use the `!Unpin`
/// argument to `#[pin_project]`.
///
/// ```rust
/// use pin_project::pin_project;
///
/// #[pin_project(!Unpin)]
/// struct Struct<T, U> {
///     #[pin]
///     pinned: T,
///     unpinned: U,
/// }
/// ```
///
/// You can also ensure `!Unpin` by using `#[pin]` attribute for [`PhantomPinned`] field.
///
/// ```rust
/// use pin_project::pin_project;
/// use std::marker::PhantomPinned;
///
/// #[pin_project]
/// struct Struct<T, U> {
///     #[pin]
///     pinned: T,
///     unpinned: U,
///     #[pin]
///     _pin: PhantomPinned,
/// }
/// ```
///
/// Note that using [`PhantomPinned`] without `#[pin]` attribute has no effect.
///
/// ## `UnsafeUnpin`
///
/// If you want to implement [`Unpin`] manually, you must use the `UnsafeUnpin`
/// argument to `#[pin_project]`.
///
/// ```rust
/// use pin_project::{pin_project, UnsafeUnpin};
///
/// #[pin_project(UnsafeUnpin)]
/// struct Struct<T, U> {
///     #[pin]
///     pinned: T,
///     unpinned: U,
/// }
///
/// unsafe impl<T: Unpin, U> UnsafeUnpin for Struct<T, U> {}
/// ```
///
/// Note the usage of the unsafe [`UnsafeUnpin`] trait, instead of the usual
/// [`Unpin`] trait. [`UnsafeUnpin`] behaves exactly like [`Unpin`], except that is
/// unsafe to implement. This unsafety comes from the fact that pin projections
/// are being used. If you implement [`UnsafeUnpin`], you must ensure that it is
/// only implemented when all pin-projected fields implement [`Unpin`].
///
/// See [`UnsafeUnpin`] trait for more details.
///
/// ## `#[pinned_drop]`
///
/// In order to correctly implement pin projections, a type's [`Drop`] impl must
/// not move out of any structurally pinned fields. Unfortunately, [`Drop::drop`]
/// takes `&mut Self`, not [`Pin`]`<&mut Self>`.
///
/// To ensure that this requirement is upheld, the `#[pin_project]` attribute will
/// provide a [`Drop`] impl for you. This [`Drop`] impl will delegate to an impl
/// block annotated with `#[pinned_drop]` if you use the `PinnedDrop` argument
/// to `#[pin_project]`.
///
/// This impl block acts just like a normal [`Drop`] impl,
/// except for the following two:
///
/// * `drop` method takes [`Pin`]`<&mut Self>`
/// * Name of the trait is `PinnedDrop`.
///
/// ```rust
/// # use std::pin::Pin;
/// pub trait PinnedDrop {
///     fn drop(self: Pin<&mut Self>);
/// }
/// ```
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
/// use std::{fmt::Debug, pin::Pin};
///
/// #[pin_project(PinnedDrop)]
/// struct Struct<T: Debug, U: Debug> {
///     #[pin]
///     pinned_field: T,
///     unpin_field: U,
/// }
///
/// #[pinned_drop]
/// impl<T: Debug, U: Debug> PinnedDrop for Struct<T, U> {
///     fn drop(self: Pin<&mut Self>) {
///         println!("Dropping pinned field: {:?}", self.pinned_field);
///         println!("Dropping unpin field: {:?}", self.unpin_field);
///     }
/// }
///
/// fn main() {
///     let _x = Struct { pinned_field: true, unpin_field: 40 };
/// }
/// ```
///
/// See also [`pinned_drop`] attribute.
///
/// ## `project_replace()`
///
/// In addition to the `project()` and `project_ref()` methods which are always
/// provided when you use the `#[pin_project]` attribute, there is a third method,
/// `project_replace()` which can be useful in some situations. It is equivalent
/// to [`Pin::set`], except that the unpinned fields are moved and returned,
/// instead of being dropped in-place.
///
/// ```
/// # #[rustversion::since(1.36)]
/// # fn dox() {
/// # use std::pin::Pin;
/// # type ProjectionOwned = ();
/// # trait Dox {
/// fn project_replace(self: Pin<&mut Self>, other: Self) -> ProjectionOwned;
/// # }
/// # }
/// ```
///
/// The `ProjectionOwned` type is identical to the `Self` type, except that
/// all pinned fields have been replaced by equivalent [`PhantomData`] types.
///
/// This method is opt-in, because it is only supported for [`Sized`] types, and
/// because it is incompatible with the [`#[pinned_drop]`][pinned-drop] attribute described
/// above. It can be enabled by using `#[pin_project(Replace)]`.
///
/// For example:
///
/// ```rust
/// use pin_project::{pin_project, project_replace};
///
/// #[pin_project(Replace)]
/// enum Struct<T> {
///     A {
///         #[pin]
///         pinned_field: i32,
///         unpinned_field: T,
///     },
///     B,
/// }
///
/// #[project_replace]
/// fn main() {
///     let mut x = Box::pin(Struct::A { pinned_field: 42, unpinned_field: "hello" });
///
///     #[project_replace]
///     match x.as_mut().project_replace(Struct::B) {
///         Struct::A { unpinned_field, .. } => assert_eq!(unpinned_field, "hello"),
///         Struct::B => unreachable!(),
///     }
/// }
/// ```
///
/// The [`project_replace`] attributes are necessary whenever destructuring the return
/// type of `project_replace()`, and work in exactly the same way as the
/// [`project`] and [`project_ref`] attributes.
///
/// [`PhantomData`]: core::marker::PhantomData
/// [`PhantomPinned`]: core::marker::PhantomPinned
/// [`Pin::as_mut`]: core::pin::Pin::as_mut
/// [`Pin::set`]: core::pin::Pin::set
/// [`Pin`]: core::pin::Pin
/// [`UnsafeUnpin`]: https://docs.rs/pin-project/0.4/pin_project/trait.UnsafeUnpin.html
/// [`pinned_drop`]: ./attr.pinned_drop.html
/// [`project_ref`]: ./attr.project_ref.html
/// [`project_replace`]: ./attr.project_replace.html
/// [`project`]: ./attr.project.html
/// [drop-guarantee]: https://doc.rust-lang.org/nightly/std/pin/index.html#drop-guarantee
/// [pinned-drop]: ./attr.pin_project.html#pinned_drop
/// [repr-packed]: https://doc.rust-lang.org/nomicon/other-reprs.html#reprpacked
/// [pin-projection]: https://doc.rust-lang.org/nightly/std/pin/index.html#projections-and-structural-pinning
/// [undefined-behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
/// [unsafe-unpin]: ./attr.pin_project.html#unsafeunpin
#[proc_macro_attribute]
pub fn pin_project(args: TokenStream, input: TokenStream) -> TokenStream {
    pin_project::attribute(&args.into(), input.into()).into()
}

/// An attribute for annotating an impl block that implements [`Drop`].
///
/// This attribute is only needed when you wish to provide a [`Drop`]
/// impl for your type.
///
/// This impl block acts just like a normal [`Drop`] impl,
/// except for the following two:
///
/// * `drop` method takes [`Pin`]`<&mut Self>`
/// * Name of the trait is `PinnedDrop`.
///
/// ```rust
/// # use std::pin::Pin;
/// pub trait PinnedDrop {
///     fn drop(self: Pin<&mut Self>);
/// }
/// ```
///
/// `#[pin_project]` implements the actual [`Drop`] trait via `PinnedDrop` you
/// implemented. To drop a type that implements `PinnedDrop`, use the [`drop`]
/// function just like dropping a type that directly implements [`Drop`].
///
/// In particular, it will never be called more than once, just like [`Drop::drop`].
///
/// ## Example
///
/// ```rust
/// use pin_project::{pin_project, pinned_drop};
/// use std::pin::Pin;
///
/// #[pin_project(PinnedDrop)]
/// struct Foo {
///     #[pin]
///     field: u8,
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
///     let _x = Foo { field: 50 };
/// }
/// ```
///
/// See also ["pinned-drop" section of `#[pin_project]` attribute][pinned-drop].
///
/// ## Why `#[pinned_drop]` attribute is needed?
///
/// Implementing `PinnedDrop::drop` is safe, but calling it is not safe.
// This is because destructors can be called multiple times in safe code and
/// [double dropping is unsound](https://github.com/rust-lang/rust/pull/62360).
///
/// Ideally, it would be desirable to be able to forbid manual calls in
/// the same way as [`Drop::drop`], but the library cannot do it. So, by using
/// macros and replacing them with private traits like the following, we prevent users from
/// calling `PinnedDrop::drop` in safe code.
///
/// ```rust
/// # use std::pin::Pin;
/// pub trait PinnedDrop {
///     unsafe fn drop(self: Pin<&mut Self>);
/// }
/// ```
///
/// This allows implementing [`Drop`] safely using `#[pinned_drop]`.
/// Also by using the [`drop`] function just like dropping a type that directly implements [`Drop`],
/// can drop safely a type that implements `PinnedDrop`.
///
/// [`Pin`]: core::pin::Pin
/// [pinned-drop]: ./attr.pin_project.html#pinned_drop
#[proc_macro_attribute]
pub fn pinned_drop(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    pinned_drop::attribute(&args.into(), input).into()
}

/// An attribute to provide way to refer to the projected type returned by
/// `project` method.
///
/// The following syntaxes are supported.
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
/// ## `match` expressions
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
/// ## `impl` blocks
///
/// All methods and associated functions in `#[project] impl` block become
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
///     fn bar(self: Pin<&mut Self>) {
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
/// ## `use` statements
///
/// ### Examples
///
/// ```rust
/// # mod dox {
/// use pin_project::pin_project;
///
/// #[pin_project]
/// struct Foo<A> {
///     #[pin]
///     field: A,
/// }
///
/// mod bar {
///     use super::Foo;
///     use pin_project::project;
///     use std::pin::Pin;
///
///     #[project]
///     use super::Foo;
///
///     #[project]
///     fn baz<A>(foo: Pin<&mut Foo<A>>) {
///         #[project]
///         let Foo { field } = foo.project();
///         let _: Pin<&mut A> = field;
///     }
/// }
/// # }
/// ```
#[proc_macro_attribute]
pub fn project(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    project::attribute(&args.into(), input, Mutable).into()
}

/// An attribute to provide way to refer to the projected type returned by
/// `project_ref` method.
///
/// This is the same as [`project`] attribute except it refers to the projected
/// type returned by the `project_ref` method.
///
/// See [`project`] attribute for more details.
///
/// [`project`]: ./attr.project.html
#[proc_macro_attribute]
pub fn project_ref(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    project::attribute(&args.into(), input, Immutable).into()
}

/// An attribute to provide way to refer to the projected type returned by
/// `project_replace` method.
///
/// This is the same as [`project`] attribute except it refers to the projected
/// type returned by the `project_replace` method.
///
/// See [`project`] attribute for more details.
///
/// [`project`]: ./attr.project.html
#[proc_macro_attribute]
pub fn project_replace(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input);
    project::attribute(&args.into(), input, Owned).into()
}

/// An internal helper macro.
#[doc(hidden)]
#[proc_macro_derive(__PinProjectInternalDerive, attributes(pin))]
pub fn __pin_project_internal_derive(input: TokenStream) -> TokenStream {
    pin_project::derive(input.into()).into()
}
