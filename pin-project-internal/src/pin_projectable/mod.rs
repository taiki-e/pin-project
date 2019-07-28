use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Generics, Item, ItemEnum, ItemFn, ItemStruct, Meta, NestedMeta, Result, Type,
};

use crate::utils::VecExt;

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
        Ok(PinProject { items })
    }
}

fn handle_type(args: TokenStream, item: Item, pinned_drop: Option<ItemFn>) -> Result<TokenStream> {
    match item {
        Item::Struct(item) => {
            ensure_not_packed(&item.attrs)?;
            Ok(structs::parse(args, item, pinned_drop)?)
        }
        Item::Enum(item) => {
            ensure_not_packed(&item.attrs)?;
            Ok(enums::parse(args, item, pinned_drop)?)
        }
        _ => panic!("Unexpected item: {:?}", item),
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
                    if let Some(pos) = attrs.iter().position(|a| a.path.is_ident("pin_projectable")) {
                        // Remove the 'pin_projectable' attribute, to prevent it from
                        // being parsed again
                        let attr = attrs.remove(pos);
                        let args = match attr.parse_meta()? {
                            Meta::List(l) => l.nested.into_token_stream(),
                            Meta::Word(_) => TokenStream::new(),
                            _ => return Err(error!(span!(attr), "invalid arguments"))
                        };

                        found_type = Some((item.clone(), args));
                    } else {
                        return Err(error!(span, "type delcared in pin_project! must have pin_projectable attribute"))
                    }
                } else {
                    return Err(error!(span, "cannot declare multiple types within pinned module"))
                }
            },
            Item::Fn(ref mut fn_) => {
                if fn_.attrs.find_remove(PINNED_DROP) {
                    if found_pinned_drop.is_none() {
                        found_pinned_drop = Some(fn_.clone());
                    } else {
                        return Err(error!(span, "cannot declare multiple functions within pinned module"));
                    }
                } else {
                    return Err(error!(span, "only #[pinned_drop] functions cannot be declared within pinend module"));
                }
            },
            _ => return Err(error!(span, "pinned module may only contain a struct/enum with an option #[pinned_drop] function"))
        }
    }

    if found_type.is_none() {
        return Err(error!(span, "pin_project must declare a struct or enum"));
    }

    let (type_, args) = match found_type {
        Some(t) => t,
        None => return Err(error!(span, "No #[pin_projectable] type found!")),
    };

    let res = handle_type(args, type_, found_pinned_drop.clone());
    res
}

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let span = span!(input);
    let item = syn::parse2(input)?;
    match &item {
        Item::Struct(_) | Item::Enum(_) => handle_type(args, item, None),
        _ => Err(error!(span, "may only be used on structs or enums")),
    }
}

fn ensure_not_packed(attrs: &[Attribute]) -> Result<()> {
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
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
    }
    return Ok(());
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
        Ok(ImplDrop { generics, pinned_drop })
    }

    fn build(self, ident: &Ident) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        match self.pinned_drop {
            Some(fn_) => {
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
                            // Therefore, we enforce a return type of '()' by explicitly
                            // assigning it to a temporary.
                            let _: () = #fn_name(pinned_self);
                        }
                    }
                }
            }
            None => {
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
            "" => Ok(Self { generics: generics.clone(), auto: true }),
            "unsafe_Unpin" => Ok(Self { generics: generics.clone(), auto: false }),
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

        let res = if self.auto {
            quote! {
                impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
            }
        } else {
            let pin_project_crate = pin_project_crate_path();
            where_clause.predicates.push(syn::parse_quote!(::#pin_project_crate::Wrapper<#ident #ty_generics>: ::#pin_project_crate::UnsafeUnpin));

            quote! {
                impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
            }
        };
        res
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
