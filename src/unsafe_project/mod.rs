use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Generics, Item, Result, Type};

mod enums;
mod structs;

/// The annotation for pinned type.
const PIN: &str = "pin";

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let span = span!(input);
    match syn::parse2(input) {
        Ok(Item::Struct(item)) => structs::parse(args, item),
        Ok(Item::Enum(item)) => enums::parse(args, item),
        _ => Err(error!(span, "may only be used on structs or enums")),
    }
    .unwrap_or_else(|e| e.to_compile_error())
}

/// Makes the generics of projected type from the reference of the original generics.
fn proj_generics(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    generics.params.insert(0, syn::parse_quote!('__a));
    generics
}

// =================================================================================================
// conditional Unpin implementation

#[derive(Default)]
struct ImplUnpin(Option<Generics>);

impl ImplUnpin {
    /// Parses attribute arguments.
    fn new(args: TokenStream, generics: &Generics) -> Result<Self> {
        match &*args.to_string() {
            "" => Ok(Self::default()),
            "Unpin" => Ok(Self(Some(generics.clone()))),
            _ => Err(error!(args, "an invalid argument was passed")),
        }
    }

    fn push(&mut self, ty: &Type) {
        if let Some(generics) = &mut self.0 {
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: ::core::marker::Unpin));
        }
    }

    /// Creates `Unpin` implementation.
    fn build(self, ident: &Ident) -> TokenStream {
        self.0
            .map(|generics| {
                let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
                quote! {
                    impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
                }
            })
            .unwrap_or_default()
    }
}
