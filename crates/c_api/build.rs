extern crate cbindgen;

use path_clean::{clean, PathClean};
use std::env;
use std::path::PathBuf;

use cbindgen::Language;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_path = PathBuf::from(&env::var("OUT_DIR").unwrap())
        .join("..") // out
        .join("..") // crate_name_UUID
        .join("..") // build
        .clean();
    let header_path = PathBuf::from(&lib_path).join("mic2.h").clean();
    println!("cargo:warning=HEADER_PATH:{header_path:#?}");

    println!("cargo:warning=LIB_PATH:{lib_path:#?}");
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(Language::C)
        .with_pragma_once(true)
        .with_include_version(true)
        .with_cpp_compat(true)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(header_path);

    println!("cargo:rerun-if-changed=build.rs");

    //verify_header(&lib_path);
}


fn verify_header(lib_path: &PathBuf) {
    let exec_path = lib_path.join("test_build").clean();
    let lib_path = lib_path.to_str().unwrap();

    let exec_path = cmake::Config::new("tests/test_build/")
      .build_target("test_build")
      .cflag(format!("-I{lib_path}"))
      .cflag(format!("-L{lib_path}"))
      //.cflag(format!("-lmic2"))
      .build();
    println!("cargo:warning=TEST_BUILD_PATH:{exec_path:#?}");



    // cc::Build::new()
    //   .file("tests/build/build.c")
    //   .include(&lib_path)
    //   .compile("test_me");

    println!("cargo:rerun-if-changed=tests/test_build/test_build.c");
    // println!("cargo:rustc-link-search=native={}", lib_path);
    // println!("cargo:rustc-link-lib=static=mic2");
}
