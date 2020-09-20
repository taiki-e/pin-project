use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    visit_mut::VisitMut,
    *,
};

use super::PIN;
use crate::utils::{
    determine_lifetime_name, determine_visibility, insert_lifetime_and_bound, LeadingColon,
    ParseBufferExt, ReplaceReceiver, SliceExt, Variants,
};

pub(super) fn parse_derive(input: TokenStream) -> Result<TokenStream> {
    let mut input: DeriveInput = syn::parse2(input)?;

    let ident = &input.ident;
    let ty_generics = input.generics.split_for_impl().1;
    let self_ty = parse_quote!(#ident #ty_generics);
    let mut visitor = ReplaceReceiver(&self_ty);
    visitor.visit_generics_mut(&mut input.generics);
    visitor.visit_data_mut(&mut input.data);

    let mut cx = Context::new(&input.attrs, &input.vis, &input.ident, &mut input.generics)?;
    let mut generate = GenerateTokens::default();
    let packed_check;

    match &input.data {
        Data::Struct(data) => {
            // Do this first for a better error message.
            packed_check = Some(cx.ensure_not_packed(&data.fields)?);
            cx.parse_struct(data, &mut generate)?;
        }
        Data::Enum(data) => {
            // We don't need to check for `#[repr(packed)]`,
            // since it does not apply to enums.
            packed_check = None;
            cx.parse_enum(data, &mut generate)?;
        }
        Data::Union(_) => {
            return Err(error!(
                input,
                "#[pin_project] attribute may only be used on structs or enums"
            ));
        }
    }

    Ok(generate.into_tokens(&cx, packed_check))
}

fn validate_struct(ident: &Ident, fields: &Fields) -> Result<()> {
    if fields.is_empty() {
        let msg = "#[pin_project] attribute may not be used on structs with zero fields";
        if let Fields::Unit = fields { Err(error!(ident, msg)) } else { Err(error!(fields, msg)) }
    } else {
        Ok(())
    }
}

fn validate_enum(brace_token: token::Brace, variants: &Variants) -> Result<()> {
    if variants.is_empty() {
        return Err(Error::new(
            brace_token.span,
            "#[pin_project] attribute may not be used on enums without variants",
        ));
    }
    let has_field = variants.iter().try_fold(false, |has_field, v| {
        if let Some((_, e)) = &v.discriminant {
            Err(error!(e, "#[pin_project] attribute may not be used on enums with discriminants"))
        } else if let Some(attr) = v.attrs.find(PIN) {
            Err(error!(attr, "#[pin] attribute may only be used on fields of structs or variants"))
        } else if v.fields.is_empty() {
            Ok(has_field)
        } else {
            Ok(true)
        }
    })?;
    if has_field {
        Ok(())
    } else {
        Err(error!(variants, "#[pin_project] attribute may not be used on enums with zero fields"))
    }
}

#[derive(Default)]
struct GenerateTokens {
    exposed: TokenStream,
    scoped: TokenStream,
}

impl GenerateTokens {
    fn extend(&mut self, expose: bool, tokens: TokenStream) {
        if expose {
            self.exposed.extend(tokens);
        } else {
            self.scoped.extend(tokens);
        }
    }

    fn into_tokens(self, cx: &Context<'_>, packed_check: Option<TokenStream>) -> TokenStream {
        let mut tokens = self.exposed;
        let scoped = self.scoped;

        let unpin_impl = cx.make_unpin_impl();
        let drop_impl = cx.make_drop_impl();

        let dummy_const = if cfg!(underscore_consts) {
            format_ident!("_")
        } else {
            format_ident!("__SCOPE_{}", cx.orig.ident)
        };
        let allowed_lints = global_allowed_lints();

        tokens.extend(quote! {
            // All items except projected types are generated inside a `const` scope.
            // This makes it impossible for user code to refer to these types.
            // However, this prevents Rustdoc from displaying docs for any
            // of our types. In particular, users cannot see the
            // automatically generated `Unpin` impl for the '__UnpinStruct' types
            //
            // Previously, we provided a flag to correctly document the
            // automatically generated `Unpin` impl by using def-site hygiene,
            // but it is now removed.
            //
            // Refs:
            // * https://github.com/rust-lang/rust/issues/63281
            // * https://github.com/taiki-e/pin-project/pull/53#issuecomment-525906867
            // * https://github.com/taiki-e/pin-project/pull/70
            #[doc(hidden)] // for Rust 1.34 - 1.41
            #[allow(non_upper_case_globals)] // for Rust 1.34 - 1.36
            #[allow(clippy::used_underscore_binding)]
            #allowed_lints
            const #dummy_const: () = {
                #scoped
                #unpin_impl
                #drop_impl
                #packed_check
            };
        });
        tokens
    }
}

/// Returns attributes that should be applied to all generated code.
fn global_allowed_lints() -> TokenStream {
    quote! {
        #[allow(box_pointers)] // This lint warns use of the `Box` type.
        #[allow(explicit_outlives_requirements)] // https://github.com/rust-lang/rust/issues/60993
        #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
        #[allow(clippy::pattern_type_mismatch)]
    }
}

/// Returns attributes used on projected types.
fn proj_allowed_lints(is_enum: bool) -> (TokenStream, TokenStream, TokenStream) {
    let large_enum_variant =
        if is_enum { Some(quote!(#[allow(clippy::large_enum_variant)])) } else { None };
    let global_allowed_lints = global_allowed_lints();
    let proj_mut = quote! {
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326}
        #global_allowed_lints
    };
    let proj_ref = quote! {
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
        #global_allowed_lints
    };
    let proj_own = quote! {
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(unreachable_pub)] // This lint warns `pub` field in private struct.
        #large_enum_variant
        #global_allowed_lints
    };
    (proj_mut, proj_ref, proj_own)
}

struct Args {
    /// `PinnedDrop` argument.
    pinned_drop: Option<Span>,
    /// `UnsafeUnpin` or `!Unpin` argument.
    unpin_impl: UnpinImpl,
    /// `project = <ident>` argument.
    project: Option<Ident>,
    /// `project_ref = <ident>` argument.
    project_ref: Option<Ident>,
    /// `project_replace [= <ident>]` argument.
    project_replace: ProjReplace,
}

impl Args {
    fn get(attrs: &[Attribute]) -> Result<Self> {
        // `(__private(<args>))` -> `<args>`
        struct Input(Option<TokenStream>);

        impl Parse for Input {
            fn parse(input: ParseStream<'_>) -> Result<Self> {
                Ok(Self((|| {
                    let content = input.parenthesized().ok()?;
                    let private = content.parse::<Ident>().ok()?;
                    if private == "__private" {
                        content.parenthesized().ok()?.parse::<TokenStream>().ok()
                    } else {
                        None
                    }
                })()))
            }
        }

        if let Some(attr) = attrs.find("pin_project") {
            return Err(error!(attr, "duplicate #[pin_project] attribute"));
        }

        let mut attrs = attrs.iter().filter(|attr| attr.path.is_ident(PIN));

        let prev = if let Some(attr) = attrs.next() {
            (attr, syn::parse2::<Input>(attr.tokens.clone()).unwrap().0)
        } else {
            // This only fails if another macro removes `#[pin]`.
            return Err(error!(TokenStream::new(), "#[pin_project] attribute has been removed"));
        };

        if let Some(attr) = attrs.next() {
            let (prev_attr, prev_res) = &prev;
            // As the `#[pin]` attribute generated by `#[pin_project]`
            // has the same span as `#[pin_project]`, it is possible
            // that a useless error message will be generated.
            let res = syn::parse2::<Input>(attr.tokens.clone()).unwrap().0;
            let span = match (prev_res, res) {
                (Some(_), _) => attr,
                (_, Some(_)) => prev_attr,
                (None, None) => prev_attr,
            };
            Err(error!(span, "duplicate #[pin] attribute"))
        } else {
            // This `unwrap` only fails if another macro removes `#[pin]` and inserts own `#[pin]`.
            syn::parse2(prev.1.unwrap())
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        mod kw {
            syn::custom_keyword!(Unpin);
        }

        /// Parses `= <value>` in `<name> = <value>` and returns value and span of name-value pair.
        fn parse_value(
            input: ParseStream<'_>,
            name: &Ident,
            has_prev: bool,
        ) -> Result<(Ident, TokenStream)> {
            if input.is_empty() {
                return Err(error!(name, "expected `{0} = <identifier>`, found `{0}`", name));
            }
            let eq_token: Token![=] = input.parse()?;
            if input.is_empty() {
                let span = quote!(#name #eq_token);
                return Err(error!(span, "expected `{0} = <identifier>`, found `{0} =`", name));
            }
            let value: Ident = input.parse()?;
            let span = quote!(#name #value);
            if has_prev {
                Err(error!(span, "duplicate `{}` argument", name))
            } else {
                Ok((value, span))
            }
        }

        let mut pinned_drop = None;
        let mut unsafe_unpin = None;
        let mut not_unpin = None;
        let mut project = None;
        let mut project_ref = None;
        let mut project_replace_value = None;
        let mut project_replace_span = None;

        while !input.is_empty() {
            if input.peek(Token![!]) {
                let bang: Token![!] = input.parse()?;
                if input.is_empty() {
                    return Err(error!(bang, "expected `!Unpin`, found `!`"));
                }
                let unpin: kw::Unpin = input.parse()?;
                let span = quote!(#bang #unpin);
                if not_unpin.replace(span.span()).is_some() {
                    return Err(error!(span, "duplicate `!Unpin` argument"));
                }
            } else {
                let token = input.parse::<Ident>()?;
                match &*token.to_string() {
                    "PinnedDrop" => {
                        if pinned_drop.replace(token.span()).is_some() {
                            return Err(error!(token, "duplicate `PinnedDrop` argument"));
                        }
                    }
                    "UnsafeUnpin" => {
                        if unsafe_unpin.replace(token.span()).is_some() {
                            return Err(error!(token, "duplicate `UnsafeUnpin` argument"));
                        }
                    }
                    "project" => {
                        project = Some(parse_value(input, &token, project.is_some())?.0);
                    }
                    "project_ref" => {
                        project_ref = Some(parse_value(input, &token, project_ref.is_some())?.0);
                    }
                    "project_replace" => {
                        if input.peek(Token![=]) {
                            let (value, span) =
                                parse_value(input, &token, project_replace_span.is_some())?;
                            project_replace_value = Some(value);
                            project_replace_span = Some(span.span());
                        } else if project_replace_span.is_some() {
                            return Err(error!(token, "duplicate `project_replace` argument"));
                        } else {
                            project_replace_span = Some(token.span());
                        }
                    }
                    "Replace" => {
                        return Err(error!(
                            token,
                            "`Replace` argument was removed, use `project_replace` argument instead"
                        ));
                    }
                    _ => return Err(error!(token, "unexpected argument: {}", token)),
                }
            }

            if input.is_empty() {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        if project.is_some() || project_ref.is_some() {
            if project == project_ref {
                return Err(error!(
                    project_ref,
                    "name `{}` is already specified by `project` argument",
                    project_ref.as_ref().unwrap()
                ));
            }
            if let Some(ident) = &project_replace_value {
                if project == project_replace_value {
                    return Err(error!(
                        ident,
                        "name `{}` is already specified by `project` argument", ident
                    ));
                } else if project_ref == project_replace_value {
                    return Err(error!(
                        ident,
                        "name `{}` is already specified by `project_ref` argument", ident
                    ));
                }
            }
        }

        if let Some(span) = pinned_drop {
            if project_replace_span.is_some() {
                return Err(Error::new(
                    span,
                    "arguments `PinnedDrop` and `project_replace` are mutually exclusive",
                ));
            }
        }
        let project_replace = match (project_replace_span, project_replace_value) {
            (None, _) => ProjReplace::None,
            (Some(span), Some(ident)) => ProjReplace::Named { ident, span },
            (Some(span), None) => ProjReplace::Unnamed { span },
        };
        let unpin_impl = match (unsafe_unpin, not_unpin) {
            (None, None) => UnpinImpl::Default,
            (Some(span), None) => UnpinImpl::Unsafe(span),
            (None, Some(span)) => UnpinImpl::Negative(span),
            (Some(span), Some(_)) => {
                return Err(Error::new(
                    span,
                    "arguments `UnsafeUnpin` and `!Unpin` are mutually exclusive",
                ));
            }
        };

        Ok(Self { pinned_drop, unpin_impl, project, project_ref, project_replace })
    }
}

/// `UnsafeUnpin` or `!Unpin` argument.
#[derive(Clone, Copy)]
enum UnpinImpl {
    Default,
    /// `UnsafeUnpin`.
    Unsafe(Span),
    /// `!Unpin`.
    Negative(Span),
}

/// `project_replace [= <ident>]` argument.
enum ProjReplace {
    None,
    /// `project_replace`.
    Unnamed {
        span: Span,
    },
    /// `project_replace = <ident>`.
    Named {
        span: Span,
        ident: Ident,
    },
}

impl ProjReplace {
    /// Return the span of this argument.
    fn span(&self) -> Option<Span> {
        match self {
            ProjReplace::None => None,
            ProjReplace::Named { span, .. } | ProjReplace::Unnamed { span, .. } => Some(*span),
        }
    }

    fn ident(&self) -> Option<&Ident> {
        if let ProjReplace::Named { ident, .. } = self { Some(ident) } else { None }
    }
}

struct OriginalType<'a> {
    /// Attributes of the original type.
    attrs: &'a [Attribute],
    /// Visibility of the original type.
    vis: &'a Visibility,
    /// Name of the original type.
    ident: &'a Ident,
    /// Generics of the original type.
    generics: &'a Generics,
}

struct ProjectedType {
    /// Visibility of the projected types.
    vis: Visibility,
    /// Name of the projected type returned by `project` method.
    mut_ident: Ident,
    /// Name of the projected type returned by `project_ref` method.
    ref_ident: Ident,
    /// Name of the projected type returned by `project_replace` method.
    own_ident: Ident,
    /// Lifetime on the generated projected types.
    lifetime: Lifetime,
    /// Generics of the projected types.
    generics: Generics,
    /// `where` clause of the projected types. This has an additional
    /// bound generated by `insert_lifetime_and_bound`
    where_clause: WhereClause,
}

struct ProjectedVariants {
    proj_variants: TokenStream,
    proj_ref_variants: TokenStream,
    proj_own_variants: TokenStream,
    proj_arms: TokenStream,
    proj_ref_arms: TokenStream,
    proj_own_arms: TokenStream,
}

struct ProjectedFields {
    proj_pat: TokenStream,
    proj_body: TokenStream,
    proj_own_body: TokenStream,
    proj_fields: Option<Group>,
    proj_ref_fields: Option<Group>,
    proj_own_fields: Option<Group>,
}

impl ProjectedFields {
    fn unit(cx: &Context<'_>, variant: LeadingColon<'_>) -> Self {
        let orig_ident = cx.orig.ident;
        Self {
            proj_pat: quote!(#orig_ident #variant),
            proj_body: variant.to_token_stream(),
            proj_own_body: cx.proj_own_body(Some(variant), None, &[]),
            proj_fields: None,
            proj_ref_fields: None,
            proj_own_fields: None,
        }
    }
}

struct Context<'a> {
    /// The original type.
    orig: OriginalType<'a>,
    /// The projected types.
    proj: ProjectedType,
    /// Types of the pinned fields.
    pinned_fields: Vec<Type>,

    /// `PinnedDrop` argument.
    pinned_drop: Option<Span>,
    /// `UnsafeUnpin` or `!Unpin` argument.
    unpin_impl: UnpinImpl,
    /// `project` argument.
    project: bool,
    /// `project_ref` argument.
    project_ref: bool,
    /// `project_replace [= <ident>]` argument.
    project_replace: ProjReplace,
}

impl<'a> Context<'a> {
    fn new(
        attrs: &'a [Attribute],
        vis: &'a Visibility,
        ident: &'a Ident,
        generics: &'a mut Generics,
    ) -> Result<Self> {
        let Args { pinned_drop, unpin_impl, project, project_ref, project_replace } =
            Args::get(attrs)?;

        if let Some(name) = [project.as_ref(), project_ref.as_ref(), project_replace.ident()]
            .iter()
            .filter_map(Option::as_ref)
            .find(|name| **name == ident)
        {
            return Err(error!(name, "name `{}` is the same as the original type name", name));
        }

        let mut lifetime_name = String::from("'pin");
        determine_lifetime_name(&mut lifetime_name, generics);
        let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

        let ty_generics = generics.split_for_impl().1;
        let ty_generics_as_generics = parse_quote!(#ty_generics);
        let mut proj_generics = generics.clone();
        let pred = insert_lifetime_and_bound(
            &mut proj_generics,
            lifetime.clone(),
            &ty_generics_as_generics,
            ident,
        );
        let mut where_clause = generics.make_where_clause().clone();
        where_clause.predicates.push(pred);

        let own_ident = project_replace
            .ident()
            .cloned()
            .unwrap_or_else(|| format_ident!("__{}ProjectionOwned", ident));

        Ok(Self {
            pinned_drop,
            unpin_impl,
            project: project.is_some(),
            project_ref: project_ref.is_some(),
            project_replace,
            proj: ProjectedType {
                vis: determine_visibility(vis),
                mut_ident: project.unwrap_or_else(|| format_ident!("__{}Projection", ident)),
                ref_ident: project_ref.unwrap_or_else(|| format_ident!("__{}ProjectionRef", ident)),
                own_ident,
                lifetime,
                generics: proj_generics,
                where_clause,
            },
            orig: OriginalType { attrs, vis, ident, generics },
            pinned_fields: Vec::new(),
        })
    }

    fn parse_struct(
        &mut self,
        DataStruct { fields, .. }: &DataStruct,
        generate: &mut GenerateTokens,
    ) -> Result<()> {
        validate_struct(self.orig.ident, fields)?;

        let ProjectedFields {
            proj_pat,
            proj_body,
            proj_fields,
            proj_ref_fields,
            proj_own_fields,
            proj_own_body,
        } = match fields {
            Fields::Named(_) => self.visit_fields(None, fields, Delimiter::Brace)?,
            Fields::Unnamed(_) => self.visit_fields(None, fields, Delimiter::Parenthesis)?,
            Fields::Unit => unreachable!(),
        };

        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let proj_own_ident = &self.proj.own_ident;
        let vis = &self.proj.vis;
        let mut orig_generics = self.orig.generics.clone();
        let orig_where_clause = orig_generics.where_clause.take();
        let proj_generics = &self.proj.generics;
        let proj_where_clause = &self.proj.where_clause;

        // For tuple structs, we need to generate `(T1, T2) where Foo: Bar`
        // For non-tuple structs, we need to generate `where Foo: Bar { field1: T }`
        let (where_clause_fields, where_clause_ref_fields, where_clause_own_fields) = match fields {
            Fields::Named(_) => (
                quote!(#proj_where_clause #proj_fields),
                quote!(#proj_where_clause #proj_ref_fields),
                quote!(#orig_where_clause #proj_own_fields),
            ),
            Fields::Unnamed(_) => (
                quote!(#proj_fields #proj_where_clause;),
                quote!(#proj_ref_fields #proj_where_clause;),
                quote!(#proj_own_fields #orig_where_clause;),
            ),
            Fields::Unit => unreachable!(),
        };

        let (proj_attrs, proj_ref_attrs, proj_own_attrs) = proj_allowed_lints(false);
        generate.extend(self.project, quote! {
            #proj_attrs
            #vis struct #proj_ident #proj_generics #where_clause_fields
        });
        generate.extend(self.project_ref, quote! {
            #proj_ref_attrs
            #vis struct #proj_ref_ident #proj_generics #where_clause_ref_fields
        });
        if self.project_replace.span().is_some() {
            generate.extend(self.project_replace.ident().is_some(), quote! {
                #proj_own_attrs
                #vis struct #proj_own_ident #orig_generics #where_clause_own_fields
            });
        }

        let proj_mut_body = quote! {
            let #proj_pat = self.get_unchecked_mut();
            #proj_ident #proj_body
        };
        let proj_ref_body = quote! {
            let #proj_pat = self.get_ref();
            #proj_ref_ident #proj_body
        };
        let proj_own_body = quote! {
            let __self_ptr: *mut Self = self.get_unchecked_mut();
            let #proj_pat = &mut *__self_ptr;
            #proj_own_body
        };
        generate.extend(
            false,
            self.make_proj_impl(false, &proj_mut_body, &proj_ref_body, &proj_own_body),
        );

        Ok(())
    }

    fn parse_enum(
        &mut self,
        DataEnum { brace_token, variants, .. }: &DataEnum,
        generate: &mut GenerateTokens,
    ) -> Result<()> {
        if let ProjReplace::Unnamed { span } = &self.project_replace {
            return Err(Error::new(
                *span,
                "`project_replace` argument requires a value when used on enums",
            ));
        }

        validate_enum(*brace_token, variants)?;

        let ProjectedVariants {
            proj_variants,
            proj_ref_variants,
            proj_own_variants,
            proj_arms,
            proj_ref_arms,
            proj_own_arms,
        } = self.visit_variants(variants)?;

        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let proj_own_ident = &self.proj.own_ident;
        let vis = &self.proj.vis;
        let mut orig_generics = self.orig.generics.clone();
        let orig_where_clause = orig_generics.where_clause.take();
        let proj_generics = &self.proj.generics;
        let proj_where_clause = &self.proj.where_clause;

        let (proj_attrs, proj_ref_attrs, proj_own_attrs) = proj_allowed_lints(true);
        if self.project {
            generate.extend(true, quote! {
                #proj_attrs
                #vis enum #proj_ident #proj_generics #proj_where_clause {
                    #proj_variants
                }
            });
        }
        if self.project_ref {
            generate.extend(true, quote! {
                #proj_ref_attrs
                #vis enum #proj_ref_ident #proj_generics #proj_where_clause {
                    #proj_ref_variants
                }
            });
        }
        if self.project_replace.ident().is_some() {
            generate.extend(true, quote! {
                #proj_own_attrs
                #vis enum #proj_own_ident #orig_generics #orig_where_clause {
                    #proj_own_variants
                }
            });
        }

        let proj_mut_body = quote! {
            match self.get_unchecked_mut() {
                #proj_arms
            }
        };
        let proj_ref_body = quote! {
            match self.get_ref() {
                #proj_ref_arms
            }
        };
        let proj_own_body = quote! {
            let __self_ptr: *mut Self = self.get_unchecked_mut();
            match &mut *__self_ptr {
                #proj_own_arms
            }
        };
        generate.extend(
            false,
            self.make_proj_impl(true, &proj_mut_body, &proj_ref_body, &proj_own_body),
        );

        Ok(())
    }

    fn visit_variants(&mut self, variants: &Variants) -> Result<ProjectedVariants> {
        let mut proj_variants = TokenStream::new();
        let mut proj_ref_variants = TokenStream::new();
        let mut proj_own_variants = TokenStream::new();
        let mut proj_arms = TokenStream::new();
        let mut proj_ref_arms = TokenStream::new();
        let mut proj_own_arms = TokenStream::new();

        for Variant { ident, fields, .. } in variants {
            let variant = LeadingColon(ident);

            let ProjectedFields {
                proj_pat,
                proj_body,
                proj_fields,
                proj_ref_fields,
                proj_own_fields,
                proj_own_body,
            } = match fields {
                Fields::Named(_) => self.visit_fields(Some(variant), fields, Delimiter::Brace)?,
                Fields::Unnamed(_) => {
                    self.visit_fields(Some(variant), fields, Delimiter::Parenthesis)?
                }
                Fields::Unit => ProjectedFields::unit(self, variant),
            };

            let proj_ident = &self.proj.mut_ident;
            let proj_ref_ident = &self.proj.ref_ident;
            proj_variants.extend(quote! {
                #ident #proj_fields,
            });
            proj_ref_variants.extend(quote! {
                #ident #proj_ref_fields,
            });
            proj_own_variants.extend(quote! {
                #ident #proj_own_fields,
            });

            proj_arms.extend(quote! {
                #proj_pat => { #proj_ident #proj_body }
            });
            proj_ref_arms.extend(quote! {
                #proj_pat => { #proj_ref_ident #proj_body }
            });
            proj_own_arms.extend(quote! {
                #proj_pat => { #proj_own_body }
            });
        }

        Ok(ProjectedVariants {
            proj_variants,
            proj_ref_variants,
            proj_own_variants,
            proj_arms,
            proj_ref_arms,
            proj_own_arms,
        })
    }

    fn visit_fields(
        &mut self,
        variant: Option<LeadingColon<'_>>,
        fields: &Fields,
        delim: Delimiter,
    ) -> Result<ProjectedFields> {
        let mut proj_pat = TokenStream::new();
        let mut proj_body = TokenStream::new();
        let mut proj_fields = TokenStream::new();
        let mut proj_ref_fields = TokenStream::new();
        let mut proj_own_fields = TokenStream::new();
        let mut proj_move = TokenStream::new();
        let mut pinned_bindings = Vec::with_capacity(fields.len());

        for (i, Field { attrs, vis, ident, colon_token, ty, .. }) in fields.iter().enumerate() {
            let binding = ident.clone().unwrap_or_else(|| format_ident!("_{}", i));
            proj_pat.extend(quote!(#binding,));
            if attrs.position_exact(PIN)?.is_some() {
                let lifetime = &self.proj.lifetime;
                proj_fields.extend(quote! {
                    #vis #ident #colon_token ::pin_project::__private::Pin<&#lifetime mut (#ty)>,
                });
                proj_ref_fields.extend(quote! {
                    #vis #ident #colon_token ::pin_project::__private::Pin<&#lifetime (#ty)>,
                });
                proj_own_fields.extend(quote! {
                    #vis #ident #colon_token ::pin_project::__private::PhantomData<#ty>,
                });
                proj_body.extend(quote! {
                    #ident #colon_token ::pin_project::__private::Pin::new_unchecked(#binding),
                });
                proj_move.extend(quote! {
                    #ident #colon_token ::pin_project::__private::PhantomData,
                });

                self.pinned_fields.push(ty.clone());
                pinned_bindings.push(binding);
            } else {
                let lifetime = &self.proj.lifetime;
                proj_fields.extend(quote! {
                    #vis #ident #colon_token &#lifetime mut (#ty),
                });
                proj_ref_fields.extend(quote! {
                    #vis #ident #colon_token &#lifetime (#ty),
                });
                proj_own_fields.extend(quote! {
                    #vis #ident #colon_token #ty,
                });
                proj_body.extend(quote! {
                    #binding,
                });
                proj_move.extend(quote! {
                    #ident #colon_token ::pin_project::__private::ptr::read(#binding),
                });
            }
        }

        let orig_ident = self.orig.ident;
        let proj_pat = Group::new(delim, proj_pat);
        let proj_pat = quote!(#orig_ident #variant #proj_pat);

        let proj_body = Group::new(delim, proj_body);
        let proj_body = quote!(#variant #proj_body);

        let proj_move = Group::new(delim, proj_move);
        let proj_own_body = self.proj_own_body(variant, Some(proj_move), &pinned_bindings);

        Ok(ProjectedFields {
            proj_pat,
            proj_body,
            proj_own_body,
            proj_fields: Some(Group::new(delim, proj_fields)),
            proj_ref_fields: Some(Group::new(delim, proj_ref_fields)),
            proj_own_fields: Some(Group::new(delim, proj_own_fields)),
        })
    }

    /// Generates the processing that `project_replace` does for the struct or each variant.
    fn proj_own_body(
        &self,
        variant: Option<LeadingColon<'_>>,
        proj_move: Option<Group>,
        pinned_fields: &[Ident],
    ) -> TokenStream {
        let ident = &self.proj.own_ident;

        quote! {
            // First, extract all the unpinned fields
            let __result = #ident #variant #proj_move;

            // Destructors will run in reverse order, so next create a guard to overwrite
            // `self` with the replacement value without calling destructors.
            let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                target: __self_ptr,
                value: ::pin_project::__private::ManuallyDrop::new(__replacement),
            };

            // Now create guards to drop all the pinned fields
            //
            // Due to a compiler bug (https://github.com/rust-lang/rust/issues/47949)
            // this must be in its own scope, or else `__result` will not be dropped
            // if any of the destructors panic.
            {
                #(
                    let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(#pinned_fields);
                )*
            }

            // Finally, return the result
            __result
        }
    }

    /// Creates `Unpin` implementation for the original type.
    ///
    /// The kind of `Unpin` impl generated depends on `unpin_impl` field:
    /// * `UnpinImpl::Unsafe` - Implements `Unpin` via `UnsafeUnpin` impl.
    /// * `UnpinImpl::Negative` - Generates `Unpin` impl with bounds that will never be true.
    /// * `UnpinImpl::Default` - Generates `Unpin` impl that requires `Unpin` for all pinned fields.
    fn make_unpin_impl(&self) -> TokenStream {
        match self.unpin_impl {
            UnpinImpl::Unsafe(span) => {
                let mut proj_generics = self.proj.generics.clone();
                let orig_ident = self.orig.ident;
                let lifetime = &self.proj.lifetime;

                // Make the error message highlight `UnsafeUnpin` argument.
                proj_generics.make_where_clause().predicates.push(parse_quote_spanned! { span =>
                    ::pin_project::__private::Wrapper<#lifetime, Self>: ::pin_project::UnsafeUnpin
                });

                let (impl_generics, _, where_clause) = proj_generics.split_for_impl();
                let ty_generics = self.orig.generics.split_for_impl().1;

                quote_spanned! { span =>
                    impl #impl_generics ::pin_project::__private::Unpin for #orig_ident #ty_generics
                    #where_clause
                    {
                    }
                }
            }
            UnpinImpl::Negative(span) => {
                let mut proj_generics = self.proj.generics.clone();
                let orig_ident = self.orig.ident;
                let lifetime = &self.proj.lifetime;

                proj_generics.make_where_clause().predicates.push(parse_quote! {
                    ::pin_project::__private::Wrapper<
                        #lifetime, ::pin_project::__private::PhantomPinned
                    >: ::pin_project::__private::Unpin
                });

                let (proj_impl_generics, _, proj_where_clause) = proj_generics.split_for_impl();
                let (impl_generics, ty_generics, orig_where_clause) =
                    self.orig.generics.split_for_impl();

                // For interoperability with `forbid(unsafe_code)`, `unsafe` token should be
                // call-site span.
                let unsafety = <Token![unsafe]>::default();
                quote_spanned! { span =>
                    impl #proj_impl_generics ::pin_project::__private::Unpin
                        for #orig_ident #ty_generics
                    #proj_where_clause
                    {
                    }

                    // A dummy impl of `UnsafeUnpin`, to ensure that the user cannot implement it.
                    //
                    // To ensure that users don't accidentally write a non-functional `UnsafeUnpin`
                    // impls, we emit one ourselves. If the user ends up writing an `UnsafeUnpin`
                    // impl, they'll get a "conflicting implementations of trait" error when
                    // coherence checks are run.
                    #unsafety impl #impl_generics ::pin_project::UnsafeUnpin
                        for #orig_ident #ty_generics
                    #orig_where_clause
                    {
                    }
                }
            }
            UnpinImpl::Default => {
                let mut full_where_clause = self.orig.generics.where_clause.clone().unwrap();

                // Generate a field in our new struct for every
                // pinned field in the original type.
                let fields = self.pinned_fields.iter().enumerate().map(|(i, ty)| {
                    let field_ident = format_ident!("__field{}", i);
                    quote!(#field_ident: #ty)
                });

                // We could try to determine the subset of type parameters
                // and lifetimes that are actually used by the pinned fields
                // (as opposed to those only used by unpinned fields).
                // However, this would be tricky and error-prone, since
                // it's possible for users to create types that would alias
                // with generic parameters (e.g. 'struct T').
                //
                // Instead, we generate a use of every single type parameter
                // and lifetime used in the original struct. For type parameters,
                // we generate code like this:
                //
                // ```rust
                // struct AlwaysUnpin<T: ?Sized>(PhantomData<T>) {}
                // impl<T: ?Sized> Unpin for AlwaysUnpin<T> {}
                //
                // ...
                // _field: AlwaysUnpin<(A, B, C)>
                // ```
                //
                // This ensures that any unused type parameters
                // don't end up with `Unpin` bounds.
                let lifetime_fields = self.orig.generics.lifetimes().enumerate().map(
                    |(i, LifetimeDef { lifetime, .. })| {
                        let field_ident = format_ident!("__lifetime{}", i);
                        quote!(#field_ident: &#lifetime ())
                    },
                );

                let orig_ident = self.orig.ident;
                let struct_ident = format_ident!("__{}", orig_ident);
                let vis = self.orig.vis;
                let lifetime = &self.proj.lifetime;
                let type_params = self.orig.generics.type_params().map(|t| &t.ident);
                let proj_generics = &self.proj.generics;
                let (proj_impl_generics, proj_ty_generics, _) = proj_generics.split_for_impl();
                let (impl_generics, ty_generics, where_clause) =
                    self.orig.generics.split_for_impl();

                full_where_clause.predicates.push(parse_quote! {
                    #struct_ident #proj_ty_generics: ::pin_project::__private::Unpin
                });

                quote! {
                    // This needs to have the same visibility as the original type,
                    // due to the limitations of the 'public in private' error.
                    //
                    // Our goal is to implement the public trait `Unpin` for
                    // a potentially public user type. Because of this, rust
                    // requires that any types mentioned in the where clause of
                    // our `Unpin` impl also be public. This means that our generated
                    // `__UnpinStruct` type must also be public.
                    // However, we ensure that the user can never actually reference
                    // this 'public' type by creating this type in the inside of `const`.
                    #vis struct #struct_ident #proj_generics #where_clause {
                        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<
                            #lifetime, (#(::pin_project::__private::PhantomData<#type_params>),*)
                        >,

                        #(#fields,)*
                        #(#lifetime_fields,)*
                    }

                    impl #proj_impl_generics ::pin_project::__private::Unpin
                        for #orig_ident #ty_generics
                    #full_where_clause
                    {
                    }

                    // A dummy impl of `UnsafeUnpin`, to ensure that the user cannot implement it.
                    //
                    // To ensure that users don't accidentally write a non-functional `UnsafeUnpin`
                    // impls, we emit one ourselves. If the user ends up writing an `UnsafeUnpin`
                    // impl, they'll get a "conflicting implementations of trait" error when
                    // coherence checks are run.
                    unsafe impl #impl_generics ::pin_project::UnsafeUnpin
                        for #orig_ident #ty_generics
                    #where_clause
                    {
                    }
                }
            }
        }
    }

    /// Creates `Drop` implementation for the original type.
    ///
    /// The kind of `Drop` impl generated depends on `pinned_drop` field:
    /// * `Some` - Implements `Drop` via `PinnedDrop` impl.
    /// * `None` - Generates code that ensures that `Drop`
    ///   trait is not implemented, not `Drop` impl.
    fn make_drop_impl(&self) -> TokenStream {
        let ident = self.orig.ident;
        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();

        if let Some(span) = self.pinned_drop {
            // For interoperability with `forbid(unsafe_code)`, `unsafe` token should be
            // call-site span.
            let unsafety = <Token![unsafe]>::default();
            quote_spanned! { span =>
                impl #impl_generics ::pin_project::__private::Drop for #ident #ty_generics
                #where_clause
                {
                    fn drop(&mut self) {
                        // Safety - we're in 'drop', so we know that 'self' will
                        // never move again.
                        let pinned_self = #unsafety {
                            ::pin_project::__private::Pin::new_unchecked(self)
                        };
                        // We call `pinned_drop` only once. Since `PinnedDrop::drop`
                        // is an unsafe method and a private API, it is never called again in safe
                        // code *unless the user uses a maliciously crafted macro*.
                        #unsafety {
                            ::pin_project::__private::PinnedDrop::drop(pinned_self);
                        }
                    }
                }
            }
        } else {
            // If the user does not provide a `PinnedDrop` impl,
            // we need to ensure that they don't provide a `Drop` impl of their
            // own.
            // Based on https://github.com/upsuper/assert-impl/blob/f503255b292ab0ba8d085b657f4065403cfa46eb/src/lib.rs#L80-L87
            //
            // We create a new identifier for each struct, so that the traits
            // for different types do not conflict with each other.
            //
            // Another approach would be to provide an empty Drop impl,
            // which would conflict with a user-provided Drop impl.
            // However, this would trigger the compiler's special handling
            // of Drop types (e.g. fields cannot be moved out of a Drop type).
            // This approach prevents the creation of needless Drop impls,
            // giving users more flexibility.
            let trait_ident = format_ident!("{}MustNotImplDrop", ident);

            quote! {
                // There are two possible cases:
                // 1. The user type does not implement Drop. In this case,
                // the first blanked impl will not apply to it. This code
                // will compile, as there is only one impl of MustNotImplDrop for the user type
                // 2. The user type does impl Drop. This will make the blanket impl applicable,
                // which will then conflict with the explicit MustNotImplDrop impl below.
                // This will result in a compilation error, which is exactly what we want.
                trait #trait_ident {}
                #[allow(clippy::drop_bounds)]
                impl<T: ::pin_project::__private::Drop> #trait_ident for T {}
                impl #impl_generics #trait_ident for #ident #ty_generics #where_clause {}

                // A dummy impl of `PinnedDrop`, to ensure that the user cannot implement it.
                // Since the user did not pass `PinnedDrop` to `#[pin_project]`, any `PinnedDrop`
                // impl will not actually be called. Unfortunately, we can't detect this situation
                // directly from either the `#[pin_project]` or `#[pinned_drop]` attributes, since
                // we don't know what other attirbutes/impl may exist.
                //
                // To ensure that users don't accidentally write a non-functional `PinnedDrop`
                // impls, we emit one ourselves. If the user ends up writing a `PinnedDrop` impl,
                // they'll get a "conflicting implementations of trait" error when coherence
                // checks are run.
                impl #impl_generics ::pin_project::__private::PinnedDrop for #ident #ty_generics
                #where_clause
                {
                    unsafe fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }
            }
        }
    }

    /// Creates an implementation of the projection methods.
    ///
    /// On structs, both `project` and `project_ref` methods are always generated,
    /// and `project_replace` is only generated if `ProjReplace::span` is `Some`.
    ///
    /// On enums, only methods that the returned projected type is named will be generated.
    fn make_proj_impl(
        &self,
        is_enum: bool,
        proj_body: &TokenStream,
        proj_ref_body: &TokenStream,
        proj_own_body: &TokenStream,
    ) -> TokenStream {
        let vis = &self.proj.vis;
        let lifetime = &self.proj.lifetime;
        let orig_ident = self.orig.ident;
        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let proj_own_ident = &self.proj.own_ident;

        let orig_ty_generics = self.orig.generics.split_for_impl().1;
        let proj_ty_generics = self.proj.generics.split_for_impl().1;
        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();

        let mut project = Some(quote! {
            #vis fn project<#lifetime>(
                self: ::pin_project::__private::Pin<&#lifetime mut Self>,
            ) -> #proj_ident #proj_ty_generics {
                unsafe {
                    #proj_body
                }
            }
        });
        let mut project_ref = Some(quote! {
            #vis fn project_ref<#lifetime>(
                self: ::pin_project::__private::Pin<&#lifetime Self>,
            ) -> #proj_ref_ident #proj_ty_generics {
                unsafe {
                    #proj_ref_body
                }
            }
        });
        let mut project_replace = self.project_replace.span().map(|span| {
            // For interoperability with `forbid(unsafe_code)`, `unsafe` token should be
            // call-site span.
            let unsafety = <Token![unsafe]>::default();
            // It is enough to only set the span of the signature.
            let sig = quote_spanned! { span =>
                #vis fn project_replace(
                    self: ::pin_project::__private::Pin<&mut Self>,
                    __replacement: Self,
                ) -> #proj_own_ident #orig_ty_generics
            };
            quote! {
                #sig {
                    #unsafety {
                        #proj_own_body
                    }
                }
            }
        });

        if is_enum {
            if !self.project {
                project = None;
            }
            if !self.project_ref {
                project_ref = None;
            }
            if self.project_replace.ident().is_none() {
                project_replace = None;
            }
        }

        quote! {
            impl #impl_generics #orig_ident #ty_generics #where_clause {
                #project
                #project_ref
                #project_replace
            }
        }
    }

    /// Checks that the `[repr(packed)]` attribute is not included.
    ///
    /// This currently does two checks:
    /// * Checks the attributes of structs to ensure there is no `[repr(packed)]`.
    /// * Generates a function that borrows fields without an unsafe block.
    fn ensure_not_packed(&self, fields: &Fields) -> Result<TokenStream> {
        for meta in self.orig.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
            if let Meta::List(list) = meta {
                if list.path.is_ident("repr") {
                    for repr in list.nested.iter() {
                        match repr {
                            NestedMeta::Meta(Meta::Path(path))
                            | NestedMeta::Meta(Meta::List(MetaList { path, .. }))
                                if path.is_ident("packed") =>
                            {
                                return Err(error!(
                                    repr,
                                    "#[pin_project] attribute may not be used on #[repr(packed)] types"
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // As proc-macro-derive can't rewrite the structure definition,
        // it's probably no longer necessary, but it keeps it for now.

        // Workaround for https://github.com/taiki-e/pin-project/issues/32
        // Through the tricky use of proc macros, it's possible to bypass
        // the above check for the `repr` attribute.
        // To ensure that it's impossible to use pin projections on a `#[repr(packed)]`
        // struct, we generate code like this:
        //
        // ```rust
        // #[deny(safe_packed_borrows)]
        // fn assert_not_repr_packed(val: &MyStruct) {
        //     let _field1 = &val.field1;
        //     let _field2 = &val.field2;
        //     ...
        //     let _fieldn = &val.fieldn;
        // }
        // ```
        //
        // Taking a reference to a packed field is unsafe, and applying
        // `#[deny(safe_packed_borrows)]` makes sure that doing this without
        // an `unsafe` block (which we deliberately do not generate)
        // is a hard error.
        //
        // If the struct ends up having `#[repr(packed)]` applied somehow,
        // this will generate an (unfriendly) error message. Under all reasonable
        // circumstances, we'll detect the `#[repr(packed)]` attribute, and generate
        // a much nicer error above.
        //
        // There is one exception: If the type of a struct field has an alignment of 1
        // (e.g. u8), it is always safe to take a reference to it, even if the struct
        // is `#[repr(packed)]`. If the struct is composed entirely of types of
        // alignment 1, our generated method will not trigger an error if the
        // struct is `#[repr(packed)]`.
        //
        // Fortunately, this should have no observable consequence - `#[repr(packed)]`
        // is essentially a no-op on such a type. Nevertheless, we include a test
        // to ensure that the compiler doesn't ever try to copy the fields on
        // such a struct when trying to drop it - which is reason we prevent
        // `#[repr(packed)]` in the first place.
        //
        // See also https://github.com/taiki-e/pin-project/pull/34.
        let mut field_refs = vec![];
        match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                for Field { ident, .. } in named {
                    field_refs.push(quote!(&val.#ident;));
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                for (index, _) in unnamed.iter().enumerate() {
                    let index = Index::from(index);
                    field_refs.push(quote!(&val.#index;));
                }
            }
            Fields::Unit => {}
        }

        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();
        let ident = self.orig.ident;
        Ok(quote! {
            #[deny(safe_packed_borrows)]
            fn __assert_not_repr_packed #impl_generics (val: &#ident #ty_generics) #where_clause {
                #(#field_refs)*
            }
        })
    }
}
