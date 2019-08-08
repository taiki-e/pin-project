use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    Fields, FieldsNamed, FieldsUnnamed, Generics, Item, ItemEnum, ItemFn, ItemStruct, Meta,
    NestedMeta, Result, ReturnType, Type, TypeTuple, Index
};

use crate::utils::{Nothing, VecExt};

mod enums;
mod structs;

/// The annotation for pinned type.
const PIN: &str = "pin";

const PINNED_DROP: &str = "pinned_drop";

struct PinProject {
    items: Vec<Item>,
}

impl Parse for PinProject {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut items = vec![];
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}

fn handle_type(args: TokenStream, item: Item, pinned_drop: Option<ItemFn>) -> Result<TokenStream> {
    match item {
        Item::Struct(item) => {
            let packed_check = ensure_not_packed(&item)?;
            let mut res = structs::parse(args, item, pinned_drop)?;
            //println!("Packed check: {}", packed_check);
            res.extend(packed_check);
            Ok(res)
        }
        Item::Enum(item) => {
            // We don't need to check for '#[repr(packed)]',
            // since it does not apply to enums
            enums::parse(args, item, pinned_drop)
        }
        _ => unreachable!(),
    }
}

pub(super) fn pin_project(input: TokenStream) -> Result<TokenStream> {
    let span = span!(input);
    let items: Vec<Item> = syn::parse2::<PinProject>(input)?.items;

    let mut found_type = None;
    let mut found_pinned_drop = None;

    for mut item in items {
        match &mut item {
            Item::Struct(ItemStruct { attrs, .. }) | Item::Enum(ItemEnum { attrs, .. }) => {
                if found_type.is_none() {
                    if let Some(attr) = attrs.find_remove("pin_projectable") {
                        let args = match attr.parse_meta()? {
                            Meta::List(l) => l.nested.into_token_stream(),
                            Meta::Word(_) => TokenStream::new(),
                            _ => return Err(error!(span!(attr), "invalid arguments"))
                        };

                        found_type = Some((item.clone(), args));
                    } else {
                        return Err(error!(item, "type declared in pin_project! must have pin_projectable attribute"))
                    }
                } else {
                    return Err(error!(item, "cannot declare multiple types within pinned module"))
                }
            },
            Item::Fn(fn_) => {
                if let Some(attr)= fn_.attrs.find_remove(PINNED_DROP) {
                    let _: Nothing = syn::parse2(attr.tts)?;
                    if found_pinned_drop.is_none() {
                        if let ReturnType::Type(_, ty) = &fn_.decl.output {
                            match &**ty {
                                Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty() => {}
                                _ => return Err(error!(ty, "#[pinned_drop] function must return the unit type")),
                            }
                        }
                        found_pinned_drop = Some(fn_.clone());
                    } else {
                        return Err(error!(fn_, "cannot declare multiple functions within pinned module"));
                    }
                } else {
                    return Err(error!(fn_, "only #[pinned_drop] functions can be declared within a pin_project! block"));
                }
            },
            _ => return Err(error!(item, "pin_project! block may only contain a struct/enum with an optional #[pinned_drop] function"))
        }
    }

    let (type_, args) = match found_type {
        Some(t) => t,
        None => return Err(error!(span, "No #[pin_projectable] type found!")),
    };

    handle_type(args, type_, found_pinned_drop)
}

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let item = syn::parse2(input)?;
    match &item {
        Item::Struct(_) | Item::Enum(_) => handle_type(args, item, None),
        _ => Err(error!(item, "may only be used on structs or enums")),
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
                                "pin_projectable may not be used on #[repr(packed)] types"
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

struct ImplDrop {
    generics: Generics,
    pinned_drop: Option<ItemFn>,
}

impl ImplDrop {
    /// Parses attribute arguments.
    fn new(generics: Generics, pinned_drop: Option<ItemFn>) -> Result<Self> {
        Ok(Self { generics, pinned_drop })
    }

    fn build(self, ident: &Ident) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        if let Some(fn_) = self.pinned_drop {
            let fn_name = fn_.ident.clone();
            quote! {
                impl #impl_generics ::core::ops::Drop for #ident #ty_generics #where_clause {
                    fn drop(&mut self) {
                        // Declare the #[pinned_drop] function *inside* our drop function
                        // This guarantees that it's impossible for any other user code
                        // to call it
                        #fn_
                        // Safety - we're in 'drop', so we know that 'self' will
                        // never move again
                        let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
                        // 'pinned_drop' is a free function - if it were part of a trait impl,
                        // it would be possible for user code to call it by directly invoking
                        // the trait.
                        #fn_name(pinned_self);
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
    auto: bool,
}

impl ImplUnpin {
    /// Parses attribute arguments.
    fn new(args: TokenStream, generics: &Generics) -> Result<Self> {
        let mut generics = generics.clone();
        generics.make_where_clause();

        match &*args.to_string() {
            "" => Ok(Self { generics, auto: true }),
            "unsafe_Unpin" => Ok(Self { generics, auto: false }),
            _ => Err(error!(args, "an invalid argument was passed")),
        }
    }

    fn push(&mut self, ty: &Type) {
        // We only add bounds for automatically generated impls
        if self.auto {
            self.generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: ::core::marker::Unpin));
        }
    }

    /// Creates `Unpin` implementation.
    fn build(self, ident: &Ident) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let mut where_clause = where_clause.unwrap().clone(); // Created in 'new'

        if self.auto {
            quote! {
                impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
            }
        } else {
            let pin_project_crate = pin_project_crate_path();
            where_clause.predicates.push(syn::parse_quote!(::#pin_project_crate::Wrapper<#ident #ty_generics>: ::#pin_project_crate::UnsafeUnpin));

            quote! {
                impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
            }
        }
    }
}

/// If the 'renamed' feature is enabled, we detect
/// the actual name of the 'pin-project' crate in the consumer's Cargo.toml
#[cfg(feature = "renamed")]
fn pin_project_crate_path() -> Ident {
    use crate::PIN_PROJECT_CRATE;
    // This is fairly subtle.
    // Normally, you would use `env!("CARGO_PKG_NAME")` to get the name of the package,
    // since it's set at compile time.
    // However, we're in a proc macro, which runs while *another* crate is being compiled.
    // By retreiving the runtime value of `CARGO_PKG_NAME`, we can figure out the name
    // of the crate that's calling us.
    let cur_crate = std::env::var("CARGO_PKG_NAME")
        .expect("Could not find CARGO_PKG_NAME environemnt variable");
    Ident::new(
        if cur_crate == "pin-project" { "pin_project" } else { PIN_PROJECT_CRATE.as_str() },
        Span::call_site(),
    )
}

/// If the 'renamed' feature is not enabled, we just
/// assume that the 'pin-project' dependency has not been renamed
#[cfg(not(feature = "renamed"))]
fn pin_project_crate_path() -> Ident {
    Ident::new("pin_project", Span::call_site())
}
