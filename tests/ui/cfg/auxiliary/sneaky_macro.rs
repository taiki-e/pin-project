// force-host
// no-prefer-dynamic

#![crate_type = "proc-macro"]

extern crate proc_macro;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

fn op(c: char) -> TokenTree {
    Punct::new(c, Spacing::Alone).into()
}

fn ident(sym: &str) -> Ident {
    Ident::new(sym, Span::call_site())
}

fn word(sym: &str) -> TokenTree {
    ident(sym).into()
}

#[proc_macro_attribute]
pub fn add_pinned_field(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut tokens: Vec<_> = input.into_iter().collect();
    if let Some(TokenTree::Group(g)) = tokens.pop() {
        let mut vec = vec![];
        vec.extend(g.stream());

        // #[pin]
        // __field: __HiddenPinnedField
        vec.push(op('#'));
        vec.push(TokenTree::Group(Group::new(Delimiter::Bracket, word("pin").into())));
        vec.push(word("__field"));
        vec.push(op(':'));
        vec.push(word("__HiddenPinnedField"));

        tokens.extend(TokenStream::from(TokenTree::Group(Group::new(
            Delimiter::Brace,
            vec.into_iter().collect(),
        ))));
        let mut vec = vec![];

        // pub struct __HiddenPinnedField;
        vec.push(word("pub"));
        vec.push(word("struct"));
        vec.push(word("__HiddenPinnedField"));
        vec.push(op(';'));

        // impl !Unpin for __HiddenPinnedField {}
        vec.push(word("impl"));
        vec.push(op('!'));
        vec.push(word("Unpin"));
        vec.push(word("for"));
        vec.push(word("__HiddenPinnedField"));
        vec.push(TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())));
        tokens.extend(vec);

        tokens.into_iter().collect()
    } else {
        unreachable!()
    }
}

#[proc_macro_attribute]
pub fn hidden_repr(attr: TokenStream, item: TokenStream) -> TokenStream {
    format!("#[repr({})] {}", attr, item).parse().unwrap()
}

#[proc_macro_attribute]
pub fn hidden_repr_cfg_any(attr: TokenStream, item: TokenStream) -> TokenStream {
    format!("#[cfg_attr(any(), repr({}))] {}", attr, item).parse().unwrap()
}

#[proc_macro_attribute]
pub fn hidden_repr_cfg_not_any(attr: TokenStream, item: TokenStream) -> TokenStream {
    format!("#[cfg_attr(not(any()), repr({}))] {}", attr, item).parse().unwrap()
}
