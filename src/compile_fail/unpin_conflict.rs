/// The same implementation.
///
/// ```compile_fail,E0119
/// use pin_project::unsafe_project;
/// use std::marker::Unpin;
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
/// // conflicting implementations
/// impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
/// ```
///
/// The implementation that under different conditions.
///
/// ```compile_fail,E0119
/// use pin_project::unsafe_project;
/// use std::marker::Unpin;
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
/// // conflicting implementations
/// impl<T, U> Unpin for Foo<T, U> {} // Non-conditional Unpin impl
/// ```
///
/// ```compile_fail,E0119
/// use pin_project::unsafe_project;
/// use std::marker::Unpin;
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
/// // conflicting implementations
/// impl<T: Unpin, U: Unpin> Unpin for Foo<T, U> {} // Conditional Unpin impl
/// ```
mod unsafe_project {}

/// The same implementation.
///
/// ```compile_fail,E0119
/// use pin_project::unsafe_fields;
/// use std::marker::Unpin;
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
/// // conflicting implementations
/// impl<T, U> Unpin for Foo<T, U> where T: Unpin {} // Conditional Unpin impl
/// ```
///
/// The implementation that under different conditions.
///
/// ```compile_fail,E0119
/// use pin_project::unsafe_fields;
/// use std::marker::Unpin;
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
/// // conflicting implementations
/// impl<T, U> Unpin for Foo<T, U> {} // Non-conditional Unpin impl
/// ```
///
/// ```compile_fail,E0119
/// use pin_project::unsafe_fields;
/// use std::marker::Unpin;
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
/// // conflicting implementations
/// impl<T: Unpin, U: Unpin> Unpin for Foo<T, U> {} // Conditional Unpin impl
/// ```
#[cfg(feature = "unsafe_fields")]
mod unsafe_fields {}
