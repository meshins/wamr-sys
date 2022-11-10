extern crate bindgen;
extern crate cmake;

use std::{env, path::PathBuf};

use cmake::Config;

// use cmake::Config;
// use build_target;

fn main() {
    let target = build_target::target_arch().unwrap();
    let platform = build_target::target_triple()
        .unwrap()
        .split('-')
        .nth(2) // necessary since some targets have more info, i.e. x86_64-unknown-linux-gnu
        .unwrap()
        .to_owned();

    // FIXME: find a solution for cmake esp-idf cross build
    if platform.as_str().ne("espidf") {
        // Run cmake to build nng
        let dst = Config::new("libiwasm")
            .generator("Unix Makefiles")
            .define("CMAKE_BUILD_TYPE", "Release")
            .define("WAMR_BUILD_PLATFORM", platform.as_str())
            .define("WAMR_BUILD_TARGET", target.as_str().to_uppercase())
            .no_build_target(true)
            .build();
        // Check output of `cargo build --verbose`, should see something like:
        // -L native=/path/runng/target/debug/build/runng-sys-abc1234/out
        // That contains output from cmake
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("build").display()
        );
        println!("cargo:rustc-link-lib=iwasm");
        println!("cargo:rustc-link-lib=vmlib");
    }

    // INFO: necessary because of missing esp support of bindgen
    let clang_args = match target.as_str() {
        "riscv32" => "--target=riscv32-unknown-none-elf",
        _ => "",
    };

    let bindings = bindgen::Builder::default()
        .header("wasm-micro-runtime/core/iwasm/include/wasm_export.h")
        .clang_arg(clang_args)
        // This is needed if use `#include <nng.h>` instead of `#include "path/nng.h"`
        //.clang_arg("-Inng/src/")
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
