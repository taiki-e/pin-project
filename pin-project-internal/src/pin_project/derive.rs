use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse::Nothing, *};

use crate::utils::{self, proj_lifetime_name, Variants, VecExt, DEFAULT_LIFETIME_NAME};

use super::PIN;

pub(super) fn parse_derive(item: Item) -> Result<TokenStream> {
    match item {
        Item::Struct(ItemStruct { vis, ident, generics, fields, .. }) => {
            let mut cx = DeriveContext::new(ident, vis, generics);
            super::validate_struct(&cx.ident, &fields)?;
            cx.visit_fields(&fields);
            Ok(cx.make_unpin_impl())
        }
        Item::Enum(ItemEnum { vis, ident, generics, brace_token, variants, .. }) => {
            let mut cx = DeriveContext::new(ident, vis, generics);
            super::validate_enum(brace_token, &variants)?;
            cx.visit_variants(&variants);
            Ok(cx.make_unpin_impl())
        }
        _ => unreachable!(),
    }
}

struct DeriveContext {
    /// Name of the original type.
    ident: Ident,

    /// Visibility of the original type.
    vis: Visibility,

    /// Generics of the original type.
    generics: Generics,

    /// Lifetime on the generated projected type.
    lifetime: Lifetime,

    /// Types of the pinned fields.
    pinned_fields: Vec<Type>,
}

impl DeriveContext {
    fn new(ident: Ident, vis: Visibility, generics: Generics) -> Self {
        let mut lifetime_name = String::from(DEFAULT_LIFETIME_NAME);
        proj_lifetime_name(&mut lifetime_name, &generics.params);
        let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

        Self { ident, vis, generics, lifetime, pinned_fields: Vec::new() }
    }

    /// Creates the generics of projected type.
    fn proj_generics(&self) -> Generics {
        let mut generics = self.generics.clone();
        utils::proj_generics(&mut generics, self.lifetime.clone());
        generics
    }

    fn visit_variants(&mut self, variants: &Variants) {
        for Variant { fields, .. } in variants {
            self.visit_fields(fields)
        }
    }

    fn visit_fields(&mut self, fields: &Fields) {
        let fields = match fields {
            Fields::Unnamed(fields) => &fields.unnamed,
            Fields::Named(fields) => &fields.named,
            Fields::Unit => return,
        };

        for Field { attrs, ty, .. } in fields {
            if let Some(attr) = attrs.position(PIN).and_then(|i| attrs.get(i)) {
                let _: Nothing = syn::parse2(attr.tokens.clone()).unwrap();
                self.pinned_fields.push(ty.clone());
            }
        }
    }

    /// Creates conditional `Unpin` implementation for original type.
    fn make_unpin_impl(&mut self) -> TokenStream {
        let where_clause = self.generics.make_where_clause().clone();
        let orig_ident = &self.ident;

        let make_span = || {
            #[cfg(proc_macro_def_site)]
            {
                proc_macro::Span::def_site().into()
            }
            #[cfg(not(proc_macro_def_site))]
            {
                proc_macro2::Span::call_site()
            }
        };

        let struct_ident = if cfg!(proc_macro_def_site) {
            format_ident!("UnpinStruct{}", orig_ident, span = make_span())
        } else {
            format_ident!("__UnpinStruct{}", orig_ident)
        };

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

        let Self { vis, lifetime, .. } = &self;
        let type_params: Vec<_> = self.generics.type_params().map(|t| t.ident.clone()).collect();
        let proj_generics = self.proj_generics();
        let (impl_generics, proj_ty_generics, _) = proj_generics.split_for_impl();
        let ty_generics = self.generics.split_for_impl().1;
        let mut full_where_clause = where_clause.clone();

        let unpin_clause: WherePredicate = syn::parse_quote! {
            #struct_ident #proj_ty_generics: ::core::marker::Unpin
        };
        full_where_clause.predicates.push(unpin_clause);

        let attrs = if cfg!(proc_macro_def_site) { quote!() } else { quote!(#[doc(hidden)]) };

        let inner_data = quote! {
            // This needs to have the same visibility as the original type,
            // due to the limitations of the 'public in private' error.
            //
            // Out goal is to implement the public trait Unpin for
            // a potentially public user type. Because of this, rust
            // requires that any types mentioned in the where clause of
            // our Unpin impl also be public. This means that our generated
            // '__UnpinStruct' type must also be public. However, we take
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

        if cfg!(proc_macro_def_site) {
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
            // the automatically generated Unpin impl for the '__UnpinStruct$Name' types.
            quote! {
                #[allow(non_snake_case)]
                fn #scope_ident() {
                    #inner_data
                }
            }
        }
    }
}
