use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse::{Nothing, Parse, ParseStream},
    token::Comma,
    *,
};

use crate::utils::{
    self, collect_cfg, determine_visibility, proj_ident, proj_lifetime_name, Immutable, Mutable,
    Variants, VecExt, DEFAULT_LIFETIME_NAME,
};

use super::PIN;

pub(super) fn parse_attribute(args: TokenStream, mut item: Item) -> Result<TokenStream> {
    let cx;
    let proj_items = match &mut item {
        Item::Struct(item) => {
            super::validate_struct(&item.ident, &item.fields)?;
            cx = Context::new(args, &mut item.attrs, &item.vis, &item.ident, &item.generics)?;

            let packed_check = ensure_not_packed(&item)?;
            let mut proj_items = cx.parse_struct(&mut item.fields)?;
            proj_items.extend(packed_check);
            proj_items
        }
        Item::Enum(ItemEnum { attrs, vis, ident, generics, brace_token, variants, .. }) => {
            super::validate_enum(*brace_token, variants)?;
            cx = Context::new(args, attrs, vis, ident, generics)?;

            // We don't need to check for '#[repr(packed)]',
            // since it does not apply to enums.
            cx.parse_enum(variants)?
        }
        _ => {
            return Err(error!(
                item,
                "#[pin_project] attribute may only be used on structs or enums"
            ));
        }
    };

    let mut item = item.into_token_stream();
    item.extend(proj_items);
    item.extend(cx.make_unpin_impl());
    item.extend(cx.make_drop_impl());
    Ok(item)
}

#[allow(dead_code)] // https://github.com/rust-lang/rust/issues/56750
struct Args {
    pinned_drop: Option<Span>,
    unsafe_unpin: Option<Span>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut pinned_drop = None;
        let mut unsafe_unpin = None;
        while !input.is_empty() {
            let arg = input.parse::<Ident>()?;
            match &*arg.to_string() {
                "PinnedDrop" => {
                    if pinned_drop.is_some() {
                        return Err(error!(arg, "duplicate `PinnedDrop` argument"));
                    }
                    pinned_drop = Some(arg.span());
                }
                "UnsafeUnpin" => {
                    if unsafe_unpin.is_some() {
                        return Err(error!(arg, "duplicate `UnsafeUnpin` argument"));
                    }
                    unsafe_unpin = Some(arg.span());
                }
                _ => {
                    return Err(error!(
                        arg,
                        "an invalid argument was passed to #[pin_project] attribute"
                    ));
                }
            }

            if !input.is_empty() {
                let _: Comma = input.parse()?;
            }
        }

        Ok(Self { pinned_drop, unsafe_unpin })
    }
}

struct Context {
    /// Visibility of the original type.
    vis: Visibility,

    /// Name of the original type.
    orig_ident: Ident,

    /// Name of the projected type returned by `project` method.
    proj_ident: Ident,

    /// Name of the projected type returned by `project_ref` method.
    proj_ref_ident: Ident,

    /// Generics of the original type.
    generics: Generics,

    /// Lifetime on the generated projected type.
    lifetime: Lifetime,

    /// `UnsafeUnpin` attribute.
    unsafe_unpin: Option<Span>,

    /// `PinnedDrop` attribute.
    pinned_drop: Option<Span>,
}

impl Context {
    fn new(
        args: TokenStream,
        attrs: &mut Vec<Attribute>,
        vis: &Visibility,
        orig_ident: &Ident,
        generics: &Generics,
    ) -> Result<Self> {
        let Args { pinned_drop, unsafe_unpin } = syn::parse2(args)?;

        if unsafe_unpin.is_none() {
            attrs.push(
                syn::parse_quote!(#[derive(::pin_project::__private::__PinProjectAutoImplUnpin)]),
            );
        }

        let mut lifetime_name = String::from(DEFAULT_LIFETIME_NAME);
        proj_lifetime_name(&mut lifetime_name, &generics.params);
        let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

        Ok(Self {
            vis: determine_visibility(vis),
            orig_ident: orig_ident.clone(),
            proj_ident: proj_ident(orig_ident, Mutable),
            proj_ref_ident: proj_ident(orig_ident, Immutable),
            generics: generics.clone(),
            lifetime,
            unsafe_unpin,
            pinned_drop,
        })
    }

    /// Creates the generics of projected type.
    fn proj_generics(&self) -> Generics {
        let mut generics = self.generics.clone();
        utils::proj_generics(&mut generics, self.lifetime.clone());
        generics
    }

    fn find_pin_attr(&self, attrs: &mut Vec<Attribute>) -> Result<bool> {
        if let Some(pos) = attrs.position(PIN) {
            let tokens = if self.unsafe_unpin.is_some() {
                attrs.remove(pos).tokens
            } else {
                attrs[pos].tokens.clone()
            };
            let _: Nothing = syn::parse2(tokens)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Creates `Unpin` implementation for original type if `UnsafeUnpin` argument used.
    fn make_unpin_impl(&self) -> TokenStream {
        let unsafe_unpin = if let Some(unsafe_unpin) = self.unsafe_unpin {
            unsafe_unpin
        } else {
            // To generate the correct `Unpin` implementation,
            // we need to collect the types of the pinned fields.
            // However, since proc-macro-attribute is applied before cfg,
            // we cannot be collecting field types at this timing.
            // So instead of generating the `Unpin` implementation here,
            // we need to delegate automatic generation of the `Unpin`
            // implementation to proc-macro-derive.
            return TokenStream::new();
        };

        let mut proj_generics = self.proj_generics();
        let Self { orig_ident, lifetime, .. } = self;

        proj_generics.make_where_clause().predicates.push(
            syn::parse2(quote_spanned! { unsafe_unpin =>
                ::pin_project::__private::Wrapper<#lifetime, Self>: ::pin_project::UnsafeUnpin
            })
            .unwrap(),
        );

        let (impl_generics, _, where_clause) = proj_generics.split_for_impl();
        let ty_generics = self.generics.split_for_impl().1;

        quote! {
            #[allow(single_use_lifetimes)]
            impl #impl_generics ::core::marker::Unpin for #orig_ident #ty_generics #where_clause {}
        }
    }

    /// Creates `Drop` implementation for original type.
    fn make_drop_impl(&self) -> TokenStream {
        let orig_ident = &self.orig_ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        if let Some(pinned_drop) = self.pinned_drop {
            let call = quote_spanned! { pinned_drop =>
                ::pin_project::__private::PinnedDrop::drop(pinned_self)
            };

            quote! {
                #[allow(single_use_lifetimes)]
                impl #impl_generics ::core::ops::Drop for #orig_ident #ty_generics #where_clause {
                    fn drop(&mut self) {
                        // Safety - we're in 'drop', so we know that 'self' will
                        // never move again.
                        let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
                        // We call `pinned_drop` only once. Since `PinnedDrop::drop`
                        // is an unsafe function and a private API, it is never called again in safe
                        // code *unless the user uses a maliciously crafted macro*.
                        unsafe {
                            #call;
                        }
                    }
                }
            }
        } else {
            // If the user does not provide a pinned_drop impl,
            // we need to ensure that they don't provide a `Drop` impl of their
            // own.
            // Based on https://github.com/upsuper/assert-impl/blob/f503255b292ab0ba8d085b657f4065403cfa46eb/src/lib.rs#L80-L87
            //
            // We create a new identifier for each struct, so that the traits
            // for different types do not conflcit with each other.
            //
            // Another approach would be to provide an empty Drop impl,
            // which would conflict with a user-provided Drop impl.
            // However, this would trigger the compiler's special handling
            // of Drop types (e.g. fields cannot be moved out of a Drop type).
            // This approach prevents the creation of needless Drop impls,
            // giving users more flexibility.
            let trait_ident = format_ident!("{}MustNotImplDrop", orig_ident);

            quote! {
                // There are two possible cases:
                // 1. The user type does not implement Drop. In this case,
                // the first blanked impl will not apply to it. This code
                // will compile, as there is only one impl of MustNotImplDrop for the user type
                // 2. The user type does impl Drop. This will make the blanket impl applicable,
                // which will then comflict with the explicit MustNotImplDrop impl below.
                // This will result in a compilation error, which is exactly what we want.
                trait #trait_ident {}
                #[allow(clippy::drop_bounds)]
                impl<T: ::core::ops::Drop> #trait_ident for T {}
                #[allow(single_use_lifetimes)]
                impl #impl_generics #trait_ident for #orig_ident #ty_generics #where_clause {}
            }
        }
    }

    /// Creates an implementation of the projection method.
    fn make_proj_impl(&self, proj_body: &TokenStream, proj_ref_body: &TokenStream) -> TokenStream {
        let Context { orig_ident, proj_ident, proj_ref_ident, vis, lifetime, .. } = self;

        let proj_generics = self.proj_generics();
        let proj_ty_generics = proj_generics.split_for_impl().1;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

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
}

fn ensure_not_packed(item: &ItemStruct) -> Result<TokenStream> {
    for meta in item.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
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

    // Workaround for https://github.com/taiki-e/pin-project/issues/32
    // Through the tricky use of proc macros, it's possible to bypass
    // the above check for the 'repr' attribute.
    // To ensure that it's impossible to use pin projections on a #[repr(packed)]
    // struct, we generate code like this:
    //
    // #[deny(safe_packed_borrows)]
    // fn enforce_not_packed_for_MyStruct(val: MyStruct) {
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
    match &item.fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            for Field { attrs, ident, .. } in named {
                let cfg = collect_cfg(attrs);
                field_refs.push(quote! {
                    #(#cfg)*
                    { &val.#ident; }
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

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let struct_name = &item.ident;
    let method_name = format_ident!("__pin_project_assert_not_repr_packed_{}", item.ident);
    let test_fn = quote! {
        #[allow(single_use_lifetimes)]
        #[allow(non_snake_case)]
        #[deny(safe_packed_borrows)]
        fn #method_name #impl_generics (val: #struct_name #ty_generics) #where_clause {
            #(#field_refs)*
        }
    };
    Ok(test_fn)
}

// Visit structs/enums
impl Context {
    fn parse_struct(&self, fields: &mut Fields) -> Result<TokenStream> {
        let (proj_pat, proj_init, proj_fields, proj_ref_fields) = match fields {
            Fields::Named(fields) => self.visit_named(fields)?,
            Fields::Unnamed(fields) => self.visit_unnamed(fields, true)?,
            Fields::Unit => unreachable!(),
        };

        let Context { orig_ident, proj_ident, proj_ref_ident, vis, .. } = self;
        let proj_generics = self.proj_generics();
        let where_clause = self.generics.split_for_impl().2;

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

    fn parse_enum(&self, variants: &mut Variants) -> Result<TokenStream> {
        let (proj_variants, proj_ref_variants, proj_arms, proj_ref_arms) =
            self.visit_variants(variants)?;

        let Context { proj_ident, proj_ref_ident, vis, .. } = &self;
        let proj_generics = self.proj_generics();
        let where_clause = self.generics.split_for_impl().2;

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
        &self,
        variants: &mut Variants,
    ) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>)> {
        let mut proj_variants = Vec::with_capacity(variants.len());
        let mut proj_ref_variants = Vec::with_capacity(variants.len());
        let mut proj_arms = Vec::with_capacity(variants.len());
        let mut proj_ref_arms = Vec::with_capacity(variants.len());
        for Variant { attrs, ident, fields, .. } in variants {
            let (proj_pat, proj_body, proj_fields, proj_ref_fields) = match fields {
                Fields::Named(fields) => self.visit_named(fields)?,
                Fields::Unnamed(fields) => self.visit_unnamed(fields, false)?,
                Fields::Unit => {
                    (TokenStream::new(), TokenStream::new(), TokenStream::new(), TokenStream::new())
                }
            };

            let cfg = collect_cfg(attrs);
            let Self { orig_ident, proj_ident, proj_ref_ident, .. } = self;
            proj_variants.push(quote! {
                #(#cfg)*
                #ident #proj_fields
            });
            proj_ref_variants.push(quote! {
                #(#cfg)*
                #ident #proj_ref_fields
            });
            proj_arms.push(quote! {
                #(#cfg)*
                #orig_ident::#ident #proj_pat => {
                    #proj_ident::#ident #proj_body
                }
            });
            proj_ref_arms.push(quote! {
                #(#cfg)*
                #orig_ident::#ident #proj_pat => {
                    #proj_ref_ident::#ident #proj_body
                }
            });
        }

        Ok((proj_variants, proj_ref_variants, proj_arms, proj_ref_arms))
    }

    #[allow(clippy::cognitive_complexity)]
    fn visit_named(
        &self,
        FieldsNamed { named: fields, .. }: &mut FieldsNamed,
    ) -> Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        for Field { attrs, vis, ident, ty, .. } in fields {
            let cfg = collect_cfg(attrs);
            if self.find_pin_attr(attrs)? {
                let lifetime = &self.lifetime;
                proj_fields.push(quote! {
                    #(#cfg)*
                    #vis #ident: ::core::pin::Pin<&#lifetime mut #ty>
                });
                proj_ref_fields.push(quote! {
                    #(#cfg)*
                    #vis #ident: ::core::pin::Pin<&#lifetime #ty>
                });
                proj_body.push(quote! {
                    #(#cfg)*
                    #ident: ::core::pin::Pin::new_unchecked(#ident)
                });
            } else {
                let lifetime = &self.lifetime;
                proj_fields.push(quote! {
                    #(#cfg)*
                    #vis #ident: &#lifetime mut #ty
                });
                proj_ref_fields.push(quote! {
                    #(#cfg)*
                    #vis #ident: &#lifetime #ty
                });
                proj_body.push(quote! {
                    #(#cfg)*
                    #ident: #ident
                });
            }
            proj_pat.push(quote! {
                #(#cfg)* #ident
            });
        }

        let proj_pat = quote!({ #(#proj_pat),* });
        let proj_body = quote!({ #(#proj_body),* });
        let proj_fields = quote!({ #(#proj_fields),* });
        let proj_ref_fields = quote!({ #(#proj_ref_fields),* });

        Ok((proj_pat, proj_body, proj_fields, proj_ref_fields))
    }

    fn visit_unnamed(
        &self,
        FieldsUnnamed { unnamed: fields, .. }: &mut FieldsUnnamed,
        is_struct: bool,
    ) -> Result<(TokenStream, TokenStream, TokenStream, TokenStream)> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        for (i, Field { attrs, vis, ty, .. }) in fields.iter_mut().enumerate() {
            let id = format_ident!("_x{}", i);
            let cfg = collect_cfg(attrs);
            if !cfg.is_empty() {
                return Err(error!(
                    cfg.first(),
                    "`cfg` attributes on the field of tuple {} are not supported",
                    if is_struct { "structs" } else { "variants" }
                ));
            }
            if self.find_pin_attr(attrs)? {
                let lifetime = &self.lifetime;
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
                let lifetime = &self.lifetime;
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
            proj_pat.push(quote! {
                #id
            });
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
}
