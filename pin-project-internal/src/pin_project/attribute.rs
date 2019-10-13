use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    token::Comma,
    *,
};

use crate::utils::SliceExt;

use super::PIN;

// To generate the correct `Unpin` implementation and the projection methods,
// we need to collect the types of the pinned fields.
// However, since proc-macro-attribute is applied before `#[cfg]` and `#[cfg_attr]` on fields,
// we cannot be collecting field types properly at this timing.
// So instead of generating the `Unpin` implementation and the projection methods here,
// delegate their processing to proc-macro-derive.

pub(super) fn parse_attribute(args: TokenStream, mut item: Item) -> Result<TokenStream> {
    let Args { pinned_drop, unsafe_unpin } = syn::parse2(args)?;

    let drop_impl = match &mut item {
        Item::Struct(ItemStruct { attrs, ident, generics, fields, .. }) => {
            super::validate_struct(ident, fields)?;
            push_pin_attr(attrs, unsafe_unpin)?;
            make_drop_impl(ident, generics, pinned_drop)
        }
        Item::Enum(ItemEnum { attrs, ident, generics, brace_token, variants, .. }) => {
            super::validate_enum(*brace_token, variants)?;
            push_pin_attr(attrs, unsafe_unpin)?;
            make_drop_impl(ident, generics, pinned_drop)
        }
        _ => {
            return Err(error!(
                item,
                "#[pin_project] attribute may only be used on structs or enums"
            ));
        }
    };

    let mut item = item.into_token_stream();
    item.extend(drop_impl);
    Ok(item)
}

#[allow(dead_code)] // https://github.com/rust-lang/rust/issues/56750
struct Args {
    pinned_drop: Option<Span>,
    unsafe_unpin: Option<Span>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut pinned_drop = None;
        let mut unsafe_unpin = None;
        while !input.is_empty() {
            let arg = input.parse::<Ident>()?;
            match &*arg.to_string() {
                "PinnedDrop" => {
                    if pinned_drop.is_some() {
                        return Err(error!(arg, "duplicate `PinnedDrop` argument"));
                    }
                    pinned_drop = Some(arg.span());
                }
                "UnsafeUnpin" => {
                    if unsafe_unpin.is_some() {
                        return Err(error!(arg, "duplicate `UnsafeUnpin` argument"));
                    }
                    unsafe_unpin = Some(arg.span());
                }
                _ => {
                    return Err(error!(
                        arg,
                        "an invalid argument was passed to #[pin_project] attribute"
                    ));
                }
            }

            if !input.is_empty() {
                let _: Comma = input.parse()?;
            }
        }

        Ok(Self { pinned_drop, unsafe_unpin })
    }
}

fn push_pin_attr(attrs: &mut Vec<Attribute>, unsafe_unpin: Option<Span>) -> Result<()> {
    if let Some(attr) = attrs.find(PIN) {
        return Err(error!(
            attr,
            "#[pin] attribute may only be used on fields of structs or variants"
        ));
    }

    attrs.push(syn::parse_quote!(#[derive(::pin_project::__private::__PinProjectInternalDerive)]));
    if let Some(span) = unsafe_unpin {
        // Make the error message highlight `UnsafeUnpin` argument.
        let attr = quote_spanned!(span => #[pin(__unsafe_unpin)]);
        attrs.push(syn::parse_quote!(#attr));
    } else {
        attrs.push(syn::parse_quote!(#[pin(__auto_impl_unpin)]));
    }
    Ok(())
}

/// Creates `Drop` implementation for original type.
fn make_drop_impl(ident: &Ident, generics: &Generics, pinned_drop: Option<Span>) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    if let Some(pinned_drop) = pinned_drop {
        // Make the error message highlight `PinnedDrop` argument.
        // See https://github.com/taiki-e/pin-project/issues/16#issuecomment-513586812
        // for why this is only for the span of function calls, not the entire `impl` block.
        let call_drop = quote_spanned! { pinned_drop =>
            ::pin_project::__private::PinnedDrop::drop(pinned_self)
        };

        quote! {
            #[allow(single_use_lifetimes)]
            impl #impl_generics ::core::ops::Drop for #ident #ty_generics #where_clause {
                fn drop(&mut self) {
                    // Safety - we're in 'drop', so we know that 'self' will
                    // never move again.
                    let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
                    // We call `pinned_drop` only once. Since `PinnedDrop::drop`
                    // is an unsafe function and a private API, it is never called again in safe
                    // code *unless the user uses a maliciously crafted macro*.
                    unsafe {
                        #call_drop;
                    }
                }
            }
        }
    } else {
        // If the user does not provide a pinned_drop impl,
        // we need to ensure that they don't provide a `Drop` impl of their
        // own.
        // Based on https://github.com/upsuper/assert-impl/blob/f503255b292ab0ba8d085b657f4065403cfa46eb/src/lib.rs#L80-L87
        //
        // We create a new identifier for each struct, so that the traits
        // for different types do not conflcit with each other.
        //
        // Another approach would be to provide an empty Drop impl,
        // which would conflict with a user-provided Drop impl.
        // However, this would trigger the compiler's special handling
        // of Drop types (e.g. fields cannot be moved out of a Drop type).
        // This approach prevents the creation of needless Drop impls,
        // giving users more flexibility.
        let trait_ident = format_ident!("{}MustNotImplDrop", ident);

        quote! {
            // There are two possible cases:
            // 1. The user type does not implement Drop. In this case,
            // the first blanked impl will not apply to it. This code
            // will compile, as there is only one impl of MustNotImplDrop for the user type
            // 2. The user type does impl Drop. This will make the blanket impl applicable,
            // which will then comflict with the explicit MustNotImplDrop impl below.
            // This will result in a compilation error, which is exactly what we want.
            trait #trait_ident {}
            #[allow(clippy::drop_bounds)]
            impl<T: ::core::ops::Drop> #trait_ident for T {}
            #[allow(single_use_lifetimes)]
            impl #impl_generics #trait_ident for #ident #ty_generics #where_clause {}
        }
    }
}
