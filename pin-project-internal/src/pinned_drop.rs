use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    FnArg, GenericArgument, ItemFn, PatType, PathArguments, Result, ReturnType, Type, TypePath,
    TypeReference, TypeTuple,
};

use crate::utils::crate_path;

pub(crate) fn attribute(input: &ItemFn) -> TokenStream {
    parse(input).unwrap_or_else(|e| e.to_compile_error())
}

fn parse_arg(arg: &FnArg) -> Result<&Type> {
    if let FnArg::Typed(PatType { ty, .. }) = arg {
        if let Type::Path(TypePath { qself: None, path }) = &**ty {
            let ty = &path.segments[path.segments.len() - 1];
            if let PathArguments::AngleBracketed(args) = &ty.arguments {
                if args.args.len() == 1 && ty.ident == "Pin" {
                    // &mut <elem>
                    if let GenericArgument::Type(Type::Reference(TypeReference {
                        mutability: Some(_),
                        elem,
                        ..
                    })) = &args.args[0]
                    {
                        return Ok(&**elem);
                    }
                }
            }
        }
    }

    Err(error!(arg, "#[pinned_drop] function must take a argument `Pin<&mut Type>`"))
}

fn parse(item: &ItemFn) -> Result<TokenStream> {
    if let ReturnType::Type(_, ty) = &item.sig.output {
        match &**ty {
            Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty() => {}
            _ => return Err(error!(ty, "#[pinned_drop] function must return the unit type")),
        }
    }
    if item.sig.inputs.len() != 1 {
        return Err(error!(
            item.sig.inputs,
            "#[pinned_drop] function must take exactly one argument"
        ));
    }

    let crate_path = crate_path();
    let type_ = parse_arg(&item.sig.inputs[0])?;
    let fn_name = &item.sig.ident;
    let (impl_generics, _, where_clause) = item.sig.generics.split_for_impl();

    Ok(quote! {
        unsafe impl #impl_generics ::#crate_path::__private::UnsafePinnedDrop for #type_ #where_clause {
            unsafe fn pinned_drop(self: ::core::pin::Pin<&mut Self>) {
                // Declare the #[pinned_drop] function *inside* our pinned_drop function
                // This guarantees that it's impossible for any other user code
                // to call it.
                #item
                // #[pinned_drop] function is a free function - if it were part of a trait impl,
                // it would be possible for user code to call it by directly invoking the trait.
                #fn_name(self);
            }
        }
    })
}
