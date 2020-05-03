use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::{Parse, ParseBuffer, ParseStream},
    visit_mut::VisitMut,
    *,
};

use crate::utils::*;

use super::PIN;

pub(super) fn parse_derive(input: TokenStream) -> Result<TokenStream> {
    let mut item = syn::parse2(input)?;

    match &mut item {
        Item::Struct(ItemStruct { attrs, vis, ident, generics, fields, .. }) => {
            validate_struct(ident, fields)?;
            let mut cx = Context::new(attrs, vis, ident, generics)?;

            // Do this first for a better error message.
            let packed_check = cx.ensure_not_packed(fields)?;

            let (mut proj_items, proj_impl) = cx.parse_struct(fields)?;
            let unpin_impl = cx.make_unpin_impl();
            let drop_impl = cx.make_drop_impl();

            let dummy_const = format_ident!("__SCOPE_{}", ident);
            proj_items.extend(quote! {
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
                #[allow(non_upper_case_globals)]
                const #dummy_const: () = {
                    #proj_impl
                    #unpin_impl
                    #drop_impl
                    #packed_check
                };
            });

            Ok(proj_items)
        }
        Item::Enum(ItemEnum { attrs, vis, ident, generics, brace_token, variants, .. }) => {
            validate_enum(*brace_token, variants)?;
            let mut cx = Context::new(attrs, vis, ident, generics)?;

            // We don't need to check for `#[repr(packed)]`,
            // since it does not apply to enums.

            let (mut proj_items, proj_impl) = cx.parse_enum(variants)?;
            let unpin_impl = cx.make_unpin_impl();
            let drop_impl = cx.make_drop_impl();

            let dummy_const = format_ident!("__SCOPE_{}", ident);
            proj_items.extend(quote! {
                #[allow(non_upper_case_globals)]
                const #dummy_const: () = {
                    #proj_impl
                    #unpin_impl
                    #drop_impl
                };
            });

            Ok(proj_items)
        }
        item => Err(error!(item, "#[pin_project] attribute may only be used on structs or enums")),
    }
}

fn validate_struct(ident: &Ident, fields: &Fields) -> Result<()> {
    match fields {
        Fields::Named(FieldsNamed { named: f, .. })
        | Fields::Unnamed(FieldsUnnamed { unnamed: f, .. })
            if f.is_empty() =>
        {
            Err(error!(
                fields,
                "#[pin_project] attribute may not be used on structs with zero fields"
            ))
        }
        Fields::Unit => {
            Err(error!(ident, "#[pin_project] attribute may not be used on structs with units"))
        }
        _ => Ok(()),
    }
}

fn validate_enum(brace_token: token::Brace, variants: &Variants) -> Result<()> {
    if variants.is_empty() {
        return Err(syn::Error::new(
            brace_token.span,
            "#[pin_project] attribute may not be used on enums without variants",
        ));
    }
    let has_field = variants.iter().try_fold(false, |has_field, v| {
        if let Some((_, e)) = &v.discriminant {
            Err(error!(e, "#[pin_project] attribute may not be used on enums with discriminants"))
        } else if let Some(attr) = v.attrs.find(PIN) {
            Err(error!(attr, "#[pin] attribute may only be used on fields of structs or variants"))
        } else if let Fields::Unit = v.fields {
            Ok(has_field)
        } else {
            Ok(true)
        }
    })?;
    if has_field {
        Ok(())
    } else {
        Err(error!(
            variants,
            "#[pin_project] attribute may not be used on enums that have no field"
        ))
    }
}

#[derive(Default)]
struct Args {
    pinned_drop: Option<Span>,
    unsafe_unpin: Option<Span>,
    replace: Option<Span>,
}

const DUPLICATE_PIN: &str = "duplicate #[pin] attribute";

impl Args {
    fn get(attrs: &[Attribute]) -> Result<Self> {
        let mut prev: Option<(&Attribute, Result<Args>)> = None;

        for attr in attrs {
            if attr.path.is_ident(PIN) {
                if let Some((prev_attr, prev_res)) = &prev {
                    // As the `#[pin]` attribute generated by `#[pin_project]`
                    // has the same span as `#[pin_project]`, it is possible
                    // that a useless error message will be generated.
                    let res = syn::parse2::<Self>(attr.tokens.clone());
                    let span = match (&prev_res, res) {
                        (Ok(_), Ok(_)) => unreachable!(),
                        (_, Ok(_)) => prev_attr,
                        (Ok(_), _) => attr,
                        (Err(prev_err), Err(_)) => {
                            if prev_err.to_string() == DUPLICATE_PIN {
                                attr
                            } else {
                                prev_attr
                            }
                        }
                    };
                    return Err(error!(span, DUPLICATE_PIN));
                }
                prev = Some((attr, syn::parse2::<Self>(attr.tokens.clone())));
            }
        }

        // This `unwrap` only fails if another macro removes `#[pin]`.
        prev.unwrap().1
    }
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        fn parse_input(input: ParseStream<'_>) -> Result<ParseBuffer<'_>> {
            // Extracts `#args` from `(#private(#args))`.
            if let Ok(content) = input.parenthesized() {
                if let Ok(private) = content.parse::<Ident>() {
                    if private == CURRENT_PRIVATE_MODULE {
                        if let Ok(args) = content.parenthesized() {
                            return Ok(args);
                        }
                    }
                }
            }

            // If this fails, it means that there is a `#[pin]` attribute
            // inserted by something other than `#[pin_project]` attribute.
            Err(error!(TokenStream::new(), DUPLICATE_PIN))
        }

        let input = parse_input(input)?;
        let mut args = Self::default();
        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            match &*ident.to_string() {
                "PinnedDrop" => {
                    if args.pinned_drop.is_some() {
                        return Err(error!(ident, "duplicate `PinnedDrop` argument"));
                    } else if args.replace.is_some() {
                        return Err(error!(
                            ident,
                            "arguments `PinnedDrop` and `Replace` are mutually exclusive"
                        ));
                    }
                    args.pinned_drop = Some(ident.span());
                }
                "Replace" => {
                    if args.replace.is_some() {
                        return Err(error!(ident, "duplicate `Replace` argument"));
                    } else if args.pinned_drop.is_some() {
                        return Err(error!(
                            ident,
                            "arguments `PinnedDrop` and `Replace` are mutually exclusive"
                        ));
                    }
                    args.replace = Some(ident.span());
                }
                "UnsafeUnpin" => {
                    if args.unsafe_unpin.is_some() {
                        return Err(error!(ident, "duplicate `UnsafeUnpin` argument"));
                    }
                    args.unsafe_unpin = Some(ident.span());
                }
                _ => return Err(error!(ident, "unexpected argument: {}", ident)),
            }

            if !input.is_empty() {
                let _: token::Comma = input.parse()?;
            }
        }

        Ok(args)
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
    /// Visibility of the projected type.
    vis: Visibility,
    /// Name of the projected type returned by `project` method.
    mut_ident: Ident,
    /// Name of the projected type returned by `project_ref` method.
    ref_ident: Ident,
    /// Name of the projected type returned by `project_replace` method.
    own_ident: Ident,
    /// Lifetime on the generated projected type.
    lifetime: Lifetime,
    /// Generics of the projected type.
    generics: Generics,
    /// `where` clause of the projected type. This has an additional
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

#[derive(Default)]
struct ProjectedFields {
    proj_pat: TokenStream,
    proj_body: TokenStream,
    proj_fields: TokenStream,
    proj_ref_fields: TokenStream,
    proj_own_fields: TokenStream,
    proj_move: TokenStream,
    proj_drop: TokenStream,
}

struct Context<'a> {
    orig: OriginalType<'a>,
    proj: ProjectedType,
    /// Types of the pinned fields.
    pinned_fields: Vec<Type>,
    /// `PinnedDrop` attribute.
    pinned_drop: Option<Span>,
    /// `UnsafeUnpin` attribute.
    unsafe_unpin: Option<Span>,
    // `Replace` attribute (requires Sized bound)
    replace: Option<Span>,
}

impl<'a> Context<'a> {
    fn new(
        attrs: &'a [Attribute],
        vis: &'a Visibility,
        ident: &'a Ident,
        generics: &'a mut Generics,
    ) -> Result<Self> {
        let Args { pinned_drop, unsafe_unpin, replace } = Args::get(attrs)?;

        {
            let ty_generics = generics.split_for_impl().1;
            let self_ty = syn::parse_quote!(#ident #ty_generics);
            let mut visitor = ReplaceReceiver::new(&self_ty);
            visitor.visit_where_clause_mut(generics.make_where_clause());
        }

        let mut lifetime_name = String::from(DEFAULT_LIFETIME_NAME);
        determine_lifetime_name(&mut lifetime_name, &generics.params);
        let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

        let mut proj_generics = generics.clone();
        let ty_generics = generics.split_for_impl().1;
        let ty_generics_as_generics = syn::parse_quote!(#ty_generics);
        let pred = insert_lifetime_and_bound(
            &mut proj_generics,
            lifetime.clone(),
            &ty_generics_as_generics,
            ident.clone(),
        );
        let mut where_clause = generics.clone().make_where_clause().clone();
        where_clause.predicates.push(pred);

        Ok(Self {
            proj: ProjectedType {
                vis: determine_visibility(vis),
                mut_ident: proj_ident(ident, Mutable),
                ref_ident: proj_ident(ident, Immutable),
                own_ident: proj_ident(ident, Owned),
                lifetime,
                generics: proj_generics,
                where_clause,
            },
            orig: OriginalType { attrs, vis, ident, generics },
            pinned_drop,
            unsafe_unpin,
            replace,
            pinned_fields: Vec::new(),
        })
    }

    fn parse_struct(&mut self, fields: &Fields) -> Result<(TokenStream, TokenStream)> {
        let ProjectedFields {
            proj_pat,
            proj_body,
            proj_fields,
            proj_ref_fields,
            proj_own_fields,
            proj_move,
            proj_drop,
        } = match fields {
            Fields::Named(fields) => self.visit_named(fields)?,
            Fields::Unnamed(fields) => self.visit_unnamed(fields)?,
            Fields::Unit => unreachable!(),
        };

        let orig_ident = self.orig.ident;
        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let proj_own_ident = &self.proj.own_ident;
        let vis = &self.proj.vis;
        let mut orig_generics = self.orig.generics.clone();
        let orig_where_clause = orig_generics.where_clause.take();
        let proj_generics = &self.proj.generics;
        let where_clause = &self.proj.where_clause;
        let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());

        // For tuple structs, we need to generate `(T1, T2) where Foo: Bar`
        // For non-tuple structs, we need to generate `where Foo: Bar { field1: T }`
        let (where_clause_fields, where_clause_ref_fields, where_clause_own_fields) = match fields {
            Fields::Named(_) => (
                quote!(#where_clause #proj_fields),
                quote!(#where_clause #proj_ref_fields),
                quote!(#orig_where_clause #proj_own_fields),
            ),
            Fields::Unnamed(_) => (
                quote!(#proj_fields #where_clause;),
                quote!(#proj_ref_fields #where_clause;),
                quote!(#proj_own_fields #orig_where_clause;),
            ),
            Fields::Unit => unreachable!(),
        };

        let mut proj_items = quote! {
            #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis struct #proj_ident #proj_generics #where_clause_fields
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis struct #proj_ref_ident #proj_generics #where_clause_ref_fields
        };

        if let Some(replace) = self.replace {
            proj_items.extend(quote_spanned! { replace =>
                #[allow(dead_code)] // This lint warns unused fields/variants.
                #vis struct #proj_own_ident #orig_generics #where_clause_own_fields
            });
        }

        let proj_mut_body = quote! {
            let #orig_ident #proj_pat = self.get_unchecked_mut();
            #proj_ident #proj_body
        };
        let proj_ref_body = quote! {
            let #orig_ident #proj_pat = self.get_ref();
            #proj_ref_ident #proj_body
        };
        let proj_own_body = quote! {
            let __self_ptr: *mut Self = self.get_unchecked_mut();
            let #orig_ident #proj_pat = &mut *__self_ptr;

            // First, extract all the unpinned fields
            let __result = #proj_own_ident #proj_move;

            // Destructors will run in reverse order, so next create a guard to overwrite
            // `self` with the replacement value without calling destructors.
            let __guard = ::pin_project::#private::UnsafeOverwriteGuard {
                target: __self_ptr,
                value: ::core::mem::ManuallyDrop::new(__replacement),
            };

            // Now create guards to drop all the pinned fields
            //
            // Due to a compiler bug (https://github.com/rust-lang/rust/issues/47949)
            // this must be in its own scope, or else `__result` will not be dropped
            // if any of the destructors panic.
            { #proj_drop }

            // Finally, return the result
            __result
        };
        let proj_impl = self.make_proj_impl(&proj_mut_body, &proj_ref_body, &proj_own_body);

        Ok((proj_items, proj_impl))
    }

    fn parse_enum(&mut self, variants: &Variants) -> Result<(TokenStream, TokenStream)> {
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
        let where_clause = &self.proj.where_clause;

        let mut proj_items = quote! {
            #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis enum #proj_ident #proj_generics #where_clause {
                #proj_variants
            }
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis enum #proj_ref_ident #proj_generics #where_clause {
                #proj_ref_variants
            }
        };

        if let Some(replace) = self.replace {
            proj_items.extend(quote_spanned! { replace =>
                #[allow(dead_code)] // This lint warns unused fields/variants.
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
        let proj_impl = self.make_proj_impl(&proj_mut_body, &proj_ref_body, &proj_own_body);

        Ok((proj_items, proj_impl))
    }

    fn visit_variants(&mut self, variants: &Variants) -> Result<ProjectedVariants> {
        let mut proj_variants = TokenStream::new();
        let mut proj_ref_variants = TokenStream::new();
        let mut proj_own_variants = TokenStream::new();
        let mut proj_arms = TokenStream::new();
        let mut proj_ref_arms = TokenStream::new();
        let mut proj_own_arms = TokenStream::new();
        let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());

        for Variant { ident, fields, .. } in variants {
            let ProjectedFields {
                proj_pat,
                proj_body,
                proj_fields,
                proj_ref_fields,
                proj_own_fields,
                proj_move,
                proj_drop,
            } = match fields {
                Fields::Named(fields) => self.visit_named(fields)?,
                Fields::Unnamed(fields) => self.visit_unnamed(fields)?,
                Fields::Unit => ProjectedFields::default(),
            };

            let orig_ident = self.orig.ident;
            let proj_ident = &self.proj.mut_ident;
            let proj_ref_ident = &self.proj.ref_ident;
            let proj_own_ident = &self.proj.own_ident;
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
                #orig_ident::#ident #proj_pat => {
                    #proj_ident::#ident #proj_body
                }
            });
            proj_ref_arms.extend(quote! {
                #orig_ident::#ident #proj_pat => {
                    #proj_ref_ident::#ident #proj_body
                }
            });
            proj_own_arms.extend(quote! {
                #orig_ident::#ident #proj_pat => {
                    // First, extract all the unpinned fields
                    let __result = #proj_own_ident::#ident #proj_move;

                    // Destructors will run in reverse order, so next create a guard to overwrite
                    // `self` with the replacement value without calling destructors.
                    let __guard = ::pin_project::#private::UnsafeOverwriteGuard {
                        target: __self_ptr,
                        value: ::core::mem::ManuallyDrop::new(__replacement),
                    };

                    // Now create guards to drop all the pinned fields
                    //
                    // Due to a compiler bug (https://github.com/rust-lang/rust/issues/47949)
                    // this must be in its own scope, or else `__result` will not be dropped
                    // if any of the destructors panic.
                    { #proj_drop }

                    // Finally, return the result
                    __result
                }
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

    fn visit_named(
        &mut self,
        FieldsNamed { named: fields, .. }: &FieldsNamed,
    ) -> Result<ProjectedFields> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        let mut proj_own_fields = Vec::with_capacity(fields.len());
        let mut proj_move = Vec::with_capacity(fields.len());
        let mut proj_drop = Vec::with_capacity(fields.len());
        let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());

        for Field { attrs, vis, ident, ty, .. } in fields {
            if attrs.find_exact(PIN)?.is_some() {
                self.pinned_fields.push(ty.clone());

                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis #ident: ::core::pin::Pin<&#lifetime mut (#ty)>
                });
                proj_ref_fields.push(quote! {
                    #vis #ident: ::core::pin::Pin<&#lifetime (#ty)>
                });
                proj_own_fields.push(quote! {
                    #vis #ident: ::core::marker::PhantomData<#ty>
                });
                proj_body.push(quote! {
                    #ident: ::core::pin::Pin::new_unchecked(#ident)
                });
                proj_move.push(quote! {
                    #ident: ::core::marker::PhantomData
                });
                proj_drop.push(quote! {
                    let __guard = ::pin_project::#private::UnsafeDropInPlaceGuard(#ident);
                });
            } else {
                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis #ident: &#lifetime mut (#ty)
                });
                proj_ref_fields.push(quote! {
                    #vis #ident: &#lifetime (#ty)
                });
                proj_own_fields.push(quote! {
                    #vis #ident: #ty
                });
                proj_body.push(quote! {
                    #ident
                });
                proj_move.push(quote! {
                    #ident: ::core::ptr::read(#ident)
                });
            }
            proj_pat.push(ident);
        }

        let proj_pat = quote!({ #(#proj_pat),* });
        let proj_body = quote!({ #(#proj_body),* });
        let proj_fields = quote!({ #(#proj_fields),* });
        let proj_ref_fields = quote!({ #(#proj_ref_fields),* });
        let proj_own_fields = quote!({ #(#proj_own_fields),* });
        let proj_move = quote!({ #(#proj_move),* });
        let proj_drop = quote!(#(#proj_drop)*);

        Ok(ProjectedFields {
            proj_pat,
            proj_body,
            proj_fields,
            proj_ref_fields,
            proj_own_fields,
            proj_move,
            proj_drop,
        })
    }

    fn visit_unnamed(
        &mut self,
        FieldsUnnamed { unnamed: fields, .. }: &FieldsUnnamed,
    ) -> Result<ProjectedFields> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        let mut proj_own_fields = Vec::with_capacity(fields.len());
        let mut proj_move = Vec::with_capacity(fields.len());
        let mut proj_drop = Vec::with_capacity(fields.len());
        let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());

        for (i, Field { attrs, vis, ty, .. }) in fields.iter().enumerate() {
            let id = format_ident!("_{}", i);
            if attrs.find_exact(PIN)?.is_some() {
                self.pinned_fields.push(ty.clone());

                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis ::core::pin::Pin<&#lifetime mut (#ty)>
                });
                proj_ref_fields.push(quote! {
                    #vis ::core::pin::Pin<&#lifetime (#ty)>
                });
                proj_own_fields.push(quote! {
                    #vis ::core::marker::PhantomData<#ty>
                });
                proj_body.push(quote! {
                    ::core::pin::Pin::new_unchecked(#id)
                });
                proj_move.push(quote! {
                    ::core::marker::PhantomData
                });
                proj_drop.push(quote! {
                    let __guard = ::pin_project::#private::UnsafeDropInPlaceGuard(#id);
                });
            } else {
                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis &#lifetime mut (#ty)
                });
                proj_ref_fields.push(quote! {
                    #vis &#lifetime (#ty)
                });
                proj_own_fields.push(quote! {
                    #vis #ty
                });
                proj_body.push(quote! {
                    #id
                });
                proj_move.push(quote! {
                    ::core::ptr::read(#id)
                });
            }
            proj_pat.push(id);
        }

        let proj_pat = quote!((#(#proj_pat),*));
        let proj_body = quote!((#(#proj_body),*));
        let proj_fields = quote!((#(#proj_fields),*));
        let proj_ref_fields = quote!((#(#proj_ref_fields),*));
        let proj_own_fields = quote!((#(#proj_own_fields),*));
        let proj_move = quote!((#(#proj_move),*));
        let proj_drop = quote!(#(#proj_drop)*);

        Ok(ProjectedFields {
            proj_pat,
            proj_body,
            proj_fields,
            proj_ref_fields,
            proj_own_fields,
            proj_move,
            proj_drop,
        })
    }

    /// Creates conditional `Unpin` implementation for original type.
    fn make_unpin_impl(&mut self) -> TokenStream {
        if let Some(unsafe_unpin) = self.unsafe_unpin {
            let mut proj_generics = self.proj.generics.clone();
            let orig_ident = self.orig.ident;
            let lifetime = &self.proj.lifetime;

            let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());
            proj_generics.make_where_clause().predicates.push(
                // Make the error message highlight `UnsafeUnpin` argument.
                syn::parse2(quote_spanned! { unsafe_unpin =>
                    ::pin_project::#private::Wrapper<#lifetime, Self>: ::pin_project::UnsafeUnpin
                })
                .unwrap(),
            );

            let (impl_generics, _, where_clause) = proj_generics.split_for_impl();
            let ty_generics = self.orig.generics.split_for_impl().1;

            quote! {
                #[allow(single_use_lifetimes)]
                impl #impl_generics ::core::marker::Unpin for #orig_ident #ty_generics #where_clause {}
            }
        } else {
            let mut full_where_clause = self.orig.generics.where_clause.as_ref().cloned().unwrap();

            // Generate a field in our new struct for every
            // pinned field in the original type.
            let fields: Vec<_> = self
                .pinned_fields
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    let field_ident = format_ident!("__field{}", i);
                    quote! {
                        #field_ident: #ty
                    }
                })
                .collect();

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
            let lifetime_fields: Vec<_> = self
                .orig
                .generics
                .lifetimes()
                .enumerate()
                .map(|(i, LifetimeDef { lifetime, .. })| {
                    let field_ident = format_ident!("__lifetime{}", i);
                    quote! {
                        #field_ident: &#lifetime ()
                    }
                })
                .collect();

            let orig_ident = self.orig.ident;
            let struct_ident = format_ident!("__{}", orig_ident);
            let vis = self.orig.vis;
            let lifetime = &self.proj.lifetime;
            let type_params: Vec<_> = self.orig.generics.type_params().map(|t| &t.ident).collect();
            let proj_generics = &self.proj.generics;
            let (impl_generics, proj_ty_generics, _) = proj_generics.split_for_impl();
            let (_, ty_generics, where_clause) = self.orig.generics.split_for_impl();

            full_where_clause.predicates.push(syn::parse_quote! {
                #struct_ident #proj_ty_generics: ::core::marker::Unpin
            });

            let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());
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
                    __pin_project_use_generics: ::pin_project::#private::AlwaysUnpin<#lifetime, (#(#type_params),*)>,

                    #(#fields,)*
                    #(#lifetime_fields,)*
                }

                impl #impl_generics ::core::marker::Unpin for #orig_ident #ty_generics #full_where_clause {}
            }
        }
    }

    /// Creates `Drop` implementation for original type.
    fn make_drop_impl(&self) -> TokenStream {
        let ident = self.orig.ident;
        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();

        let private = Ident::new(CURRENT_PRIVATE_MODULE, Span::call_site());
        if let Some(pinned_drop) = self.pinned_drop {
            // Make the error message highlight `PinnedDrop` argument.
            // See https://github.com/taiki-e/pin-project/issues/16#issuecomment-513586812
            // for why this is only for the span of function calls,
            // not the entire `impl` block.
            let call_drop = quote_spanned! { pinned_drop =>
                ::pin_project::#private::PinnedDrop::drop(pinned_self)
            };

            quote! {
                #[allow(single_use_lifetimes)]
                impl #impl_generics ::core::ops::Drop for #ident #ty_generics #where_clause {
                    fn drop(&mut self) {
                        // Safety - we're in 'drop', so we know that 'self' will
                        // never move again.
                        let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
                        // We call `pinned_drop` only once. Since `PinnedDrop::drop`
                        // is an unsafe method and a private API, it is never called again in safe
                        // code *unless the user uses a maliciously crafted macro*.
                        unsafe {
                            #call_drop;
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
                impl<T: ::core::ops::Drop> #trait_ident for T {}
                #[allow(single_use_lifetimes)]
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
                #[allow(single_use_lifetimes)]
                impl #impl_generics ::pin_project::#private::PinnedDrop for #ident #ty_generics #where_clause {
                    unsafe fn drop(self: ::core::pin::Pin<&mut Self>) {}
                }
            }
        }
    }

    /// Creates an implementation of the projection method.
    fn make_proj_impl(
        &self,
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

        let replace_impl = self.replace.map(|replace| {
            quote_spanned! { replace =>
                #[allow(unsafe_code)]
                #vis fn project_replace(
                    self: ::core::pin::Pin<&mut Self>,
                    __replacement: Self,
                ) -> #proj_own_ident #orig_ty_generics {
                    unsafe {
                        #proj_own_body
                    }
                }
            }
        });

        quote! {
            impl #impl_generics #orig_ident #ty_generics #where_clause {
                #vis fn project<#lifetime>(
                    self: ::core::pin::Pin<&#lifetime mut Self>,
                ) -> #proj_ident #proj_ty_generics {
                    unsafe {
                        #proj_body
                    }
                }
                #vis fn project_ref<#lifetime>(
                    self: ::core::pin::Pin<&#lifetime Self>,
                ) -> #proj_ref_ident #proj_ty_generics {
                    unsafe {
                        #proj_ref_body
                    }
                }
                #replace_impl
            }
        }
    }

    fn ensure_not_packed(&self, fields: &Fields) -> Result<TokenStream> {
        for meta in self.orig.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
            if let Meta::List(l) = meta {
                if l.path.is_ident("repr") {
                    for repr in l.nested.iter() {
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
        // is `#[repr(packed)]`. If the struct is composed entirely of types of alignment 1,
        // our generated method will not trigger an error if the struct is `#[repr(packed)]`.
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
                    field_refs.push(quote! {
                        &val.#ident;
                    });
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                for (index, _) in unnamed.iter().enumerate() {
                    let index = Index::from(index);
                    field_refs.push(quote! {
                        &val.#index;
                    });
                }
            }
            Fields::Unit => unreachable!(),
        }

        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();
        let ident = self.orig.ident;
        Ok(quote! {
            #[allow(single_use_lifetimes)]
            #[deny(safe_packed_borrows)]
            fn __assert_not_repr_packed #impl_generics (val: &#ident #ty_generics) #where_clause {
                #(#field_refs)*
            }
        })
    }
}
