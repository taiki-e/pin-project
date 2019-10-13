use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    *,
};

use crate::utils::*;

use super::PIN;

pub(super) fn parse_derive(mut item: Item) -> Result<TokenStream> {
    match &mut item {
        Item::Struct(ItemStruct { attrs, vis, ident, generics, fields, .. }) => {
            let mut cx = Context::new(attrs, vis, ident, generics)?;
            super::validate_struct(ident, fields)?;
            let packed_check = cx.ensure_not_packed(fields)?;
            let mut proj_items = cx.parse_struct(fields)?;
            proj_items.extend(packed_check);
            proj_items.extend(cx.make_unpin_impl());
            Ok(proj_items)
        }
        Item::Enum(ItemEnum { attrs, vis, ident, generics, brace_token, variants, .. }) => {
            let mut cx = Context::new(attrs, vis, ident, generics)?;
            super::validate_enum(*brace_token, variants)?;
            // We don't need to check for '#[repr(packed)]',
            // since it does not apply to enums.
            let mut proj_items = cx.parse_enum(variants)?;
            proj_items.extend(cx.make_unpin_impl());
            Ok(proj_items)
        }
        _ => unreachable!(),
    }
}

#[allow(dead_code)] // https://github.com/rust-lang/rust/issues/56750
struct Attrs {
    unsafe_unpin: Option<Span>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        let _: token::Paren = syn::parenthesized!(content in input);
        let arg = content.parse::<Ident>()?;

        let unsafe_unpin = match &*arg.to_string() {
            "__unsafe_unpin" => Some(arg.span()),
            "__auto_impl_unpin" => None,
            _ => unreachable!(),
        };

        Ok(Self { unsafe_unpin })
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
    generics: &'a mut Generics,
}

struct ProjectedType {
    /// Visibility of the projected type.
    vis: Visibility,
    /// Name of the projected type returned by `project` method.
    mut_ident: Ident,
    /// Name of the projected type returned by `project_ref` method.
    ref_ident: Ident,
    /// Lifetime on the generated projected type.
    lifetime: Lifetime,
    /// Generics of the projected type.
    generics: Generics,
}

struct Context<'a> {
    orig: OriginalType<'a>,
    proj: ProjectedType,
    /// Types of the pinned fields.
    pinned_fields: Vec<Type>,
    /// `UnsafeUnpin` attribute.
    unsafe_unpin: Option<Span>,
}

impl<'a> Context<'a> {
    fn new(
        attrs: &'a [Attribute],
        vis: &'a Visibility,
        ident: &'a Ident,
        generics: &'a mut Generics,
    ) -> Result<Self> {
        let attr = attrs.find(PIN).unwrap();
        let Attrs { unsafe_unpin } = syn::parse2(attr.tokens.clone()).unwrap();

        let mut lifetime_name = String::from(DEFAULT_LIFETIME_NAME);
        determine_lifetime_name(&mut lifetime_name, &generics.params);
        let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

        let mut proj_generics = generics.clone();
        insert_lifetime(&mut proj_generics, lifetime.clone());

        Ok(Self {
            proj: ProjectedType {
                vis: determine_visibility(vis),
                mut_ident: proj_ident(ident, Mutable),
                ref_ident: proj_ident(ident, Immutable),
                lifetime,
                generics: proj_generics,
            },
            orig: OriginalType { attrs, vis, ident, generics },
            unsafe_unpin,
            pinned_fields: Vec::new(),
        })
    }

    fn parse_struct(&mut self, fields: &mut Fields) -> Result<TokenStream> {
        let (proj_pat, proj_init, proj_fields, proj_ref_fields) = match fields {
            Fields::Named(fields) => self.visit_named(fields)?,
            Fields::Unnamed(fields) => self.visit_unnamed(fields, true)?,
            Fields::Unit => unreachable!(),
        };

        let orig_ident = self.orig.ident;
        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let vis = &self.proj.vis;
        let proj_generics = &self.proj.generics;
        let where_clause = self.orig.generics.split_for_impl().2;

        let mut proj_items = quote! {
            #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis struct #proj_ident #proj_generics #where_clause #proj_fields
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis struct #proj_ref_ident #proj_generics #where_clause #proj_ref_fields
        };

        let proj_body = quote! {
            let #orig_ident #proj_pat = self.get_unchecked_mut();
            #proj_ident #proj_init
        };
        let proj_ref_body = quote! {
            let #orig_ident #proj_pat = self.get_ref();
            #proj_ref_ident #proj_init
        };

        proj_items.extend(self.make_proj_impl(&proj_body, &proj_ref_body));

        Ok(proj_items)
    }

    fn parse_enum(&mut self, variants: &mut Variants) -> Result<TokenStream> {
        let (proj_variants, proj_ref_variants, proj_arms, proj_ref_arms) =
            self.visit_variants(variants)?;

        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let vis = &self.proj.vis;
        let proj_generics = &self.proj.generics;
        let where_clause = self.orig.generics.split_for_impl().2;

        let mut proj_items = quote! {
            #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis enum #proj_ident #proj_generics #where_clause {
                #(#proj_variants,)*
            }
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #vis enum #proj_ref_ident #proj_generics #where_clause {
                #(#proj_ref_variants,)*
            }
        };

        let proj_body = quote! {
            match self.get_unchecked_mut() {
                #(#proj_arms)*
            }
        };
        let proj_ref_body = quote! {
            match self.get_ref() {
                #(#proj_ref_arms)*
            }
        };

        proj_items.extend(self.make_proj_impl(&proj_body, &proj_ref_body));

        Ok(proj_items)
    }

    #[allow(clippy::type_complexity)]
    fn visit_variants(
        &mut self,
        variants: &mut Variants,
    ) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>)> {
        let mut proj_variants = Vec::with_capacity(variants.len());
        let mut proj_ref_variants = Vec::with_capacity(variants.len());
        let mut proj_arms = Vec::with_capacity(variants.len());
        let mut proj_ref_arms = Vec::with_capacity(variants.len());
        for Variant { ident, fields, .. } in variants {
            let (proj_pat, proj_body, proj_fields, proj_ref_fields) = match fields {
                Fields::Named(fields) => self.visit_named(fields)?,
                Fields::Unnamed(fields) => self.visit_unnamed(fields, false)?,
                Fields::Unit => {
                    (TokenStream::new(), TokenStream::new(), TokenStream::new(), TokenStream::new())
                }
            };

            let orig_ident = self.orig.ident;
            let proj_ident = &self.proj.mut_ident;
            let proj_ref_ident = &self.proj.ref_ident;
            proj_variants.push(quote! {
                #ident #proj_fields
            });
            proj_ref_variants.push(quote! {
                #ident #proj_ref_fields
            });
            proj_arms.push(quote! {
                #orig_ident::#ident #proj_pat => {
                    #proj_ident::#ident #proj_body
                }
            });
            proj_ref_arms.push(quote! {
                #orig_ident::#ident #proj_pat => {
                    #proj_ref_ident::#ident #proj_body
                }
            });
        }

        Ok((proj_variants, proj_ref_variants, proj_arms, proj_ref_arms))
    }

    fn visit_named(
        &mut self,
        FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    ) -> Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        for Field { attrs, vis, ident, ty, .. } in fields {
            if let Some(attr) = attrs.find(PIN) {
                parse_as_empty(&attr.tokens)?;
                self.pinned_fields.push(ty.clone());

                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis #ident: ::core::pin::Pin<&#lifetime mut #ty>
                });
                proj_ref_fields.push(quote! {
                    #vis #ident: ::core::pin::Pin<&#lifetime #ty>
                });
                proj_body.push(quote! {
                    #ident: ::core::pin::Pin::new_unchecked(#ident)
                });
            } else {
                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis #ident: &#lifetime mut #ty
                });
                proj_ref_fields.push(quote! {
                    #vis #ident: &#lifetime #ty
                });
                proj_body.push(quote! {
                    #ident
                });
            }
            proj_pat.push(ident);
        }

        let proj_pat = quote!({ #(#proj_pat),* });
        let proj_body = quote!({ #(#proj_body),* });
        let proj_fields = quote!({ #(#proj_fields),* });
        let proj_ref_fields = quote!({ #(#proj_ref_fields),* });

        Ok((proj_pat, proj_body, proj_fields, proj_ref_fields))
    }

    fn visit_unnamed(
        &mut self,
        FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
        is_struct: bool,
    ) -> Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        for (i, Field { attrs, vis, ty, .. }) in fields.iter_mut().enumerate() {
            let id = format_ident!("_{}", i);
            if let Some(attr) = attrs.find(PIN) {
                parse_as_empty(&attr.tokens)?;
                self.pinned_fields.push(ty.clone());

                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis ::core::pin::Pin<&#lifetime mut #ty>
                });
                proj_ref_fields.push(quote! {
                    #vis ::core::pin::Pin<&#lifetime #ty>
                });
                proj_body.push(quote! {
                    ::core::pin::Pin::new_unchecked(#id)
                });
            } else {
                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote! {
                    #vis &#lifetime mut #ty
                });
                proj_ref_fields.push(quote! {
                    #vis &#lifetime #ty
                });
                proj_body.push(quote! {
                    #id
                });
            }
            proj_pat.push(id);
        }

        let proj_pat = quote!((#(#proj_pat),*));
        let proj_body = quote!((#(#proj_body),*));
        let (proj_fields, proj_ref_fields) = if is_struct {
            (quote!((#(#proj_fields),*);), quote!((#(#proj_ref_fields),*);))
        } else {
            (quote!((#(#proj_fields),*)), quote!((#(#proj_ref_fields),*)))
        };

        Ok((proj_pat, proj_body, proj_fields, proj_ref_fields))
    }

    /// Creates conditional `Unpin` implementation for original type.
    fn make_unpin_impl(&mut self) -> TokenStream {
        if let Some(unsafe_unpin) = self.unsafe_unpin {
            let mut proj_generics = self.proj.generics.clone();
            let orig_ident = self.orig.ident;
            let lifetime = &self.proj.lifetime;

            proj_generics.make_where_clause().predicates.push(
                // Make the error message highlight `UnsafeUnpin` argument.
                syn::parse2(quote_spanned! { unsafe_unpin =>
                    ::pin_project::__private::Wrapper<#lifetime, Self>: ::pin_project::UnsafeUnpin
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
            let mut full_where_clause = self.orig.generics.make_where_clause().clone();
            let orig_ident = self.orig.ident;

            let make_span = || {
                #[cfg(pin_project_show_unpin_struct)]
                {
                    proc_macro::Span::def_site().into()
                }
                #[cfg(not(pin_project_show_unpin_struct))]
                {
                    proc_macro2::Span::call_site()
                }
            };

            let struct_ident = format_ident!("UnpinStruct{}", orig_ident, span = make_span());

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
            // This ensures that any unused type paramters
            // don't end up with Unpin bounds.
            let lifetime_fields: Vec<_> = self
                .orig
                .generics
                .lifetimes()
                .enumerate()
                .map(|(i, l)| {
                    let field_ident = format_ident!("__lifetime{}", i);
                    quote! {
                        #field_ident: &#l ()
                    }
                })
                .collect();

            let scope_ident = format_ident!("__unpin_scope_{}", orig_ident);

            let vis = self.orig.vis;
            let lifetime = &self.proj.lifetime;
            let type_params: Vec<_> = self.orig.generics.type_params().map(|t| &t.ident).collect();
            let proj_generics = &self.proj.generics;
            let (impl_generics, proj_ty_generics, _) = proj_generics.split_for_impl();
            let (_, ty_generics, where_clause) = self.orig.generics.split_for_impl();

            full_where_clause.predicates.push(syn::parse_quote! {
                #struct_ident #proj_ty_generics: ::core::marker::Unpin
            });

            let attrs =
                if cfg!(pin_project_show_unpin_struct) { quote!() } else { quote!(#[doc(hidden)]) };

            let inner_data = quote! {
                // This needs to have the same visibility as the original type,
                // due to the limitations of the 'public in private' error.
                //
                // Out goal is to implement the public trait Unpin for
                // a potentially public user type. Because of this, rust
                // requires that any types mentioned in the where clause of
                // our Unpin impl also be public. This means that our generated
                // 'UnpinStruct' type must also be public. However, we take
                // steps to ensure that the user can never actually reference
                // this 'public' type. These steps are described below.
                //
                // See also https://github.com/taiki-e/pin-project/pull/53.
                #[allow(dead_code)]
                #attrs
                #vis struct #struct_ident #proj_generics #where_clause {
                    __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<#lifetime, (#(#type_params),*)>,

                    #(#fields,)*
                    #(#lifetime_fields,)*
                }

                impl #impl_generics ::core::marker::Unpin for #orig_ident #ty_generics #full_where_clause {}
            };

            if cfg!(pin_project_show_unpin_struct) {
                // On nightly, we use def-site hygiene to make it impossible
                // for user code to refer to any of the types we define.
                // This allows us to omit wrapping the generated types
                // in an fn() scope, allowing rustdoc to properly document
                // them.
                inner_data
            } else {
                // When we're not on nightly, we need to create an enclosing fn() scope
                // for all of our generated items. This makes it impossible for
                // user code to refer to any of our generated types, but has
                // the advantage of preventing Rustdoc from displaying
                // docs for any of our types. In particular, users cannot see
                // the automatically generated Unpin impl for the 'UnpinStruct$Name' types.
                quote! {
                    #[allow(non_snake_case)]
                    fn #scope_ident() {
                        #inner_data
                    }
                }
            }
        }
    }

    /// Creates an implementation of the projection method.
    fn make_proj_impl(&self, proj_body: &TokenStream, proj_ref_body: &TokenStream) -> TokenStream {
        let vis = &self.proj.vis;
        let lifetime = &self.proj.lifetime;
        let orig_ident = self.orig.ident;
        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;

        let proj_ty_generics = self.proj.generics.split_for_impl().1;
        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();

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
        // the above check for the 'repr' attribute.
        // To ensure that it's impossible to use pin projections on a #[repr(packed)]
        // struct, we generate code like this:
        //
        // #[deny(safe_packed_borrows)]
        // fn enforce_not_packed_for_MyStruct(val: &MyStruct) {
        //  let _field1 = &val.field1;
        //  let _field2 = &val.field2;
        //  ...
        //  let _fieldn = &val.fieldn;
        // }
        //
        // Taking a reference to a packed field is unsafe, amd appplying
        // #[deny(safe_packed_borrows)] makes sure that doing this without
        // an 'unsafe' block (which we deliberately do not generate)
        // is a hard error.
        //
        // If the struct ends up having #[repr(packed)] applied somehow,
        // this will generate an (unfriendly) error message. Under all reasonable
        // circumstances, we'll detect the #[repr(packed)] attribute, and generate
        // a much nicer error above.
        //
        // There is one exception: If the type of a struct field has an alignment of 1
        // (e.g. u8), it is always safe to take a reference to it, even if the struct
        // is #[repr(packed)]. If the struct is composed entirely of types of alignment 1,
        // our generated method will not trigger an error if the struct is #[repr(packed)]
        //
        // Fortunately, this should have no observable consequence - #[repr(packed)]
        // is essentially a no-op on such a type. Nevertheless, we include a test
        // to ensure that the compiler doesn't ever try to copy the fields on
        // such a struct when trying to drop it - which is reason we prevent
        // #[repr(packed)] in the first place.
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
            Fields::Unit => {}
        }

        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();

        let struct_name = self.orig.ident;
        let method_name = format_ident!("__pin_project_assert_not_repr_packed_{}", self.orig.ident);
        Ok(quote! {
            #[allow(single_use_lifetimes)]
            #[allow(non_snake_case)]
            #[deny(safe_packed_borrows)]
            fn #method_name #impl_generics (val: &#struct_name #ty_generics) #where_clause {
                #(#field_refs)*
            }
        })
    }
}
