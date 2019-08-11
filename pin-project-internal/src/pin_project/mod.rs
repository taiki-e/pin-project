use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    Fields, FieldsNamed, FieldsUnnamed, Generics, Index, Item, ItemStruct, Meta, NestedMeta,
    Result, Type,
};

use crate::utils::crate_path;

mod enums;
mod structs;

/// The annotation for pinned type.
const PIN: &str = "pin";

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    parse(args, input).unwrap_or_else(|e| e.to_compile_error())
}

#[derive(Clone, Copy)]
struct Args {
    pinned_drop: Option<Span>,
    unsafe_unpin: Option<Span>,
}

impl Args {
    fn impl_drop(self, generics: &Generics) -> ImplDrop<'_> {
        ImplDrop::new(generics, self.pinned_drop)
    }

    fn impl_unpin(self, generics: &Generics) -> ImplUnpin {
        ImplUnpin::new(generics, self.unsafe_unpin)
    }
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut pinned_drop = None;
        let mut unsafe_unpin = None;
        while !input.is_empty() {
            let i = input.parse::<Ident>()?;
            match &*i.to_string() {
                "PinnedDrop" => pinned_drop = Some(i.span()),
                "UnsafeUnpin" => unsafe_unpin = Some(i.span()),
                _ => return Err(error!(i, "an invalid argument was passed")),
            }
        }
        Ok(Self { pinned_drop, unsafe_unpin })
    }
}

fn parse(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let args = syn::parse2(args)?;
    match syn::parse2(input)? {
        Item::Struct(item) => {
            let packed_check = ensure_not_packed(&item)?;
            let mut res = structs::parse(args, item)?;
            res.extend(packed_check);
            Ok(res)
        }
        Item::Enum(item) => {
            // We don't need to check for '#[repr(packed)]',
            // since it does not apply to enums
            enums::parse(args, item)
        }
        item => Err(error!(item, "may only be used on structs or enums")),
    }
}

fn ensure_not_packed(item: &ItemStruct) -> Result<TokenStream> {
    for meta in item.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
        if let Meta::List(l) = meta {
            if l.ident == "repr" {
                for repr in l.nested.iter() {
                    if let NestedMeta::Meta(Meta::Word(w)) = repr {
                        if w == "packed" {
                            return Err(error!(
                                w,
                                "pin_project may not be used on #[repr(packed)] types"
                            ));
                        }
                    }
                }
            }
        }
    }

    // Workaround for https://github.com/taiki-e/pin-project/issues/32
    // Through the tricky use of proc macros, it's possible to bypass
    // the above check for the 'repr' attribute.
    // To ensure that it's impossible to use pin projections on a #[repr(packed)][
    // struct, we generate code like this:
    //
    // #[deny(safe_packed_borrows)]
    // fn enforce_not_packed_for_MyStruct(val: MyStruct) {
    //  let _field1 = &val.field1;
    //  let _field2 = &val.field2;
    //  ...
    //  let _fieldn = &val.fieldn;
    // }
    //
    // Taking a reference to a packed field is unsafe, amd appplying
    // #[deny(safe_packed_borrows)] makes sure that doing this without
    // an 'unsafe' block (which we deliberately do not generate)
    // is a hard error.
    //
    // If the struct ends up having #[repr(packed)] applied somehow,
    // this will generate an (unfriendly) error message. Under all reasonable
    // circumstances, we'll detect the #[repr(packed)] attribute, and generate
    // a much nicer error above.
    //
    // There is one exception: If the type of a struct field has a alignemtn of 1
    // (e.g. u8), it is always safe to take a reference to it, even if the struct
    // is #[repr(packed)]. If the struct is composed entirely of types of alignent 1,
    // our generated method will not trigger an error if the struct is #[repr(packed)]
    //
    // Fortunately, this should have no observable consequence - #[repr(packed)]
    // is essentially a no-op on such a type. Nevertheless, we include a test
    // to ensure that the compiler doesn't ever try to copy the fields on
    // such a struct when trying to drop it - which is reason we prevent
    // #[repr(packed)] in the first place
    let mut field_refs = vec![];
    match &item.fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            for field in named.iter() {
                let ident = field.ident.as_ref().unwrap();
                field_refs.push(quote!(&val.#ident;));
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            for (i, _) in unnamed.iter().enumerate() {
                let index = Index::from(i);
                field_refs.push(quote!(&val.#index;));
            }
        }
        Fields::Unit => {}
    }

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let struct_name = &item.ident;
    let method_name = Ident::new(
        &("__pin_project_assert_not_repr_packed_".to_string() + &item.ident.to_string()),
        Span::call_site(),
    );
    let test_fn = quote! {
        #[deny(safe_packed_borrows)]
        fn #method_name #impl_generics (val: #struct_name #ty_generics) #where_clause {
            #(#field_refs)*
        }
    };
    Ok(test_fn)
}

/// Makes the generics of projected type from the reference of the original generics.
fn proj_generics(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    generics.params.insert(0, syn::parse_quote!('__a));
    generics
}

// =================================================================================================
// Drop implementation

struct ImplDrop<'a> {
    generics: &'a Generics,
    pinned_drop: Option<Span>,
}

impl<'a> ImplDrop<'a> {
    fn new(generics: &'a Generics, pinned_drop: Option<Span>) -> Self {
        Self { generics, pinned_drop }
    }

    fn build(self, ident: &Ident) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        if let Some(pinned_drop) = self.pinned_drop {
            let crate_path = crate_path();
            let call = quote_spanned! { pinned_drop =>
                ::#crate_path::__private::UnsafePinnedDrop::pinned_drop(pinned_self)
            };

            quote! {
                impl #impl_generics ::core::ops::Drop for #ident #ty_generics #where_clause {
                    fn drop(&mut self) {
                        // Safety - we're in 'drop', so we know that 'self' will
                        // never move again
                        let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
                        // We call `pinned_drop` only once. Since `UnsafePinnedDrop::pinned_drop`
                        // is an unsafe function and a private API, it is never called again in safe
                        // code *unless the user uses a maliciously crafted macro*.
                        unsafe {
                            #call;
                        }
                    }
                }
            }
        } else {
            quote! {
                impl #impl_generics ::core::ops::Drop for #ident #ty_generics #where_clause {
                    fn drop(&mut self) {
                        // Do nothing. The precense of this Drop
                        // impl ensures that the user can't provide one of their own
                    }
                }
            }
        }
    }
}

// =================================================================================================
// conditional Unpin implementation

struct ImplUnpin {
    generics: Generics,
    unsafe_unpin: bool,
}

impl ImplUnpin {
    fn new(generics: &Generics, unsafe_unpin: Option<Span>) -> Self {
        let mut generics = generics.clone();
        if let Some(unsafe_unpin) = unsafe_unpin {
            let crate_path = crate_path();
            generics.make_where_clause().predicates.push(
                syn::parse2(quote_spanned! { unsafe_unpin =>
                    ::#crate_path::__private::Wrapper<Self>: ::#crate_path::UnsafeUnpin
                })
                .unwrap(),
            );
        }

        Self { generics, unsafe_unpin: unsafe_unpin.is_some() }
    }

    fn push(&mut self, ty: &Type) {
        // We only add bounds for automatically generated impls
        if !self.unsafe_unpin {
            self.generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: ::core::marker::Unpin));
        }
    }

    /// Creates `Unpin` implementation.
    fn build(self, ident: &Ident) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        quote! {
            impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
        }
    }
}
