#![warn(rust_2018_idioms, single_use_lifetimes)]
#![warn(nonstandard_style, rust_2018_compatibility, unused)]
// Note: This does not guarantee compatibility with `forbid(future_incompatible)` in the future.
// If rustc adds a new lint, we may not be able to keep this.
#![forbid(future_incompatible)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(unknown_lints)] // for old compilers
#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    box_pointers,
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

// Check interoperability with rustc and clippy lints.

pub mod basic {
    include!("include/basic.rs");
}

pub mod forbid_unsafe {
    #![forbid(unsafe_code)]

    include!("include/basic-safe-part.rs");
}

pub mod clippy {
    use pin_project::pin_project;

    #[rustversion::attr(before(1.37), allow(single_use_lifetimes))] // https://github.com/rust-lang/rust/issues/53738
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct MutMutStruct<'a, T, U> {
        #[pin]
        pub pinned: &'a mut T,
        pub unpinned: &'a mut U,
    }

    #[rustversion::attr(before(1.37), allow(single_use_lifetimes))] // https://github.com/rust-lang/rust/issues/53738
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct MutMutTupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

    #[rustversion::attr(before(1.37), allow(single_use_lifetimes))] // https://github.com/rust-lang/rust/issues/53738
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub enum MutMutEnum<'a, T, U> {
        Struct {
            #[pin]
            pinned: &'a mut T,
            unpinned: &'a mut U,
        },
        Tuple(#[pin] &'a mut T, &'a mut U),
        Unit,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TypeRepetitionInBoundsStruct<T, U>
    where
        Self: Sized,
    {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TypeRepetitionInBoundsTupleStruct<T, U>(#[pin] T, U)
    where
        Self: Sized;

    #[pin_project(project_replace)]
    #[derive(Debug)]
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

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct UsedUnderscoreBindingStruct<T, U> {
        #[pin]
        pub _pinned: T,
        pub _unpinned: U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub enum UsedUnderscoreBindingEnum<T, U> {
        Struct {
            #[pin]
            _pinned: T,
            _unpinned: U,
        },
    }
}

#[allow(box_pointers)]
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
