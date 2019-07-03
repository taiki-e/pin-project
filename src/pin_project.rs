use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    visit_mut::VisitMut,
    *,
};

use crate::project::Dummy;

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse2(input)
        .and_then(|mut item| {
            syn::parse2(args).map(|args: Args| {
                // TODO: Integrate into `replace_item_fn`?.
                Dummy.visit_item_fn_mut(&mut item);
                replace_item_fn(&args.0, &mut item);
                item.into_token_stream()
            })
        })
        .unwrap_or_else(|e| e.to_compile_error())
}

fn replace_item_fn(args: &[Ident], ItemFn { decl, block, .. }: &mut ItemFn) {
    decl.inputs.iter_mut().for_each(|input| match input {
        FnArg::Captured(ArgCaptured {
            pat: Pat::Ident(pat @ PatIdent { subpat: None, .. }),
            ..
        }) if args.contains(&pat.ident) => {
            let mut local = Local {
                attrs: Vec::new(),
                let_token: token::Let::default(),
                pats: Punctuated::new(),
                ty: None,
                init: None,
                semi_token: token::Semi::default(),
            };
            let (local_pat, init) = if pat.ident == "self" {
                ReplaceSelf.visit_block_mut(block);
                let mut local_pat = pat.clone();
                prepend_underscores_to_self(&mut local_pat.ident);
                (local_pat, syn::parse_quote!(self.project()))
            } else {
                let ident = &pat.ident;
                (pat.clone(), syn::parse_quote!(#ident.project()))
            };
            local.pats.push(Pat::Ident(local_pat));
            local.init = Some((token::Eq::default(), init));
            block.stmts.insert(0, Stmt::Local(local));

            if pat.by_ref.is_none() {
                pat.mutability = None;
            }
            pat.by_ref = None;
        }
        _ => {}
    })
}

struct Args(Vec<Ident>);

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Vec::new();
        let mut first = true;
        while !input.is_empty() {
            if first {
                first = false;
            } else {
                let _: Token![,] = input.parse()?;
                if input.is_empty() {
                    break;
                }
            }

            let ident = if input.peek(Token![self]) {
                let t: Token![self] = input.parse()?;
                Ident::new("self", t.span)
            } else {
                input.parse()?
            };
            if args.contains(&ident) {
                // TODO: error
            } else {
                args.push(ident);
            }
        }
        Ok(Self(args))
    }
}

// https://github.com/dtolnay/no-panic/blob/master/src/lib.rs

struct ReplaceSelf;

impl VisitMut for ReplaceSelf {
    fn visit_expr_path_mut(&mut self, i: &mut ExprPath) {
        if i.qself.is_none() && i.path.is_ident("self") {
            prepend_underscores_to_self(&mut i.path.segments[0].ident);
        }
    }

    fn visit_macro_mut(&mut self, i: &mut Macro) {
        // We can't tell in general whether `self` inside a macro invocation
        // refers to the self in the argument list or a different self
        // introduced within the macro. Heuristic: if the macro input contains
        // `fn`, then `self` is more likely to refer to something other than the
        // outer function's self argument.
        if !contains_fn(i.tts.clone()) {
            i.tts = fold_token_stream(i.tts.clone());
        }
    }

    fn visit_item_mut(&mut self, _i: &mut Item) {
        // Do nothing, as `self` now means something else.
    }
}

fn contains_fn(tts: TokenStream) -> bool {
    tts.into_iter().any(|tt| match tt {
        TokenTree::Ident(ident) => ident == "fn",
        TokenTree::Group(group) => contains_fn(group.stream()),
        _ => false,
    })
}

fn fold_token_stream(tts: TokenStream) -> TokenStream {
    tts.into_iter()
        .map(|tt| match tt {
            TokenTree::Ident(mut ident) => {
                prepend_underscores_to_self(&mut ident);
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

fn prepend_underscores_to_self(ident: &mut Ident) {
    if ident == "self" {
        *ident = Ident::new("__self", ident.span());
    }
}
