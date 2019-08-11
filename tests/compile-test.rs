#![cfg(compile_test)]
#![cfg(feature = "project_attr")]
#![warn(rust_2018_idioms)]

use std::{env, path::PathBuf};

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();
    config.mode = mode.parse().expect("invalid mode");
    let mut me = env::current_exe().unwrap();
    me.pop();
    config.target_rustcflags = Some(format!(
        "--edition=2018 \
         -Z unstable-options \
         --extern pin_project \
         -L {}",
        me.display()
    ));
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    config.src_base = src.join("tests").join(mode);

    me.pop();
    me.pop();
    config.build_base = me.join("tests").join(mode);
    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("ui");
}
