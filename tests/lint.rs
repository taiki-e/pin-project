#![warn(rust_2018_idioms, single_use_lifetimes)]
#![warn(future_incompatible, nonstandard_style, rust_2018_compatibility, unused)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[allow(unknown_lints)] // for old compilers
#[warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    box_pointers,
    confusable_idents,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    non_ascii_idents,
    private_doc_tests,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
// unused_crate_dependencies: unrelated
// unsafe_code: checked in forbid_unsafe module
// unsafe_block_in_unsafe_fn: unstable
pub mod basic {
    include!("include/basic.rs");
}

pub mod forbid_unsafe {
    #![forbid(unsafe_code)]

    include!("include/basic-safe-part.rs");
}

pub mod clippy {
    use pin_project::pin_project;

    #[pin_project(Replace)]
    pub struct MutMutStruct<'a, T, U> {
        #[pin]
        pub pinned: &'a mut T,
        pub unpinned: &'a mut U,
    }

    #[pin_project(Replace)]
    pub struct MutMutTupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

    #[pin_project(Replace)]
    pub enum MutMutEnum<'a, T, U> {
        Struct {
            #[pin]
            pinned: &'a mut T,
            unpinned: &'a mut U,
        },
        Tuple(#[pin] &'a mut T, &'a mut U),
        Unit,
    }

    #[pin_project(Replace)]
    pub struct TypeRepetitionInBoundsStruct<T, U>
    where
        Self: Sized,
    {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }

    #[pin_project(Replace)]
    pub struct TypeRepetitionInBoundsTupleStruct<T, U>(#[pin] T, U)
    where
        Self: Sized;

    #[pin_project(Replace)]
    pub enum TypeRepetitionInBoundsEnum<T, U>
    where
        Self: Sized,
    {
        Struct {
            #[pin]
            pinned: T,
            unpinned: U,
        },
        Tuple(#[pin] T, U),
        Unit,
    }

    #[pin_project(Replace)]
    pub struct UsedUnderscoreBindingStruct<T, U> {
        #[pin]
        pub _pinned: T,
        pub _unpinned: U,
    }

    #[pin_project(Replace)]
    pub enum UsedUnderscoreBindingEnum<T, U> {
        Struct {
            #[pin]
            _pinned: T,
            _unpinned: U,
        },
    }
}

#[rustversion::attr(not(nightly), ignore)]
#[test]
fn check_lint_list() {
    use std::{env, process::Command};

    (|| -> Result<(), Box<dyn std::error::Error>> {
        let current = include_str!("lint.txt");
        let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let new = String::from_utf8(Command::new(rustc).args(&["-W", "help"]).output()?.stdout)?;
        assert_eq!(current, &new);
        Ok(())
    })()
    .unwrap_or_else(|e| panic!("{}", e));
}
