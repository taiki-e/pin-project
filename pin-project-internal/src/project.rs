use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    token::Or,
    visit_mut::{self, VisitMut},
    *,
};

use crate::utils::{proj_ident, VecExt};

/// The attribute name.
const NAME: &str = "project";

pub(super) fn attribute(input: TokenStream) -> TokenStream {
    parse(input).unwrap_or_else(|e| e.to_compile_error())
}

fn parse(input: TokenStream) -> Result<TokenStream> {
    fn replace_stmt(stmt: &mut Stmt) {
        match stmt {
            Stmt::Expr(expr) => expr.replace(&mut Register::default()),
            Stmt::Local(local) => local.replace(&mut Register::default()),
            Stmt::Item(Item::Fn(item)) => Dummy.visit_item_fn_mut(item),
            _ => {}
        }
    }

    syn::parse2(input).map(|mut stmt| {
        replace_stmt(&mut stmt);
        stmt.into_token_stream()
    })
}

trait Replace {
    /// Replace the original ident with the ident of projected type.
    fn replace(&mut self, register: &mut Register);
}

impl Replace for Local {
    fn replace(&mut self, register: &mut Register) {
        self.pats.replace(register);
    }
}

impl Replace for Expr {
    fn replace(&mut self, register: &mut Register) {
        match self {
            Expr::If(expr) => expr.replace(register),
            Expr::ForLoop(ExprForLoop { pat, .. }) => pat.replace(register),
            Expr::Let(ExprLet { pats, .. }) => pats.replace(register),

            Expr::Match(ExprMatch { arms, .. }) => {
                arms.iter_mut().for_each(|Arm { pats, .. }| pats.replace(register))
            }

            Expr::Block(ExprBlock { block, .. }) | Expr::Unsafe(ExprUnsafe { block, .. }) => {
                if let Some(Stmt::Expr(expr)) = block.stmts.last_mut() {
                    expr.replace(register);
                }
            }

            Expr::While(ExprWhile { cond: expr, .. })
            | Expr::Type(ExprType { expr, .. })
            | Expr::Paren(ExprParen { expr, .. })
            | Expr::Reference(ExprReference { expr, .. }) => expr.replace(register),

            Expr::Path(ExprPath { qself: None, path, .. })
            | Expr::Struct(ExprStruct { path, .. }) => path.replace(register),

            _ => {}
        }
    }
}

impl Replace for ExprIf {
    fn replace(&mut self, register: &mut Register) {
        self.cond.replace(register);

        if let Some(Expr::If(expr)) = self.else_branch.as_mut().map(|(_, expr)| &mut **expr) {
            expr.replace(register);
        }
    }
}

impl Replace for Punctuated<Pat, Or> {
    fn replace(&mut self, register: &mut Register) {
        self.iter_mut().for_each(|pat| pat.replace(register));
    }
}

impl Replace for Pat {
    fn replace(&mut self, register: &mut Register) {
        match self {
            Pat::Ident(PatIdent { subpat: Some((_, pat)), .. })
            | Pat::Ref(PatRef { pat, .. })
            | Pat::Box(PatBox { pat, .. }) => pat.replace(register),

            Pat::Struct(PatStruct { path, .. })
            | Pat::TupleStruct(PatTupleStruct { path, .. })
            | Pat::Path(PatPath { qself: None, path }) => path.replace(register),

            _ => {}
        }
    }
}

impl Replace for Path {
    fn replace(&mut self, register: &mut Register) {
        fn is_none(args: &PathArguments) -> bool {
            match args {
                PathArguments::None => true,
                _ => false,
            }
        }

        fn replace_ident(ident: &mut Ident) {
            *ident = proj_ident(ident);
        }

        let len = match self.segments.len() {
            // struct
            1 if is_none(&self.segments[0].arguments) => 1,
            // enum
            2 if is_none(&self.segments[0].arguments) && is_none(&self.segments[1].arguments) => 2,
            // other path
            _ => return,
        };

        if register.0.is_none() || register.eq(&self.segments[0].ident, len) {
            register.update(&self.segments[0].ident, len);
            replace_ident(&mut self.segments[0].ident);
        }
    }
}

#[derive(Default)]
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

// =================================================================================================
// visitor

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
    fn parse_attr<A: AttrsMut + Replace>(attrs: &mut A) {
        if attrs.find_remove() {
            attrs.replace(&mut Register::default());
        }
    }

    match stmt {
        Stmt::Expr(expr) => parse_attr(expr),
        Stmt::Local(local) => parse_attr(local),
        _ => {}
    }
}

trait AttrsMut {
    fn attrs_mut<T, F: FnOnce(&mut Vec<Attribute>) -> T>(&mut self, f: F) -> T;

    fn find_remove(&mut self) -> bool {
        self.attrs_mut(|attrs| attrs.find_remove(NAME))
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
