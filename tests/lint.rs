#![warn(nonstandard_style, rust_2018_idioms, unused)]
// Note: This does not guarantee compatibility with forbidding these lints in the future.
// If rustc adds a new lint, we may not be able to keep this.
#![forbid(future_incompatible, rust_2018_compatibility)]
#![allow(unknown_lints)] // for old compilers
#![warn(
    box_pointers,
    deprecated_in_future,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
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
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
// absolute_paths_not_starting_with_crate, anonymous_parameters, keyword_idents, pointer_structural_match: forbidden as a part of future_incompatible
// unsafe_block_in_unsafe_fn: unstable
// unsafe_code: checked in forbid_unsafe module
// unstable_features: deprecated: https://doc.rust-lang.org/beta/rustc/lints/listing/allowed-by-default.html#unstable-features
// unused_crate_dependencies: unrelated
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![warn(clippy::restriction)]
#![allow(clippy::blanket_clippy_restriction_lints)] // this is a test, so enable all restriction lints intentionally.

// Check interoperability with rustc and clippy lints.

pub mod basic {
    include!("include/basic.rs");
}

pub mod forbid_unsafe {
    #![forbid(unsafe_code)]

    include!("include/basic-safe-part.rs");
}

pub mod box_pointers {
    use pin_project::pin_project;

    #[allow(box_pointers)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct {
        #[pin]
        pub p: Box<isize>,
        pub u: Box<isize>,
    }

    #[allow(box_pointers)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct(#[pin] pub Box<isize>, pub Box<isize>);

    #[allow(box_pointers)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum {
        Struct {
            #[pin]
            p: Box<isize>,
            u: Box<isize>,
        },
        Tuple(#[pin] Box<isize>, Box<isize>),
        Unit,
    }
}

pub mod explicit_outlives_requirements {
    use pin_project::pin_project;

    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<'a, T, U>
    where
        T: ?Sized,
        U: ?Sized,
    {
        #[pin]
        pub pinned: &'a mut T,
        pub unpinned: &'a mut U,
    }

    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<'a, T, U>(#[pin] pub &'a mut T, pub &'a mut U)
    where
        T: ?Sized,
        U: ?Sized;

    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
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
        Struct {
            #[pin]
            pinned: &'a mut T,
            unpinned: &'a mut U,
        },
        Tuple(#[pin] &'a mut T, &'a mut U),
        Unit,
    }
}

pub mod single_use_lifetimes {
    use pin_project::pin_project;

    #[allow(unused_lifetimes)]
    pub trait Trait<'a> {}

    #[allow(unused_lifetimes)] // for the type itself
    #[allow(single_use_lifetimes)] // for the type itself: https://github.com/rust-lang/rust/issues/55058
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct HRTB<'pin___, T>
    where
        for<'pin> &'pin T: Unpin,
        T: for<'pin> Trait<'pin>,
        for<'pin, 'pin_, 'pin__> &'pin &'pin_ &'pin__ T: Unpin,
    {
        #[pin]
        f: &'pin___ mut T,
    }
}

pub mod variant_size_differences {
    use pin_project::pin_project;

    #[allow(missing_debug_implementations, missing_copy_implementations)] // https://github.com/rust-lang/rust/pull/74060
    #[allow(variant_size_differences)] // for the type itself
    #[allow(clippy::large_enum_variant)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    pub enum Enum {
        V1(u8),
        V2([u8; 1024]),
    }
}

pub mod clippy_mut_mut {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<'a, T, U> {
        #[pin]
        pub pinned: &'a mut T,
        pub unpinned: &'a mut U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<'a, T, U> {
        Struct {
            #[pin]
            pinned: &'a mut T,
            unpinned: &'a mut U,
        },
        Tuple(#[pin] &'a mut T, &'a mut U),
        Unit,
    }
}

#[allow(unreachable_pub)]
mod clippy_redundant_pub_crate {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<T, U> {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<T, U>(#[pin] pub T, pub U);

    #[allow(dead_code)]
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<T, U> {
        Struct {
            #[pin]
            pinned: T,
            unpinned: U,
        },
        Tuple(#[pin] T, U),
        Unit,
    }
}

pub mod clippy_type_repetition_in_bounds {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<T, U>
    where
        Self: Sized,
    {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<T, U>(#[pin] T, U)
    where
        Self: Sized;

    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<T, U>
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
}

pub mod clippy_used_underscore_binding {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<T, U> {
        #[pin]
        pub _pinned: T,
        pub _unpinned: U,
    }

    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<T, U> {
        Struct {
            #[pin]
            _pinned: T,
            _unpinned: U,
        },
    }
}

#[cfg(not(miri))]
#[allow(box_pointers)]
#[allow(clippy::restriction)]
#[rustversion::attr(not(nightly), ignore)]
#[test]
fn check_lint_list() {
    use std::{env, fs, path::PathBuf, process::Command, str};

    type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

    fn assert_eq(expected_path: &str, actual: &str) -> Result<()> {
        let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR not set");
        let expected_path = manifest_dir.join(expected_path);
        let expected = fs::read_to_string(&expected_path)?;
        if expected != actual {
            if env::var_os("CI").map_or(false, |v| v == "true") {
                panic!(
                    "assertion failed:\n\nEXPECTED:\n{0}\n{1}\n{0}\n\nACTUAL:\n{0}\n{2}\n{0}\n",
                    "-".repeat(60),
                    expected,
                    actual,
                );
            } else {
                fs::write(&expected_path, actual)?;
            }
        }
        Ok(())
    }

    (|| -> Result<()> {
        let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(rustc).args(&["-W", "help"]).output()?;
        let new = str::from_utf8(&output.stdout)?;
        assert_eq("tests/lint.txt", new)
    })()
    .unwrap_or_else(|e| panic!("{}", e));
}
