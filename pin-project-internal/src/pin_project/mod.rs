use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    *,
};

use crate::utils::{self, crate_path, proj_ident, proj_trait_ident};

mod enums;
mod structs;

/// The annotation for pinned type.
const PIN: &str = "pin";

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    parse(args, input).unwrap_or_else(|e| e.to_compile_error())
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
            let i = input.parse::<Ident>()?;
            match &*i.to_string() {
                "PinnedDrop" => pinned_drop = Some(i.span()),
                "UnsafeUnpin" => unsafe_unpin = Some(i.span()),
                _ => return Err(error!(i, "an invalid argument was passed")),
            }

            if !input.is_empty() {
                let _: Comma = input.parse()?;
            }
        }
        Ok(Self { pinned_drop, unsafe_unpin })
    }
}

struct Context {
    /// Name of the original type.
    orig_ident: Ident,
    /// Name of the projected type.
    proj_ident: Ident,
    /// Name of the trait generated
    /// to provide a 'project' method
    proj_trait: Ident,

    /// Generics of original type.
    generics: Generics,

    /// Lifetime added to projected type.
    lifetime: Lifetime,

    /// Where-clause for conditional Unpin implementation.
    impl_unpin: WhereClause,

    unsafe_unpin: bool,
    pinned_drop: Option<Span>,
}

impl Context {
    fn new(args: TokenStream, orig_ident: &Ident, generics: &Generics) -> Result<Self> {
        let Args { pinned_drop, unsafe_unpin } = syn::parse2(args)?;
        let proj_ident = proj_ident(orig_ident);
        let proj_trait = proj_trait_ident(orig_ident);
        let lifetime = proj_lifetime(&generics.params);
        let mut generics = generics.clone();

        let mut impl_unpin = generics.make_where_clause().clone();
        if let Some(unsafe_unpin) = unsafe_unpin {
            let crate_path = crate_path();
            impl_unpin.predicates.push(
                syn::parse2(quote_spanned! { unsafe_unpin =>
                    ::#crate_path::__private::Wrapper<Self>: ::#crate_path::UnsafeUnpin
                })
                .unwrap(),
            );
        }

        Ok(Self {
            orig_ident: orig_ident.clone(),
            proj_ident,
            proj_trait,
            generics,
            lifetime,
            impl_unpin,
            unsafe_unpin: unsafe_unpin.is_some(),
            pinned_drop,
        })
    }

    fn push_unpin_bounds(&mut self, ty: &Type) {
        // We only add bounds for automatically generated impls
        if !self.unsafe_unpin {
            self.impl_unpin.predicates.push(syn::parse_quote!(#ty: ::core::marker::Unpin));
        }
    }

    /// Makes conditional `Unpin` implementation for original type.
    fn make_unpin_impl(&self) -> TokenStream {
        let orig_ident = &self.orig_ident;
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();
        let where_clause = &self.impl_unpin;

        quote! {
            impl #impl_generics ::core::marker::Unpin for #orig_ident #ty_generics #where_clause {}
        }
    }

    /// Makes `Drop` implementation for original type.
    fn make_drop_impl(&self) -> TokenStream {
        let orig_ident = &self.orig_ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        if let Some(pinned_drop) = self.pinned_drop {
            let crate_path = crate_path();
            let call = quote_spanned! { pinned_drop =>
                ::#crate_path::__private::UnsafePinnedDrop::pinned_drop(pinned_self)
            };

            quote! {
                impl #impl_generics ::core::ops::Drop for #orig_ident #ty_generics #where_clause {
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
            // If the user does not provide a pinned_drop impl,
            // we need to ensure that they don't provide a `Drop` impl of their
            // own.
            // Based on https://github.com/upsuper/assert-impl/blob/f503255b292ab0ba8d085b657f4065403cfa46eb/src/lib.rs#L80-L87
            //
            // We create a new identifier for each struct, so that the traits
            // for different types do not conflcit with each other
            //
            // Another approach would be to provide an empty Drop impl,
            // which would conflict with a user-provided Drop impl.
            // However, this would trigger the compiler's special handling
            // of Drop types (e.g. fields cannot be moved out of a Drop type).
            // This approach prevents the creation of needless Drop impls,
            // giving users more flexibility
            let trait_ident = format_ident!("{}MustNotImplDrop", orig_ident);
            quote! {
                // There are two possible cases:
                // 1. The user type does not implement Drop. In this case,
                // the first blanked impl will not apply to it. This code
                // will compile, as there is only one impl of MustNotImplDrop for the user type
                // 2. The user type does impl Drop. This will make the blanket impl applicable,
                // which will then comflict with the explicit MustNotImplDrop impl below.
                // This will result in a compilation error, which is exactly what we want
                trait #trait_ident {}
                #[allow(clippy::drop_bounds)]
                impl<T: ::core::ops::Drop> #trait_ident for T {}
                impl #impl_generics #trait_ident for #orig_ident #ty_generics #where_clause {}
            }
        }
    }

    fn make_proj_trait(&self) -> TokenStream {
        let Self { proj_ident, proj_trait, lifetime, .. } = self;
        let proj_generics = proj_generics(&self.generics, lifetime);
        let proj_ty_generics = proj_generics.split_for_impl().1;

        let (orig_generics, _, orig_where_clause) = self.generics.split_for_impl();

        quote! {
            trait #proj_trait #orig_generics {
                fn project<#lifetime>(&#lifetime mut self) -> #proj_ident #proj_ty_generics #orig_where_clause;
            }
        }
    }
}

fn parse(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    match syn::parse2(input)? {
        Item::Struct(item) => {
            let mut cx = Context::new(args, &item.ident, &item.generics)?;

            let packed_check = ensure_not_packed(&item)?;
            let mut res = structs::parse(&mut cx, item)?;
            res.extend(packed_check);
            res.extend(cx.make_drop_impl());
            res.extend(cx.make_unpin_impl());
            res.extend(cx.make_proj_trait());
            Ok(res)
        }
        Item::Enum(item) => {
            let mut cx = Context::new(args, &item.ident, &item.generics)?;

            // We don't need to check for '#[repr(packed)]',
            // since it does not apply to enums
            let mut res = enums::parse(&mut cx, item)?;
            res.extend(cx.make_drop_impl());
            res.extend(cx.make_unpin_impl());
            res.extend(cx.make_proj_trait());
            Ok(res)
        }
        item => Err(error!(item, "may only be used on structs or enums")),
    }
}

fn ensure_not_packed(item: &ItemStruct) -> Result<TokenStream> {
    for meta in item.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
        if let Meta::List(l) = meta {
            if l.path.is_ident("repr") {
                for repr in &l.nested {
                    if let NestedMeta::Meta(Meta::Path(p)) = repr {
                        if p.is_ident("packed") {
                            return Err(error!(
                                p,
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
            for field in named {
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
    let method_name = format_ident!("__pin_project_assert_not_repr_packed_{}", item.ident);
    let test_fn = quote! {
        #[allow(nonstandard_style)]
        #[deny(safe_packed_borrows)]
        fn #method_name #impl_generics (val: #struct_name #ty_generics) #where_clause {
            #(#field_refs)*
        }
    };
    Ok(test_fn)
}

/// Determine the lifetime names. Ensure it doesn't overlap with any existing lifetime names.
fn proj_lifetime(generics: &Punctuated<GenericParam, Comma>) -> Lifetime {
    let mut lifetime_name = String::from("'_pin");
    utils::proj_lifetime_name(&mut lifetime_name, generics);
    Lifetime::new(&lifetime_name, Span::call_site())
}

/// Makes the generics of projected type from the reference of the original generics.
fn proj_generics(generics: &Generics, lifetime: &Lifetime) -> Generics {
    let mut generics = generics.clone();
    utils::proj_generics(&mut generics, lifetime.clone());
    generics
}
