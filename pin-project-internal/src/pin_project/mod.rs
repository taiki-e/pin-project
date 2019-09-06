use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    token::Comma,
    *,
};

use crate::utils::{
    self, crate_path, proj_ident, proj_lifetime_name, proj_trait_ident, DEFAULT_LIFETIME_NAME,
    TRAIT_LIFETIME_NAME,
};

mod enums;
mod structs;

/// The annotation for pinned type.
const PIN: &str = "pin";

pub(super) fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    parse(args, input).unwrap_or_else(|e| e.to_compile_error())
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
    /// Name of the original type.
    orig_ident: Ident,

    /// Name of the projected type.
    proj_ident: Ident,

    /// Name of the trait generated to provide a 'project' method.
    proj_trait: Ident,

    /// Visibility of the original type.
    vis: Visibility,

    /// Generics of the original type.
    generics: Generics,

    /// Lifetime on the generated projected type.
    lifetime: Lifetime,

    /// Lifetime on the generated projection trait.
    trait_lifetime: Lifetime,

    /// `where`-clause for conditional `Unpin` implementation.
    impl_unpin: WhereClause,

    pinned_fields: Vec<Type>,

    unsafe_unpin: bool,

    pinned_drop: Option<Span>,
}

impl Context {
    fn new(
        args: TokenStream,
        orig_ident: &Ident,
        vis: &Visibility,
        generics: &Generics,
    ) -> Result<Self> {
        let Args { pinned_drop, unsafe_unpin } = syn::parse2(args)?;
        let proj_ident = proj_ident(orig_ident);
        let proj_trait = proj_trait_ident(orig_ident);

        let mut lifetime_name = String::from(DEFAULT_LIFETIME_NAME);
        proj_lifetime_name(&mut lifetime_name, &generics.params);
        let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

        let mut trait_lifetime_name = String::from(TRAIT_LIFETIME_NAME);
        proj_lifetime_name(&mut trait_lifetime_name, &generics.params);
        let trait_lifetime = Lifetime::new(&trait_lifetime_name, Span::call_site());

        let mut generics = generics.clone();
        let mut impl_unpin = generics.make_where_clause().clone();
        if let Some(unsafe_unpin) = unsafe_unpin {
            let crate_path = crate_path();
            impl_unpin.predicates.push(
                syn::parse2(quote_spanned! { unsafe_unpin =>
                    ::#crate_path::__private::Wrapper<Self>: ::#crate_path::UnsafeUnpin
                })
                .unwrap(),
            );
        }

        Ok(Self {
            orig_ident: orig_ident.clone(),
            proj_ident,
            proj_trait,
            vis: vis.clone(),
            generics,
            lifetime,
            trait_lifetime,
            impl_unpin,
            pinned_fields: vec![],
            unsafe_unpin: unsafe_unpin.is_some(),
            pinned_drop,
        })
    }

    /// Creates the generics of projected type.
    fn proj_generics(&self) -> Generics {
        let mut generics = self.generics.clone();
        utils::proj_generics(&mut generics, self.lifetime.clone());
        generics
    }

    /// Creates the generics for the 'project_into' method.
    fn project_into_generics(&self) -> Generics {
        let mut generics = self.generics.clone();
        utils::proj_generics(&mut generics, self.trait_lifetime.clone());
        generics
    }

    fn push_unpin_bounds(&mut self, ty: Type) {
        self.pinned_fields.push(ty);
    }

    /// Creates conditional `Unpin` implementation for original type.
    fn make_unpin_impl(&self) -> TokenStream {
        let orig_ident = &self.orig_ident;
        let (impl_generics, ty_generics, _) = self.generics.split_for_impl();
        let type_params: Vec<_> = self.generics.type_params().map(|t| t.ident.clone()).collect();
        let where_clause = &self.impl_unpin;

        if self.unsafe_unpin {
            quote! {
                impl #impl_generics ::core::marker::Unpin for #orig_ident #ty_generics #where_clause {}
            }
        } else {
            let make_span = || {
                #[cfg(proc_macro_def_site)]
                {
                    proc_macro::Span::def_site().into()
                }
                #[cfg(not(proc_macro_def_site))]
                {
                    Span::call_site()
                }
            };

            let struct_ident = if cfg!(proc_macro_def_site) {
                format_ident!("UnpinStruct{}", orig_ident, span = make_span())
            } else {
                format_ident!("__UnpinStruct{}", orig_ident)
            };
            let always_unpin_ident = format_ident!("AlwaysUnpin{}", orig_ident, span = make_span());

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

            let vis = &self.vis;
            let full_generics = &self.generics;
            let mut full_where_clause = where_clause.clone();

            let unpin_clause: WherePredicate = syn::parse_quote! {
                #struct_ident #ty_generics: ::core::marker::Unpin
            };

            full_where_clause.predicates.push(unpin_clause);

            let attrs = if cfg!(proc_macro_def_site) { quote!() } else { quote!(#[doc(hidden)]) };

            let inner_data = quote! {
                struct #always_unpin_ident <T: ?Sized> {
                    val: ::core::marker::PhantomData<T>
                }

                impl<T: ?Sized> ::core::marker::Unpin for #always_unpin_ident <T> {}

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
                #vis struct #struct_ident #full_generics #where_clause {
                    __pin_project_use_generics: #always_unpin_ident <(#(#type_params),*)>,

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

    /// Creates `Drop` implementation for original type.
    fn make_drop_impl(&self) -> TokenStream {
        let orig_ident = &self.orig_ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        if let Some(pinned_drop) = self.pinned_drop {
            let crate_path = crate_path();
            let call = quote_spanned! { pinned_drop =>
                ::#crate_path::__private::UnsafePinnedDrop::pinned_drop(pinned_self)
            };

            quote! {
                #[allow(single_use_lifetimes)]
                impl #impl_generics ::core::ops::Drop for #orig_ident #ty_generics #where_clause {
                    fn drop(&mut self) {
                        // Safety - we're in 'drop', so we know that 'self' will
                        // never move again.
                        let pinned_self = unsafe { ::core::pin::Pin::new_unchecked(self) };
                        // We call `pinned_drop` only once. Since `UnsafePinnedDrop::pinned_drop`
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

    /// Creates a definition of the projection trait.
    fn make_proj_trait(&self) -> TokenStream {
        let Self { proj_ident, proj_trait, lifetime, .. } = self;
        let proj_generics = self.proj_generics();
        let proj_ty_generics = proj_generics.split_for_impl().1;

        // Add trait lifetime to trait generics.
        let mut trait_generics = self.generics.clone();
        utils::proj_generics(&mut trait_generics, self.trait_lifetime.clone());

        let (trait_generics, trait_ty_generics, orig_where_clause) =
            trait_generics.split_for_impl();

        quote! {
            trait #proj_trait #trait_generics {
                fn project<#lifetime>(&#lifetime mut self) -> #proj_ident #proj_ty_generics #orig_where_clause;
                fn project_into(self) -> #proj_ident #trait_ty_generics #orig_where_clause;
            }
        }
    }

    /// Creates an implementation of the projection trait.
    fn make_proj_impl(
        &self,
        project_body: &TokenStream,
        project_into_body: &TokenStream,
    ) -> TokenStream {
        let Context { proj_ident, proj_trait, orig_ident, lifetime, trait_lifetime, .. } = &self;
        let proj_generics = self.proj_generics();

        let project_into_generics = self.project_into_generics();

        let proj_ty_generics = proj_generics.split_for_impl().1;
        let (impl_generics, project_into_ty_generics, _) = project_into_generics.split_for_impl();
        let (_, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_generics #proj_trait #project_into_ty_generics
                for ::core::pin::Pin<&#trait_lifetime mut #orig_ident #ty_generics> #where_clause
            {
                fn project<#lifetime>(&#lifetime mut self) -> #proj_ident #proj_ty_generics #where_clause {
                    unsafe {
                        #project_body
                    }
                }
                fn project_into(self) -> #proj_ident #project_into_ty_generics #where_clause {
                    unsafe {
                        #project_into_body
                    }
                }
            }
        }
    }
}

fn parse(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    match syn::parse2(input)? {
        Item::Struct(item) => {
            let mut cx = Context::new(args, &item.ident, &item.vis, &item.generics)?;

            let packed_check = ensure_not_packed(&item)?;
            let mut res = structs::parse(&mut cx, item)?;
            res.extend(cx.make_proj_trait());
            res.extend(cx.make_unpin_impl());
            res.extend(cx.make_drop_impl());
            res.extend(packed_check);
            Ok(res)
        }
        Item::Enum(item) => {
            let mut cx = Context::new(args, &item.ident, &item.vis, &item.generics)?;

            // We don't need to check for '#[repr(packed)]',
            // since it does not apply to enums.
            let mut res = enums::parse(&mut cx, item)?;
            res.extend(cx.make_proj_trait());
            res.extend(cx.make_unpin_impl());
            res.extend(cx.make_drop_impl());
            Ok(res)
        }
        item => Err(error!(item, "#[pin_project] attribute may only be used on structs or enums")),
    }
}

fn ensure_not_packed(item: &ItemStruct) -> Result<TokenStream> {
    for meta in item.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
        if let Meta::List(l) = meta {
            if l.path.is_ident("repr") {
                for repr in &l.nested {
                    if let NestedMeta::Meta(Meta::Path(p)) = repr {
                        if p.is_ident("packed") {
                            return Err(error!(
                                p,
                                "#[pin_project] attribute may not be used on #[repr(packed)] types"
                            ));
                        }
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
            for field in named {
                let ident = field.ident.as_ref().unwrap();
                field_refs.push(quote!(&val.#ident;));
            }
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            for (i, _) in unnamed.iter().enumerate() {
                let index = Index::from(i);
                field_refs.push(quote!(&val.#index;));
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
