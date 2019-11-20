use std::mem;

use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    *,
};

use crate::utils::{parse_as_empty, CURRENT_PRIVATE_MODULE};

pub(crate) fn attribute(args: &TokenStream, mut input: ItemImpl) -> TokenStream {
    if let Err(e) = parse_as_empty(args).and_then(|()| parse(&mut input)) {
        let self_ty = &input.self_ty;
        let (impl_generics, _, where_clause) = input.generics.split_for_impl();

        let mut tokens = e.to_compile_error();
        let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());
        // Generate a dummy `PinnedDrop` implementation.
        // In many cases, `#[pinned_drop] impl` is declared after `#[pin_project]`.
        // Therefore, if `pinned_drop` compile fails, you will also get an error
        // about `PinnedDrop` not being implemented.
        // This can be prevented to some extent by generating a dummy
        // `PinnedDrop` implementation.
        // We already know that we will get a compile error, so this won't
        // accidentally compile successfully.
        tokens.extend(quote! {
            impl #impl_generics ::pin_project::#private::PinnedDrop for #self_ty #where_clause {
                unsafe fn drop(self: ::core::pin::Pin<&mut Self>) {}
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
            Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty() => {}
            _ => return Err(error!(ty, "method `drop` must return the unit type")),
        }
    }

    if method.sig.inputs.len() != 1 {
        if method.sig.inputs.is_empty() {
            return Err(syn::Error::new(method.sig.paren_token.span, INVALID_ARGUMENT));
        } else {
            return Err(error!(&method.sig.inputs, INVALID_ARGUMENT));
        }
    }

    if let FnArg::Typed(PatType { pat, ty, .. }) = &method.sig.inputs[0] {
        // !by_ref (mutability) ident !subpat: path
        if let (Pat::Ident(PatIdent { by_ref: None, ident, subpat: None, .. }), Some(path)) =
            (&**pat, get_ty_path(ty))
        {
            let ty = &path.segments.last().unwrap();
            if let PathArguments::AngleBracketed(args) = &ty.arguments {
                // (mut) self: (path::)Pin<args>
                if ident == "self" && args.args.len() == 1 && ty.ident == "Pin" {
                    // &mut <elem>
                    if let GenericArgument::Type(Type::Reference(TypeReference {
                        mutability: Some(_),
                        elem,
                        ..
                    })) = &args.args[0]
                    {
                        if get_ty_path(elem).map_or(false, |path| path.is_ident("Self")) {
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
    }

    Err(error!(method.sig.inputs[0], INVALID_ARGUMENT))
}

fn parse(item: &mut ItemImpl) -> Result<()> {
    if let Some((_, path, _)) = &mut item.trait_ {
        if path.is_ident("PinnedDrop") {
            let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());
            *path = syn::parse2(quote_spanned! { path.span() =>
                ::pin_project::#private::PinnedDrop
            })
            .unwrap();
        } else {
            return Err(error!(
                path,
                "#[pinned_drop] may only be used on implementation for the `PinnedDrop` trait"
            ));
        }
    } else {
        return Err(error!(
            item.self_ty,
            "#[pinned_drop] may only be used on implementation for the `PinnedDrop` trait"
        ));
    }

    if item.unsafety.is_some() {
        return Err(error!(item.unsafety, "implementing the trait `PinnedDrop` is not unsafe"));
    }
    if item.items.is_empty() {
        return Err(error!(item, "not all trait items implemented, missing: `drop`"));
    }

    for (i, item) in item.items.iter().enumerate() {
        match item {
            ImplItem::Const(item) => {
                return Err(error!(
                    item,
                    "const `{}` is not a member of trait `PinnedDrop`", item.ident
                ));
            }
            ImplItem::Type(item) => {
                return Err(error!(
                    item,
                    "type `{}` is not a member of trait `PinnedDrop`", item.ident
                ));
            }
            ImplItem::Method(method) => {
                parse_method(method)?;
                if i != 0 {
                    return Err(error!(method, "duplicate definitions with name `drop`"));
                }
            }
            _ => parse_as_empty(&item.to_token_stream())?,
        }
    }

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

// Replace `self` and `Self` with `__self` and `self_ty`.
// Based on https://github.com/dtolnay/async-trait/blob/1.0.15/src/receiver.rs

struct ReplaceReceiver<'a> {
    self_ty: &'a Type,
}

impl<'a> ReplaceReceiver<'a> {
    fn new(self_ty: &'a Type) -> Self {
        Self { self_ty }
    }

    fn self_to_qself(&mut self, qself: &mut Option<QSelf>, path: &mut Path) {
        if path.leading_colon.is_some() {
            return;
        }

        let first = &path.segments[0];
        if first.ident != "Self" || !first.arguments.is_empty() {
            return;
        }

        match path.segments.pairs().next().unwrap().punct() {
            Some(colon) => path.leading_colon = Some(**colon),
            None => return,
        }

        *qself = Some(QSelf {
            lt_token: token::Lt::default(),
            ty: Box::new(self.self_ty.clone()),
            position: 0,
            as_token: None,
            gt_token: token::Gt::default(),
        });

        let segments = mem::replace(&mut path.segments, Punctuated::new());
        path.segments = segments.into_pairs().skip(1).collect();
    }
}

impl VisitMut for ReplaceReceiver<'_> {
    // `Self` -> `Receiver`
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(node) = ty {
            if node.qself.is_none() && node.path.is_ident("Self") {
                *ty = self.self_ty.clone();
            } else {
                self.visit_type_path_mut(node);
            }
        } else {
            visit_mut::visit_type_mut(self, ty);
        }
    }

    // `Self::Assoc` -> `<Receiver>::Assoc`
    fn visit_type_path_mut(&mut self, ty: &mut TypePath) {
        if ty.qself.is_none() {
            self.self_to_qself(&mut ty.qself, &mut ty.path);
        }
        visit_mut::visit_type_path_mut(self, ty);
    }

    // `Self::method` -> `<Receiver>::method`
    fn visit_expr_path_mut(&mut self, expr: &mut ExprPath) {
        if expr.qself.is_none() {
            prepend_underscore_to_self(&mut expr.path.segments[0].ident);
            self.self_to_qself(&mut expr.qself, &mut expr.path);
        }
        visit_mut::visit_expr_path_mut(self, expr);
    }

    fn visit_macro_mut(&mut self, node: &mut Macro) {
        // We can't tell in general whether `self` inside a macro invocation
        // refers to the self in the argument list or a different self
        // introduced within the macro. Heuristic: if the macro input contains
        // `fn`, then `self` is more likely to refer to something other than the
        // outer function's self argument.
        if !contains_fn(node.tokens.clone()) {
            node.tokens = fold_token_stream(node.tokens.clone());
        }
    }

    fn visit_item_mut(&mut self, _: &mut Item) {
        // Do not recurse into nested items.
    }
}

fn contains_fn(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|tt| match tt {
        TokenTree::Ident(ident) => ident == "fn",
        TokenTree::Group(group) => contains_fn(group.stream()),
        _ => false,
    })
}

fn fold_token_stream(tokens: TokenStream) -> TokenStream {
    tokens
        .into_iter()
        .map(|tt| match tt {
            TokenTree::Ident(mut ident) => {
                prepend_underscore_to_self(&mut ident);
                TokenTree::Ident(ident)
            }
            TokenTree::Group(group) => {
                let content = fold_token_stream(group.stream());
                TokenTree::Group(Group::new(group.delimiter(), content))
            }
            other => other,
        })
        .collect()
}

fn prepend_underscore_to_self(ident: &mut Ident) {
    if ident == "self" {
        *ident = Ident::new("__self", ident.span());
    }
}
