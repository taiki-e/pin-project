//! An attribute that would create a projection struct covering all the fields.
//!
//! ## Examples
//!
//! ```rust
//! use pin_project::unsafe_project;
//! use std::marker::Unpin;
//! use std::pin::Pin;
//!
//! #[unsafe_project]
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
//! impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl
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
#![doc(html_root_url = "https://docs.rs/pin-project/0.1.1")]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{Attribute, Field, Fields, FieldsNamed, Ident, ItemStruct};

/// An attribute that would create a projection struct covering all the fields.
///
/// For the field that use `#[pin]` attribute, makes the pinned reference to the field.
///
/// For the other field, makes the unpinned reference to the field.
///
/// ## Safety
///
/// For the field that use `#[pin]` attribute, three things need to be ensured:
///
/// - If the struct implements [`Drop`], the [`drop`] method is not allowed to
///   move the value of the field.
/// - If the struct wants to implement [`Unpin`], it has to do so conditionally:
///   The struct can only implement [`Unpin`] if the field's type is [`Unpin`].
/// - The struct must not be `#[repr(packed)]`.
///
/// For the other field, need to be ensured that the contained value not pinned in
/// the current context.
///
/// ## Examples
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
    if !args.is_empty() {
        return compile_err("`unsafe_project` do not requires arguments");
    }

    let mut item: ItemStruct = match syn::parse(input) {
        Err(_) => return compile_err("`unsafe_project` may only be used on structs"),
        Ok(i) => i,
    };

    let fields = match &mut item.fields {
        Fields::Named(FieldsNamed { named, .. }) if !named.is_empty() => named,
        Fields::Named(_) => return err("zero fields"),
        Fields::Unnamed(_) => return err("unnamed fields"),
        Fields::Unit => return err("with units"),
    };

    let mut proj_fields = Vec::with_capacity(fields.len());
    let mut proj_init = Vec::with_capacity(fields.len());
    let pin = quote!(core::pin::Pin);

    fields.iter_mut().for_each(
        |Field {
             attrs, ident, ty, ..
         }| {
            match find_remove(attrs, "pin") {
                Some(_) => {
                    proj_fields.push(quote!(#ident: #pin<&'__a mut #ty>));
                    proj_init
                        .push(quote!(#ident: unsafe { #pin::new_unchecked(&mut this.#ident) }));
                }
                None => {
                    proj_fields.push(quote!(#ident: &'__a mut #ty));
                    proj_init.push(quote!(#ident: &mut this.#ident));
                }
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

    let mut item = item.into_token_stream();
    item.extend(proj_item);
    item.extend(proj_impl);
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
        .position(|attr| attr.path.is_ident(ident) && attr.tts.is_empty())
        .map(|i| remove(attrs, i))
}
