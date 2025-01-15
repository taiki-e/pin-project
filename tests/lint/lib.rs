// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Check interoperability with rustc and clippy lints.

#![allow(unknown_lints)] // for old compilers
#![warn(nonstandard_style, rust_2018_idioms, unused, deprecated_safe)]
// Note: This does not guarantee compatibility with forbidding these lints in the future.
// If rustc adds a new lint, we may not be able to keep this.
#![forbid(
    future_incompatible,
    rust_2018_compatibility,
    rust_2021_compatibility,
    rust_2024_compatibility
)]
// lints forbidden as a part of future_incompatible, rust_2018_compatibility, rust_2021_compatibility, rust_2024_compatibility are not included in the list below.
// elided_lifetimes_in_paths, explicit_outlives_requirements, unused_extern_crates: included as a part of rust_2018_idioms
// non_exhaustive_omitted_patterns, multiple_supertrait_upcastable: unstable
// unstable_features: no way to generate #![feature(..)] by macros, expect for unstable inner attribute. and this lint is deprecated: https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unstable-features
// unused_crate_dependencies, must_not_suspend: unrelated
// unsafe_code: checked in forbid_unsafe module
#![warn(
    ambiguous_negative_literals,
    closure_returning_async_block,
    deprecated_in_future,
    dereferencing_mut_binding,
    fuzzy_provenance_casts,
    impl_trait_redundant_captures,
    invalid_reference_casting,
    let_underscore_drop,
    lossy_provenance_casts,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents, // TODO: add test case
    non_local_definitions,
    noop_method_call,
    private_bounds,
    private_interfaces,
    redundant_imports,
    redundant_lifetimes,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unit_bindings,
    unnameable_types,
    unqualified_local_imports,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
#![allow(clippy::blanket_clippy_restriction_lints)] // this is a test, so enable all restriction lints intentionally.
#![allow(
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::arbitrary_source_item_ordering
)] // TODO

/// Test for basic cases.
pub mod basic {
    include!("../include/basic.rs");

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing default struct.
                #[::pin_project::pin_project]
                #[derive(Debug)]
                #[allow(clippy::exhaustive_structs)] // for the type itself
                pub struct DefaultStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing named struct.
                #[::pin_project::pin_project(
                    project = DefaultStructNamedProj,
                    project_ref = DefaultStructNamedProjRef,
                )]
                #[derive(Debug)]
                #[allow(clippy::exhaustive_structs)] // for the type itself
                pub struct DefaultStructNamed<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing default tuple struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project]
                #[derive(Debug)]
                pub struct DefaultTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing named tuple struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(
                    project = DefaultTupleStructNamedProj,
                    project_ref = DefaultTupleStructNamedProjRef,
                )]
                #[derive(Debug)]
                pub struct DefaultTupleStructNamed<T, U>(#[pin] pub T, pub U);

                /// Testing enum.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    project = DefaultEnumProj,
                    project_ref = DefaultEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum DefaultEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                /// Testing pinned drop struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                /// Testing pinned drop tuple struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropTupleStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                /// Testing pinned drop enum.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    PinnedDrop,
                    project = PinnedDropEnumProj,
                    project_ref = PinnedDropEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum PinnedDropEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropEnum<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                /// Testing default struct with replace.
                #[::pin_project::pin_project(project_replace)]
                #[derive(Debug)]
                #[allow(clippy::exhaustive_structs)] // for the type itself
                pub struct ReplaceStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing named struct with replace.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(
                    project = ReplaceStructNamedProj,
                    project_ref = ReplaceStructNamedProjRef,
                    project_replace = ReplaceStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceStructNamed<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing default struct with replace.
                #[::pin_project::pin_project(project_replace)]
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[derive(Debug)]
                pub struct ReplaceTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing named struct with replace.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(
                    project = ReplaceTupleStructNamedProj,
                    project_ref = ReplaceTupleStructNamedProjRef,
                    project_replace = ReplaceTupleStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceTupleStructNamed<T, U>(#[pin] pub T, pub U);

                /// Testing enum with replace.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    project = ReplaceEnumProj,
                    project_ref = ReplaceEnumProjRef,
                    project_replace = ReplaceEnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum ReplaceEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                /// Testing struct with unsafe `Unpin`.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing tuple struct with unsafe `Unpin`.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing enum unsafe `Unpin`.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    UnsafeUnpin,
                    project = UnsafeUnpinEnumProj,
                    project_ref = UnsafeUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum UnsafeUnpinEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }


                /// Testing struct with `!Unpin`.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                pub struct NotUnpinStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing tuple struct with `!Unpin`.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                pub struct NotUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing enum with `!Unpin`.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    !Unpin,
                    project = NotUnpinEnumProj,
                    project_ref = NotUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum NotUnpinEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                #[allow(clippy::undocumented_unsafe_blocks)]
                unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
                    ::pin_project::UnsafeUnpin for UnsafeUnpinStruct<T, U>
                {
                }
                #[allow(clippy::undocumented_unsafe_blocks)]
                unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
                    ::pin_project::UnsafeUnpin for UnsafeUnpinTupleStruct<T, U>
                {
                }
                #[allow(clippy::undocumented_unsafe_blocks)]
                unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
                    ::pin_project::UnsafeUnpin for UnsafeUnpinEnum<T, U>
                {
                }
            };
        }

        mac!();
    }
}

/// Test for `forbid(unsafe_code)`-able cases.
pub mod forbid_unsafe {
    #![forbid(unsafe_code)]

    include!("../include/basic-safe-part.rs");

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing default struct.
                #[::pin_project::pin_project]
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[derive(Debug)]
                pub struct DefaultStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing named struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(
                    project = DefaultStructNamedProj,
                    project_ref = DefaultStructNamedProjRef,
                )]
                #[derive(Debug)]
                pub struct DefaultStructNamed<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing default tuple struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project]
                #[derive(Debug)]
                pub struct DefaultTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing named tuple struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(
                    project = DefaultTupleStructNamedProj,
                    project_ref = DefaultTupleStructNamedProjRef,
                )]
                #[derive(Debug)]
                pub struct DefaultTupleStructNamed<T, U>(#[pin] pub T, pub U);

                /// Testing enum.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    project = DefaultEnumProj,
                    project_ref = DefaultEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum DefaultEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                /// Testing pinned drop struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                /// Testing pinned drop tuple struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropTupleStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                /// Testing pinned drop enum.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    PinnedDrop,
                    project = PinnedDropEnumProj,
                    project_ref = PinnedDropEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum PinnedDropEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropEnum<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                /// Testing default struct with replace.
                #[::pin_project::pin_project(project_replace)]
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[derive(Debug)]
                pub struct ReplaceStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing named struct with replace.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(
                    project = ReplaceStructNamedProj,
                    project_ref = ReplaceStructNamedProjRef,
                    project_replace = ReplaceStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceStructNamed<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing default struct with replace.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(project_replace)]
                #[derive(Debug)]
                pub struct ReplaceTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing named struct with replace.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(
                    project = ReplaceTupleStructNamedProj,
                    project_ref = ReplaceTupleStructNamedProjRef,
                    project_replace = ReplaceTupleStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceTupleStructNamed<T, U>(#[pin] pub T, pub U);

                /// Testing enum with replace.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    project = ReplaceEnumProj,
                    project_ref = ReplaceEnumProjRef,
                    project_replace = ReplaceEnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum ReplaceEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                /// Testing struct with unsafe `Unpin`.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing tuple struct with unsafe `Unpin`.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing enum unsafe `Unpin`.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    UnsafeUnpin,
                    project = UnsafeUnpinEnumProj,
                    project_ref = UnsafeUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum UnsafeUnpinEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }

                /// Testing struct with `!Unpin`.
                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                #[allow(clippy::exhaustive_structs)] // for the type itself
                pub struct NotUnpinStruct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing tuple struct with `!Unpin`.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                pub struct NotUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing enum with `!Unpin`.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[::pin_project::pin_project(
                    !Unpin,
                    project = NotUnpinEnumProj,
                    project_ref = NotUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum NotUnpinEnum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }
            };
        }

        mac!();
    }
}

/// Test for `deprecated` lint.
pub mod deprecated {
    use pin_project::pin_project;

    /// Testing struct.
    #[allow(deprecated, clippy::exhaustive_structs, clippy::min_ident_chars)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug, Clone, Copy)]
    #[deprecated]
    pub struct Struct {
        /// Pinned field.
        #[deprecated]
        #[pin]
        pub p: (),
        /// Unpinned field.
        #[deprecated]
        pub u: (),
    }

    /// Testing tuple struct.
    #[allow(deprecated, clippy::exhaustive_structs)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug, Clone, Copy)]
    #[deprecated]
    pub struct TupleStruct(
        /// Pinned field.
        #[deprecated]
        #[pin]
        pub (),
        /// Unpinned field.
        #[deprecated]
        pub (),
    );

    /// Testing enum.
    #[allow(deprecated, clippy::exhaustive_enums, clippy::min_ident_chars)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug, Clone, Copy)]
    #[deprecated]
    pub enum Enum {
        /// Struct variant.
        #[deprecated]
        Struct {
            /// Pinned field.
            #[deprecated]
            #[pin]
            p: (),
            /// Unpinned field.
            #[deprecated]
            u: (),
        },
        /// Tuple variant.
        #[deprecated]
        Tuple(
            #[deprecated]
            #[pin]
            (),
            #[deprecated] (),
        ),
        /// Unit variant.
        #[deprecated]
        Unit,
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing struct.
                #[allow(deprecated, clippy::exhaustive_structs, clippy::min_ident_chars)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug, Clone, Copy)]
                #[deprecated]
                pub struct Struct {
                    /// Pinned field.
                    #[deprecated]
                    #[pin]
                    pub p: (),
                    /// Unpinned field.
                    #[deprecated]
                    pub u: (),
                }

                /// Testing tuple struct.
                #[allow(deprecated, clippy::exhaustive_structs)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug, Clone, Copy)]
                #[deprecated]
                pub struct TupleStruct(
                    /// Pinned field.
                    #[deprecated]
                    #[pin]
                    pub (),
                    /// Unpinned field.
                    #[deprecated]
                    pub (),
                );

                /// Testing enum.
                #[allow(deprecated, clippy::exhaustive_enums, clippy::min_ident_chars)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug, Clone, Copy)]
                #[deprecated]
                pub enum Enum {
                    /// Struct variant.
                    #[deprecated]
                    Struct {
                        /// Pinned field.
                        #[deprecated]
                        #[pin]
                        p: (),
                        /// Unpinned field.
                        #[deprecated]
                        u: (),
                    },
                    /// Tuple variant.
                    #[deprecated]
                    Tuple(
                        #[deprecated]
                        #[pin]
                        (),
                        #[deprecated] (),
                    ),
                    /// Unit variant.
                    #[deprecated]
                    Unit,
                }
            };
        }

        mac!();
    }
}

/// Test for `explicit_outlives_requirements` lint.
pub mod explicit_outlives_requirements {
    use pin_project::pin_project;

    /// Testing struct.
    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<'a, T, U>
    where
        T: ?Sized,
        U: ?Sized,
    {
        /// Pinned field.
        #[pin]
        pub pinned: &'a mut T,
        /// Unpinned field.
        pub unpinned: &'a mut U,
    }

    /// Testing tuple struct.
    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<'a, T, U>(#[pin] pub &'a mut T, pub &'a mut U)
    where
        T: ?Sized,
        U: ?Sized;

    /// Testing enum.
    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[allow(clippy::exhaustive_enums, clippy::single_char_lifetime_names)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<'a, T, U>
    where
        T: ?Sized,
        U: ?Sized,
    {
        /// Struct variant.
        Struct {
            /// Pinned field.
            #[pin]
            pinned: &'a mut T,
            /// Unpinned field.
            unpinned: &'a mut U,
        },
        /// Tuple variant.
        Tuple(#[pin] &'a mut T, &'a mut U),
        /// Unit variant.
        Unit,
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing struct.
                #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
                #[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Struct<'a, T, U>
                where
                    T: ?Sized,
                    U: ?Sized,
                {
                    /// Pinned field.
                    #[pin]
                    pub pinned: &'a mut T,
                    /// Unpinned field.
                    pub unpinned: &'a mut U,
                }

                /// Testing tuple struct.
                #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
                #[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct TupleStruct<'a, T, U>(#[pin] pub &'a mut T, pub &'a mut U)
                where
                    T: ?Sized,
                    U: ?Sized;

                /// Testing enum.
                #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
                #[allow(clippy::exhaustive_enums, clippy::single_char_lifetime_names)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum Enum<'a, T, U>
                where
                    T: ?Sized,
                    U: ?Sized,
                {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: &'a mut T,
                        /// Unpinned field.
                        unpinned: &'a mut U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] &'a mut T, &'a mut U),
                    /// Unit variant.
                    Unit,
                }
            };
        }

        mac!();
    }
}

/// Test for `single_use_lifetimes` lint.
#[allow(missing_debug_implementations)]
pub mod single_use_lifetimes {
    use pin_project::pin_project;

    /// Testing trait.
    #[allow(unused_lifetimes)]
    #[allow(clippy::single_char_lifetime_names)] // for the type itself
    pub trait Trait<'a> {}

    /// Testing HRTB.
    #[allow(unused_lifetimes)] // for the type itself
    #[allow(single_use_lifetimes)] // for the type itself: https://github.com/rust-lang/rust/issues/55058
    #[pin_project(project_replace)]
    pub struct Hrtb<'pin___, T>
    where
        for<'pin> &'pin T: Unpin,
        T: for<'pin> Trait<'pin>,
        for<'pin, 'pin_, 'pin__> &'pin &'pin_ &'pin__ T: Unpin,
    {
        /// Pinned field.
        #[pin]
        _f: &'pin___ mut T,
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing trait.
                #[allow(unused_lifetimes)]
                #[allow(clippy::single_char_lifetime_names)] // for the type itself
                pub trait Trait<'a> {}

                /// Testing HRTB.
                #[allow(unused_lifetimes)] // for the type itself
                #[allow(single_use_lifetimes)] // for the type itself: https://github.com/rust-lang/rust/issues/55058
                #[pin_project(project_replace)]
                pub struct Hrtb<'pin___, T>
                where
                    for<'pin> &'pin T: Unpin,
                    T: for<'pin> Trait<'pin>,
                    for<'pin, 'pin_, 'pin__> &'pin &'pin_ &'pin__ T: Unpin,
                {
                    /// Pinned field.
                    #[pin]
                    _f: &'pin___ mut T,
                }
            };
        }

        mac!();
    }
}

/// Test for `variant_size_differences` and `clippy::large_enum_variant` lints.
pub mod variant_size_differences {
    use pin_project::pin_project;

    /// Testing enum.
    #[allow(missing_debug_implementations, missing_copy_implementations)] // https://github.com/rust-lang/rust/pull/74060
    #[allow(variant_size_differences, clippy::exhaustive_enums, clippy::large_enum_variant)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    pub enum Enum {
        /// Small variant size.
        V1(u8),
        /// Huge variant size.
        V2([u8; 1024]),
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing enum.
                #[allow(missing_debug_implementations, missing_copy_implementations)] // https://github.com/rust-lang/rust/pull/74060
                #[allow(variant_size_differences, clippy::exhaustive_enums, clippy::large_enum_variant)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                pub enum Enum {
                    /// Small variant size.
                    V1(u8),
                    /// Huge variant size.
                    V2([u8; 1024]),
                }
            };
        }

        mac!();
    }
}

/// Test for `clippy::mut_mut` lint.
pub mod clippy_mut_mut {
    use pin_project::pin_project;

    /// Testing struct.
    #[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<'a, T, U> {
        /// Pinned field.
        #[pin]
        pub pinned: &'a mut T,
        /// Unpinned field.
        pub unpinned: &'a mut U,
    }

    /// Testing tuple struct.
    #[allow(clippy::single_char_lifetime_names)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

    /// Testing enum.
    #[allow(clippy::exhaustive_enums, clippy::single_char_lifetime_names)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<'a, T, U> {
        /// Struct variant.
        Struct {
            /// Pinned field.
            #[pin]
            pinned: &'a mut T,
            /// Unpinned field.
            unpinned: &'a mut U,
        },
        /// Tuple variant.
        Tuple(#[pin] &'a mut T, &'a mut U),
        /// Unit variant.
        Unit,
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing struct.
                #[allow(clippy::exhaustive_structs, clippy::single_char_lifetime_names)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Struct<'a, T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: &'a mut T,
                    /// Unpinned field.
                    pub unpinned: &'a mut U,
                }

                /// Testing tuple struct.
                #[allow(clippy::single_char_lifetime_names)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct TupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

                /// Testing enum.
                #[allow(clippy::exhaustive_enums, clippy::single_char_lifetime_names)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum Enum<'a, T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: &'a mut T,
                        /// Unpinned field.
                        unpinned: &'a mut U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] &'a mut T, &'a mut U),
                    /// Unit variant.
                    Unit,
                }
            };
        }

        mac!();
    }
}

/// Test for `clippy::redundant_pub_crate` lint.
#[allow(missing_debug_implementations)]
#[allow(unreachable_pub)]
mod clippy_redundant_pub_crate {
    use pin_project::pin_project;

    /// Testing struct.
    #[pin_project(project_replace)]
    pub struct Struct<T, U> {
        /// Pinned field.
        #[pin]
        pub pinned: T,
        /// Unpinned field.
        pub unpinned: U,
    }

    /// Testing tuple struct.
    #[pin_project(project_replace)]
    pub struct TupleStruct<T, U>(#[pin] pub T, pub U);

    /// Testing enum.
    #[allow(dead_code)]
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    pub enum Enum<T, U> {
        /// Struct variant.
        Struct {
            /// Pinned field.
            #[pin]
            pinned: T,
            /// Unpinned field.
            unpinned: U,
        },
        /// Tuple variant.
        Tuple(#[pin] T, U),
        /// Unit variant.
        Unit,
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[allow(clippy::redundant_pub_crate)]
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing struct.
                #[pin_project(project_replace)]
                pub struct Struct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing tuple struct.
                #[pin_project(project_replace)]
                pub struct TupleStruct<T, U>(#[pin] pub T, pub U);

                /// Testing enum.
                #[allow(dead_code)]
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                pub enum Enum<T, U> {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }
            };
        }

        mac!();
    }
}

/// Test for `clippy::type_repetition_in_bounds` lint.
#[allow(missing_debug_implementations)]
pub mod clippy_type_repetition_in_bounds {
    use pin_project::pin_project;

    /// Testing struct.
    #[pin_project(project_replace)]
    #[allow(clippy::exhaustive_structs)] // for the type itself
    pub struct Struct<T, U>
    where
        Self: Sized,
    {
        /// Pinned field.
        #[pin]
        pub pinned: T,
        /// Unpinned field.
        pub unpinned: U,
    }

    /// Testing tuple struct.
    #[pin_project(project_replace)]
    pub struct TupleStruct<T, U>(#[pin] T, U)
    where
        Self: Sized;

    /// Testing enum.
    #[allow(clippy::exhaustive_enums)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    pub enum Enum<T, U>
    where
        Self: Sized,
    {
        /// Struct variant.
        Struct {
            /// Pinned field.
            #[pin]
            pinned: T,
            /// Unpinned field.
            unpinned: U,
        },
        /// Tuple variant.
        Tuple(#[pin] T, U),
        /// Unit variant.
        Unit,
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing struct.
                #[allow(clippy::exhaustive_structs)] // for the type itself
                #[pin_project(project_replace)]
                pub struct Struct<T, U>
                where
                    Self: Sized,
                {
                    /// Pinned field.
                    #[pin]
                    pub pinned: T,
                    /// Unpinned field.
                    pub unpinned: U,
                }

                /// Testing tuple struct.
                #[pin_project(project_replace)]
                pub struct TupleStruct<T, U>(#[pin] T, U)
                where
                    Self: Sized;

                /// Testing enum.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                pub enum Enum<T, U>
                where
                    Self: Sized,
                {
                    /// Struct variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        pinned: T,
                        /// Unpinned field.
                        unpinned: U,
                    },
                    /// Tuple variant.
                    Tuple(#[pin] T, U),
                    /// Unit variant.
                    Unit,
                }
            };
        }

        mac!();
    }
}

/// Test for `clippy::use_self` lint.
#[allow(missing_debug_implementations)]
pub mod clippy_use_self {
    use pin_project::pin_project;

    /// Testing trait.
    pub trait Trait {
        /// Associated type.
        type Assoc;
    }

    /// Testing struct.
    #[pin_project(project_replace)]
    pub struct Generics<T: Trait<Assoc = Self>>
    where
        Self: Trait<Assoc = Self>,
    {
        /// Field holding generic.
        _f: T,
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        use super::Trait;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing struct.
                #[pin_project(project_replace)]
                pub struct Generics<T: Trait<Assoc = Self>>
                where
                    Self: Trait<Assoc = Self>,
                {
                    /// Field holding generic.
                    _f: T,
                }
            };
        }

        mac!();
    }
}

/// Test for `clippy::used_underscore_binding` lint.
#[allow(missing_debug_implementations)]
pub mod clippy_used_underscore_binding {
    use pin_project::pin_project;

    /// Testing struct.
    #[allow(clippy::exhaustive_structs, clippy::pub_underscore_fields)] // for the type itself
    #[pin_project(project_replace)]
    pub struct Struct<T, U> {
        /// Pinned field.
        #[pin]
        pub _pinned: T,
        /// Unpinned field.
        pub _unpinned: U,
    }

    /// Testing enum.
    #[allow(clippy::exhaustive_enums)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    pub enum Enum<T, U> {
        /// Variant.
        Struct {
            /// Pinned field.
            #[pin]
            _pinned: T,
            /// Unpinned field.
            _unpinned: U,
        },
    }

    /// Test for <https://github.com/rust-lang/rust/issues/77973>.
    pub mod inside_macro {
        use pin_project::pin_project;

        /// Test lints from macro.
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                /// Testing struct.
                #[allow(clippy::exhaustive_structs, clippy::pub_underscore_fields)] // for the type itself
                #[pin_project(project_replace)]
                pub struct Struct<T, U> {
                    /// Pinned field.
                    #[pin]
                    pub _pinned: T,
                    /// Unpinned field.
                    pub _unpinned: U,
                }

                /// Testing enum.
                #[allow(clippy::exhaustive_enums)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                pub enum Enum<T, U> {
                    /// Variant.
                    Struct {
                        /// Pinned field.
                        #[pin]
                        _pinned: T,
                        /// Unpinned field.
                        _unpinned: U,
                    },
                }
            };
        }

        mac!();
    }
}

/// Test for `clippy::ref_option_ref` lint.
#[allow(missing_debug_implementations)]
pub mod clippy_ref_option_ref {
    use pin_project::pin_project;

    /// Testing struct.
    #[allow(
        clippy::exhaustive_structs,
        clippy::pub_underscore_fields,
        clippy::single_char_lifetime_names
    )] // for the type itself
    #[pin_project]
    pub struct Struct<'a> {
        /// Pinned field.
        #[pin]
        pub _pinned: Option<&'a ()>,
        /// Unpinned field.
        pub _unpinned: Option<&'a ()>,
    }

    /// Testing enum.
    #[allow(clippy::exhaustive_enums, clippy::single_char_lifetime_names)] // for the type itself
    #[pin_project(project = EnumProj, project_ref = EnumProjRef)]
    pub enum Enum<'a> {
        /// Variant.
        Struct {
            /// Pinned field.
            #[pin]
            _pinned: Option<&'a ()>,
            /// Unpinned field.
            _unpinned: Option<&'a ()>,
        },
    }
}
