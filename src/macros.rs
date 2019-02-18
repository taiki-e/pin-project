use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{punctuated::Punctuated, token::Or, *};

use crate::utils::*;

pub(super) fn project(input: TokenStream) -> TokenStream {
    match syn::parse(input) {
        Ok(mut stmt) => {
            replace_stmt(&mut stmt);
            TokenStream::from(stmt.into_token_stream())
        }
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn replace_stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::Expr(expr) => replace_expr(expr, &mut Register::default()),
        Stmt::Local(local) => replace_local(local, &mut Register::default()),
        Stmt::Item(Item::Fn(item)) => visitor::dummy(item),
        _ => {}
    }
}

fn replace_local(local: &mut Local, register: &mut Register) {
    replace_pats(&mut local.pats, register);
}

fn replace_expr(expr: &mut Expr, register: &mut Register) {
    match expr {
        Expr::If(expr) => expr_if(expr, register),
        Expr::ForLoop(ExprForLoop { pat, .. }) => replace_pat(&mut **pat, register),
        Expr::Let(ExprLet { pats, .. }) => replace_pats(pats, register),
        Expr::Match(ExprMatch { arms, .. }) => arms
            .iter_mut()
            .for_each(|Arm { pats, .. }| replace_pats(pats, register)),

        Expr::Block(ExprBlock { block, .. }) | Expr::Unsafe(ExprUnsafe { block, .. }) => {
            if let Some(Stmt::Expr(expr)) = block.stmts.last_mut() {
                replace_expr(expr, register);
            }
        }

        Expr::While(ExprWhile { cond: expr, .. })
        | Expr::Type(ExprType { expr, .. })
        | Expr::Paren(ExprParen { expr, .. })
        | Expr::Reference(ExprReference { expr, .. }) => replace_expr(&mut **expr, register),

        Expr::Path(ExprPath {
            qself: None, path, ..
        })
        | Expr::Struct(ExprStruct { path, .. }) => replace_path(path, register),
        _ => {}
    }
}

fn expr_if(expr: &mut ExprIf, register: &mut Register) {
    replace_expr(&mut *expr.cond, register);
    if let Some(Expr::If(expr)) = expr.else_branch.as_mut().map(|(_, expr)| &mut **expr) {
        expr_if(expr, register);
    }
}

fn replace_pats(pats: &mut Punctuated<Pat, Or>, register: &mut Register) {
    pats.iter_mut().for_each(|pat| replace_pat(pat, register));
}

fn replace_pat(pat: &mut Pat, register: &mut Register) {
    match pat {
        Pat::Ident(PatIdent {
            subpat: Some((_, pat)),
            ..
        })
        | Pat::Ref(PatRef { pat, .. })
        | Pat::Box(PatBox { pat, .. }) => replace_pat(pat, register),

        Pat::Struct(PatStruct { path, .. })
        | Pat::TupleStruct(PatTupleStruct { path, .. })
        | Pat::Path(PatPath { qself: None, path }) => replace_path(path, register),
        _ => {}
    }
}

fn replace_path(path: &mut Path, register: &mut Register) {
    fn is_none(args: &PathArguments) -> bool {
        match args {
            PathArguments::None => true,
            _ => false,
        }
    }

    fn replace_ident(ident: &mut Ident) {
        *ident = proj_ident(&ident);
    }

    let len = match path.segments.len() {
        // structs
        1 if is_none(&path.segments[0].arguments) => 1,
        // enums
        2 if is_none(&path.segments[0].arguments) && is_none(&path.segments[1].arguments) => 2,
        _ => return,
    };

    if register.0.is_none() || register.eq(&path.segments[0].ident, len) {
        register.update(&path.segments[0].ident, len);
        replace_ident(&mut path.segments[0].ident);
    }
}

struct Register(Option<(String, usize)>);

impl Register {
    fn update(&mut self, ident: &Ident, len: usize) {
        if self.0.is_none() {
            self.0 = Some((ident.to_string(), len));
        }
    }

    fn eq(&self, ident: &Ident, len: usize) -> bool {
        match &self.0 {
            Some((i, l)) => *l == len && ident == i,
            None => false,
        }
    }
}

impl Default for Register {
    fn default() -> Self {
        Self(None)
    }
}

mod visitor {
    use super::*;
    use syn::visit_mut::{self, VisitMut};

    const NAMES: &[&str] = &["project"];

    pub(super) fn dummy(item: &mut ItemFn) {
        Dummy.visit_item_fn_mut(item)
    }

    struct Dummy;

    impl VisitMut for Dummy {
        fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
            visit_mut::visit_stmt_mut(self, stmt);
            visit_stmt_mut(stmt);
        }

        // Stop at item bounds
        fn visit_item_mut(&mut self, _item: &mut Item) {}
    }

    fn visit_stmt_mut(stmt: &mut Stmt) {
        fn parse_attr<A: AttrsMut, F: FnOnce(&mut A, &mut Register)>(attrs: &mut A, f: F) {
            if attrs.find_remove().is_some() {
                f(attrs, &mut Register::default());
            }
        }

        match stmt {
            Stmt::Expr(expr) => parse_attr(expr, replace_expr),
            Stmt::Local(local) => parse_attr(local, replace_local),
            _ => {}
        }
    }

    trait AttrsMut {
        fn attrs_mut<T, F: FnOnce(&mut Vec<Attribute>) -> T>(&mut self, f: F) -> T;

        fn find_remove(&mut self) -> Option<Attribute> {
            fn find_remove(attrs: &mut Vec<Attribute>) -> Option<Attribute> {
                attrs
                    .iter()
                    .position(|Attribute { path, tts, .. }| {
                        NAMES.iter().any(|i| path.is_ident(i)) && tts.is_empty()
                    })
                    .map(|i| remove(attrs, i))
            }

            self.attrs_mut(find_remove)
        }
    }

    impl<A: AttrsMut> AttrsMut for &'_ mut A {
        fn attrs_mut<T, F: FnOnce(&mut Vec<Attribute>) -> T>(&mut self, f: F) -> T {
            (**self).attrs_mut(f)
        }
    }

    impl AttrsMut for Vec<Attribute> {
        fn attrs_mut<T, F: FnOnce(&mut Vec<Attribute>) -> T>(&mut self, f: F) -> T {
            f(self)
        }
    }

    impl AttrsMut for Local {
        fn attrs_mut<T, F: FnOnce(&mut Vec<Attribute>) -> T>(&mut self, f: F) -> T {
            f(&mut self.attrs)
        }
    }

    macro_rules! attrs_impl {
        ($($Expr:ident),*) => {
            impl AttrsMut for Expr {
                fn attrs_mut<T, F: FnOnce(&mut Vec<Attribute>) -> T>(&mut self, f: F) -> T {
                    match self {
                        $(Expr::$Expr(expr) => f(&mut expr.attrs),)*
                        Expr::Verbatim(_) => f(&mut Vec::with_capacity(0)),
                    }
                }
            }
        };
    }

    attrs_impl! {
        Box,
        InPlace,
        Array,
        Call,
        MethodCall,
        Tuple,
        Binary,
        Unary,
        Lit,
        Cast,
        Type,
        Let,
        If,
        While,
        ForLoop,
        Loop,
        Match,
        Closure,
        Unsafe,
        Block,
        Assign,
        AssignOp,
        Field,
        Index,
        Range,
        Path,
        Reference,
        Break,
        Continue,
        Return,
        Macro,
        Struct,
        Repeat,
        Paren,
        Group,
        Try,
        Async,
        TryBlock,
        Yield
    }
}
