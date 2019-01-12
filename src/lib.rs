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

mod compile_fail;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse_quote, Attribute, Field, Fields, FieldsNamed, ItemStruct};

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
    let mut item: ItemStruct = match syn::parse(input) {
        Err(_) => return compile_err("`unsafe_project` may only be used on structs"),
        Ok(item) => item,
    };

    let mut impl_unpin = match &*args.to_string() {
        "" => None,
        "Unpin" => Some(item.generics.clone()),
        _ => return compile_err("`unsafe_project` an invalid argument was passed"),
    };

    let fields = match &mut item.fields {
        Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => named,
        Fields::Named(_) => return err("zero fields"),
        Fields::Unnamed(_) => return err("unnamed fields"),
        Fields::Unit => return err("with units"),
    };

    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    let pin = quote!(::core::pin::Pin);
    let unpin = quote!(::core::marker::Unpin);

    fields.iter_mut().for_each(
        |Field {
             attrs, ident, ty, ..
         }| {
            if find_remove(attrs, "pin").is_some() {
                proj_fields.push(quote!(#ident: #pin<&'__a mut #ty>));
                proj_init.push(quote!(#ident: unsafe { #pin::new_unchecked(&mut this.#ident) }));

                if let Some(generics) = &mut impl_unpin {
                    generics
                        .make_where_clause()
                        .predicates
                        .push(parse_quote!(#ty: #unpin));
                }
            } else {
                proj_fields.push(quote!(#ident: &'__a mut #ty));
                proj_init.push(quote!(#ident: &mut this.#ident));
            }
        },
    );

    let proj_ident = Ident::new(&format!("__{}Projection", item.ident), Span::call_site());
    let proj_generics = {
        let generics = item.generics.params.iter();
        quote!(<'__a, #(#generics),*>)
    };
    let proj_item = quote! {
        struct #proj_ident #proj_generics {
            #(#proj_fields,)*
        }
    };

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let proj_impl = quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            fn project<'__a>(self: #pin<&'__a mut Self>) -> #proj_ident #proj_generics {
                let this = unsafe { #pin::get_unchecked_mut(self) };
                #proj_ident { #(#proj_init,)* }
            }
        }
    };
    let impl_unpin = impl_unpin
        .as_ref()
        .map(|generics| {
            let where_clause = generics.split_for_impl().2;
            quote! {
                impl #impl_generics #unpin for #ident #ty_generics #where_clause {}
            }
        })
        .unwrap_or_default();

    let mut item = item.into_token_stream();
    item.extend(proj_item);
    item.extend(proj_impl);
    item.extend(impl_unpin);
    TokenStream::from(item)
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
    let mut item: ItemStruct = match syn::parse(input) {
        Err(_) => return compile_err("`unsafe_fields` may only be used on structs"),
        Ok(item) => item,
    };

    let mut impl_unpin = match &*args.to_string() {
        "" => None,
        "Unpin" => Some(item.generics.clone()),
        _ => return compile_err("`unsafe_fields` an invalid argument was passed"),
    };

    let fields = match &mut item.fields {
        Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => named,
        Fields::Named(_) => return err("zero fields"),
        Fields::Unnamed(_) => return err("unnamed fields"),
        Fields::Unit => return err("with units"),
    };

    let mut proj_methods = Vec::with_capacity(fields.len());
    let pin = quote!(::core::pin::Pin);
    let unpin = quote!(::core::marker::Unpin);

    fields.iter_mut().for_each(
        |Field {
             attrs, ident, ty, ..
         }| {
            if find_remove(attrs, "skip").is_none() {
                if find_remove(attrs, "pin").is_some() {
                    proj_methods.push(quote! {
                        fn #ident<'__a>(self: #pin<&'__a mut Self>) -> #pin<&'__a mut #ty> {
                            unsafe { #pin::map_unchecked_mut(self, |x| &mut x.#ident) }
                        }
                    });

                    if let Some(generics) = &mut impl_unpin {
                        generics
                            .make_where_clause()
                            .predicates
                            .push(parse_quote!(#ty: #unpin));
                    }
                } else {
                    proj_methods.push(quote! {
                        fn #ident<'__a>(self: #pin<&'__a mut Self>) -> &'__a mut #ty {
                            unsafe { &mut #pin::get_unchecked_mut(self).#ident }
                        }
                    });
                }
            }
        },
    );

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let proj_impl = quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#proj_methods)*
        }
    };
    let impl_unpin = impl_unpin
        .as_ref()
        .map(|generics| {
            let where_clause = generics.split_for_impl().2;
            quote! {
                impl #impl_generics #unpin for #ident #ty_generics #where_clause {}
            }
        })
        .unwrap_or_default();

    let mut item = item.into_token_stream();
    item.extend(proj_impl);
    item.extend(impl_unpin);
    TokenStream::from(item)
}

#[inline(never)]
fn compile_err(msg: &str) -> TokenStream {
    TokenStream::from(quote!(compile_error!(#msg);))
}

#[inline(never)]
fn err(msg: &str) -> TokenStream {
    compile_err(&format!("cannot be implemented for structs with {}", msg))
}

fn find_remove(attrs: &mut Vec<Attribute>, ident: &str) -> Option<Attribute> {
    fn remove<T>(v: &mut Vec<T>, index: usize) -> T {
        match v.len() {
            1 => v.pop().unwrap(),
            2 => v.swap_remove(index),
            _ => v.remove(index),
        }
    }

    attrs
        .iter()
        .position(|Attribute { path, tts, .. }| path.is_ident(ident) && tts.is_empty())
        .map(|i| remove(attrs, i))
}
