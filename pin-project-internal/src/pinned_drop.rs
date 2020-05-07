use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, visit_mut::VisitMut, *};

use crate::utils::{parse_as_empty, prepend_underscore_to_self, ReplaceReceiver, SliceExt};

pub(crate) fn attribute(args: &TokenStream, mut input: ItemImpl) -> TokenStream {
    if let Err(e) = parse_as_empty(args).and_then(|()| parse(&mut input)) {
        let self_ty = &input.self_ty;
        let (impl_generics, _, where_clause) = input.generics.split_for_impl();

        let mut tokens = e.to_compile_error();
        // Generate a dummy `PinnedDrop` implementation.
        // In many cases, `#[pinned_drop] impl` is declared after `#[pin_project]`.
        // Therefore, if `pinned_drop` compile fails, you will also get an error
        // about `PinnedDrop` not being implemented.
        // This can be prevented to some extent by generating a dummy
        // `PinnedDrop` implementation.
        // We already know that we will get a compile error, so this won't
        // accidentally compile successfully.
        tokens.extend(quote! {
            impl #impl_generics ::pin_project::__private::PinnedDrop for #self_ty #where_clause {
                unsafe fn drop(self: ::pin_project::__reexport::pin::Pin<&mut Self>) {}
            }
        });
        tokens
    } else {
        input.into_token_stream()
    }
}

fn parse_method(method: &ImplItemMethod) -> Result<()> {
    fn get_ty_path(ty: &Type) -> Option<&Path> {
        if let Type::Path(TypePath { qself: None, path }) = ty { Some(path) } else { None }
    }

    const INVALID_ARGUMENT: &str = "method `drop` must take an argument `self: Pin<&mut Self>`";

    if method.sig.ident != "drop" {
        return Err(error!(
            method.sig.ident,
            "method `{}` is not a member of trait `PinnedDrop", method.sig.ident,
        ));
    }

    if let ReturnType::Type(_, ty) = &method.sig.output {
        match &**ty {
            Type::Tuple(ty) if ty.elems.is_empty() => {}
            _ => return Err(error!(ty, "method `drop` must return the unit type")),
        }
    }

    if method.sig.inputs.len() != 1 {
        if method.sig.inputs.is_empty() {
            return Err(Error::new(method.sig.paren_token.span, INVALID_ARGUMENT));
        } else {
            return Err(error!(&method.sig.inputs, INVALID_ARGUMENT));
        }
    }

    if let FnArg::Typed(pat) = &method.sig.inputs[0] {
        // by-ref binding `ref (mut) self` and sub-patterns `@` are not allowed in receivers (rejected by rustc).
        // <pat>: <path>
        if let (Pat::Ident(pat @ PatIdent { by_ref: None, subpat: None, .. }), Some(path)) =
            (&*pat.pat, get_ty_path(&pat.ty))
        {
            let ty = path.segments.last().unwrap();
            if let PathArguments::AngleBracketed(args) = &ty.arguments {
                // <pat>: (<path>::)<ty><&mut <elem>..>
                if let Some(GenericArgument::Type(Type::Reference(TypeReference {
                    mutability: Some(_),
                    elem,
                    ..
                }))) = &args.args.first()
                {
                    // (mut) self: (<path>::)Pin<&mut Self>
                    if args.args.len() == 1
                        && pat.ident == "self"
                        && ty.ident == "Pin"
                        && get_ty_path(elem).map_or(false, |path| path.is_ident("Self"))
                    {
                        if method.sig.unsafety.is_some() {
                            return Err(error!(
                                method.sig.unsafety,
                                "implementing the method `drop` is not unsafe"
                            ));
                        }
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(error!(method.sig.inputs[0], INVALID_ARGUMENT))
}

fn parse(item: &mut ItemImpl) -> Result<()> {
    const INVALID_ITEM: &str =
        "#[pinned_drop] may only be used on implementation for the `PinnedDrop` trait";

    if let Some(attr) = item.attrs.find("pinned_drop") {
        return Err(error!(attr, "duplicate #[pinned_drop] attribute"));
    }

    if let Some((_, path, _)) = &mut item.trait_ {
        if path.is_ident("PinnedDrop") {
            *path = syn::parse2(quote_spanned! { path.span() =>
                ::pin_project::__private::PinnedDrop
            })
            .unwrap();
        } else {
            return Err(error!(path, INVALID_ITEM));
        }
    } else {
        return Err(error!(item.self_ty, INVALID_ITEM));
    }

    if item.unsafety.is_some() {
        return Err(error!(item.unsafety, "implementing the trait `PinnedDrop` is not unsafe"));
    }
    if item.items.is_empty() {
        return Err(error!(item, "not all trait items implemented, missing: `drop`"));
    }

    item.items.iter().enumerate().try_for_each(|(i, item)| match item {
        ImplItem::Const(item) => {
            Err(error!(item, "const `{}` is not a member of trait `PinnedDrop`", item.ident))
        }
        ImplItem::Type(item) => {
            Err(error!(item, "type `{}` is not a member of trait `PinnedDrop`", item.ident))
        }
        ImplItem::Method(method) => {
            parse_method(method)?;
            if i == 0 {
                Ok(())
            } else {
                Err(error!(method, "duplicate definitions with name `drop`"))
            }
        }
        _ => unreachable!("unexpected ImplItem"),
    })?;

    expand_item(item);

    Ok(())
}

// from:
//
// fn drop(self: Pin<&mut Self>) {
//     // something
// }
//
// into:
//
// unsafe fn drop(self: Pin<&mut Self>) {
//     fn __drop_inner<T>(__self: Pin<&mut Foo<'_, T>>) {
//         // something
//     }
//     __drop_inner(self);
// }
//
fn expand_item(item: &mut ItemImpl) {
    let method =
        if let ImplItem::Method(method) = &mut item.items[0] { method } else { unreachable!() };
    let mut drop_inner = method.clone();

    // `fn drop(mut self: Pin<&mut Self>)` -> `fn __drop_inner<T>(mut __self: Pin<&mut Receiver>)`
    drop_inner.sig.ident = Ident::new("__drop_inner", drop_inner.sig.ident.span());
    drop_inner.sig.generics = item.generics.clone();
    if let FnArg::Typed(arg) = &mut drop_inner.sig.inputs[0] {
        if let Pat::Ident(ident) = &mut *arg.pat {
            prepend_underscore_to_self(&mut ident.ident);
        }
    }
    // This lint does not warn the receiver.
    drop_inner.attrs.push(syn::parse_quote!(#[allow(clippy::needless_pass_by_value)]));
    let mut visitor = ReplaceReceiver::new(&item.self_ty);
    visitor.visit_signature_mut(&mut drop_inner.sig);
    visitor.visit_block_mut(&mut drop_inner.block);

    // `fn drop(mut self: Pin<&mut Self>)` -> `unsafe fn drop(self: Pin<&mut Self>)`
    method.sig.unsafety = Some(token::Unsafe::default());
    if let FnArg::Typed(arg) = &mut method.sig.inputs[0] {
        if let Pat::Ident(ident) = &mut *arg.pat {
            ident.mutability = None;
        }
    }

    method.block = syn::parse_quote! {{
        #drop_inner
        __drop_inner(self);
    }};
}
