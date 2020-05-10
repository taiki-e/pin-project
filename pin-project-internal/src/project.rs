use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    visit_mut::{self, VisitMut},
    *,
};

use crate::utils::{
    determine_lifetime_name, insert_lifetime, parse_as_empty, Immutable, Mutability, Mutable,
    Owned, SliceExt, VecExt,
};

pub(crate) fn attribute(args: &TokenStream, input: Stmt, mutability: Mutability) -> TokenStream {
    parse_as_empty(args)
        .and_then(|()| parse(input, mutability))
        .unwrap_or_else(|e| e.to_compile_error())
}

fn replace_expr(expr: &mut Expr, mutability: Mutability) {
    match expr {
        Expr::Match(expr) => {
            Context::new(mutability).replace_expr_match(expr);
        }
        Expr::If(expr_if) => {
            let mut expr_if = expr_if;
            while let Expr::Let(ref mut expr) = &mut *expr_if.cond {
                Context::new(mutability).replace_expr_let(expr);
                if let Some((_, ref mut expr)) = expr_if.else_branch {
                    if let Expr::If(new_expr_if) = &mut **expr {
                        expr_if = new_expr_if;
                        continue;
                    }
                }
                break;
            }
        }
        _ => {}
    }
}

fn parse(mut stmt: Stmt, mutability: Mutability) -> Result<TokenStream> {
    match &mut stmt {
        Stmt::Expr(expr) | Stmt::Semi(expr, _) => replace_expr(expr, mutability),
        Stmt::Local(local) => Context::new(mutability).replace_local(local)?,
        Stmt::Item(Item::Fn(item)) => replace_item_fn(item, mutability)?,
        Stmt::Item(Item::Impl(item)) => replace_item_impl(item, mutability)?,
        Stmt::Item(Item::Use(item)) => replace_item_use(item, mutability)?,
        _ => {}
    }

    Ok(stmt.into_token_stream())
}

struct Context {
    register: Option<(Ident, usize)>,
    replaced: bool,
    mutability: Mutability,
}

impl Context {
    fn new(mutability: Mutability) -> Self {
        Self { register: None, replaced: false, mutability }
    }

    fn update(&mut self, ident: &Ident, len: usize) {
        if self.register.is_none() {
            self.register = Some((ident.clone(), len));
        }
    }

    fn compare_paths(&self, ident: &Ident, len: usize) -> bool {
        match &self.register {
            Some((i, l)) => *l == len && i == ident,
            None => false,
        }
    }

    fn replace_local(&mut self, local: &mut Local) -> Result<()> {
        if let Some(attr) = local.attrs.find(self.mutability.method_name()) {
            return Err(error!(attr, "duplicate #[{}] attribute", self.mutability.method_name()));
        }

        if let Some(Expr::Match(expr)) = local.init.as_mut().map(|(_, expr)| &mut **expr) {
            self.replace_expr_match(expr);
        }

        if self.replaced {
            if is_replaceable(&local.pat, false) {
                return Err(error!(
                    local.pat,
                    "Both initializer expression and pattern are replaceable, \
                     you need to split the initializer expression into separate let bindings \
                     to avoid ambiguity"
                ));
            }
        } else {
            self.replace_pat(&mut local.pat, false);
        }

        Ok(())
    }

    fn replace_expr_let(&mut self, expr: &mut ExprLet) {
        self.replace_pat(&mut expr.pat, true)
    }

    fn replace_expr_match(&mut self, expr: &mut ExprMatch) {
        expr.arms.iter_mut().for_each(|arm| self.replace_pat(&mut arm.pat, true))
    }

    fn replace_pat(&mut self, pat: &mut Pat, allow_pat_path: bool) {
        match pat {
            Pat::Ident(PatIdent { subpat: Some((_, pat)), .. })
            | Pat::Reference(PatReference { pat, .. })
            | Pat::Box(PatBox { pat, .. })
            | Pat::Type(PatType { pat, .. }) => self.replace_pat(pat, allow_pat_path),

            Pat::Or(PatOr { cases, .. }) => {
                cases.iter_mut().for_each(|pat| self.replace_pat(pat, allow_pat_path))
            }

            Pat::Struct(PatStruct { path, .. }) | Pat::TupleStruct(PatTupleStruct { path, .. }) => {
                self.replace_path(path)
            }
            Pat::Path(PatPath { qself: None, path, .. }) if allow_pat_path => {
                self.replace_path(path)
            }
            _ => {}
        }
    }

    fn replace_path(&mut self, path: &mut Path) {
        let len = match path.segments.len() {
            // 1: struct
            // 2: enum
            len @ 1 | len @ 2 => len,
            // other path
            _ => return,
        };

        if self.register.is_none() || self.compare_paths(&path.segments[0].ident, len) {
            self.update(&path.segments[0].ident, len);
            self.replaced = true;
            replace_ident(&mut path.segments[0].ident, self.mutability);
        }
    }
}

fn is_replaceable(pat: &Pat, allow_pat_path: bool) -> bool {
    match pat {
        Pat::Ident(PatIdent { subpat: Some((_, pat)), .. })
        | Pat::Reference(PatReference { pat, .. })
        | Pat::Box(PatBox { pat, .. })
        | Pat::Type(PatType { pat, .. }) => is_replaceable(pat, allow_pat_path),

        Pat::Or(PatOr { cases, .. }) => cases.iter().any(|pat| is_replaceable(pat, allow_pat_path)),

        Pat::Struct(_) | Pat::TupleStruct(_) => true,
        Pat::Path(PatPath { qself: None, .. }) => allow_pat_path,
        _ => false,
    }
}

fn replace_ident(ident: &mut Ident, mutability: Mutability) {
    *ident = mutability.proj_ident(ident);
}

fn replace_item_impl(item: &mut ItemImpl, mutability: Mutability) -> Result<()> {
    if let Some(attr) = item.attrs.find(mutability.method_name()) {
        return Err(error!(attr, "duplicate #[{}] attribute", mutability.method_name()));
    }

    let PathSegment { ident, arguments } = match &mut *item.self_ty {
        Type::Path(TypePath { qself: None, path }) => path.segments.last_mut().unwrap(),
        _ => return Ok(()),
    };

    replace_ident(ident, mutability);

    let mut lifetime_name = String::from("'pin");
    determine_lifetime_name(&mut lifetime_name, &mut item.generics);
    item.items
        .iter_mut()
        .filter_map(|i| if let ImplItem::Method(i) = i { Some(i) } else { None })
        .for_each(|item| determine_lifetime_name(&mut lifetime_name, &mut item.sig.generics));
    let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

    insert_lifetime(&mut item.generics, lifetime.clone());

    match arguments {
        PathArguments::None => {
            *arguments = PathArguments::AngleBracketed(syn::parse_quote!(<#lifetime>));
        }
        PathArguments::AngleBracketed(args) => {
            args.args.insert(0, syn::parse_quote!(#lifetime));
        }
        PathArguments::Parenthesized(_) => unreachable!(),
    }
    Ok(())
}

fn replace_item_fn(item: &mut ItemFn, mutability: Mutability) -> Result<()> {
    struct FnVisitor(Result<()>);

    impl FnVisitor {
        fn visit_stmt(&mut self, node: &mut Stmt) -> Result<()> {
            match node {
                Stmt::Expr(expr) | Stmt::Semi(expr, _) => self.visit_expr(expr),
                Stmt::Local(local) => {
                    visit_mut::visit_local_mut(self, local);

                    let mut prev = None;
                    for &mutability in &[Immutable, Mutable, Owned] {
                        if let Some(attr) = local.attrs.find_remove(mutability.method_name())? {
                            if let Some(prev) = prev.replace(mutability) {
                                return Err(error!(
                                    attr,
                                    "attributes `{}` and `{}` are mutually exclusive",
                                    prev.method_name(),
                                    mutability.method_name(),
                                ));
                            }
                            Context::new(mutability).replace_local(local)?;
                        }
                    }

                    Ok(())
                }
                // Do not recurse into nested items.
                Stmt::Item(_) => Ok(()),
            }
        }

        fn visit_expr(&mut self, node: &mut Expr) -> Result<()> {
            visit_mut::visit_expr_mut(self, node);
            match node {
                Expr::Match(expr) => {
                    let mut prev = None;
                    for &mutability in &[Immutable, Mutable, Owned] {
                        if let Some(attr) = expr.attrs.find_remove(mutability.method_name())? {
                            if let Some(prev) = prev.replace(mutability) {
                                return Err(error!(
                                    attr,
                                    "attributes `{}` and `{}` are mutually exclusive",
                                    prev.method_name(),
                                    mutability.method_name(),
                                ));
                            }
                        }
                    }
                    if let Some(mutability) = prev {
                        replace_expr(node, mutability);
                    }
                }
                Expr::If(expr_if) => {
                    if let Expr::Let(_) = &*expr_if.cond {
                        let mut prev = None;
                        for &mutability in &[Immutable, Mutable, Owned] {
                            if let Some(attr) =
                                expr_if.attrs.find_remove(mutability.method_name())?
                            {
                                if let Some(prev) = prev.replace(mutability) {
                                    return Err(error!(
                                        attr,
                                        "attributes `{}` and `{}` are mutually exclusive",
                                        prev.method_name(),
                                        mutability.method_name(),
                                    ));
                                }
                            }
                        }
                        if let Some(mutability) = prev {
                            replace_expr(node, mutability);
                        }
                    }
                }
                _ => {}
            }
            Ok(())
        }
    }

    impl VisitMut for FnVisitor {
        fn visit_stmt_mut(&mut self, node: &mut Stmt) {
            if self.0.is_err() {
                return;
            }
            if let Err(e) = self.visit_stmt(node) {
                self.0 = Err(e)
            }
        }

        fn visit_expr_mut(&mut self, node: &mut Expr) {
            if self.0.is_err() {
                return;
            }
            if let Err(e) = self.visit_expr(node) {
                self.0 = Err(e)
            }
        }

        fn visit_item_mut(&mut self, _: &mut Item) {
            // Do not recurse into nested items.
        }
    }

    if let Some(attr) = item.attrs.find(mutability.method_name()) {
        return Err(error!(attr, "duplicate #[{}] attribute", mutability.method_name()));
    }

    let mut visitor = FnVisitor(Ok(()));
    visitor.visit_block_mut(&mut item.block);
    visitor.0
}

fn replace_item_use(item: &mut ItemUse, mutability: Mutability) -> Result<()> {
    struct UseTreeVisitor {
        res: Result<()>,
        mutability: Mutability,
    }

    impl VisitMut for UseTreeVisitor {
        fn visit_use_tree_mut(&mut self, node: &mut UseTree) {
            if self.res.is_err() {
                return;
            }

            match node {
                // Desugar `use tree::<name>` into `tree::__<name>Projection`.
                UseTree::Name(name) => replace_ident(&mut name.ident, self.mutability),
                UseTree::Glob(glob) => {
                    self.res = Err(error!(
                        glob,
                        "#[{}] attribute may not be used on glob imports",
                        self.mutability.method_name()
                    ));
                }
                UseTree::Rename(rename) => {
                    self.res = Err(error!(
                        rename,
                        "#[{}] attribute may not be used on renamed imports",
                        self.mutability.method_name()
                    ));
                }
                UseTree::Path(_) | UseTree::Group(_) => visit_mut::visit_use_tree_mut(self, node),
            }
        }
    }

    if let Some(attr) = item.attrs.find(mutability.method_name()) {
        return Err(error!(attr, "duplicate #[{}] attribute", mutability.method_name()));
    }

    let mut visitor = UseTreeVisitor { res: Ok(()), mutability };
    visitor.visit_item_use_mut(item);
    visitor.res
}
