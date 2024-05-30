use path_clean::PathClean;
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

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(Language::C)
        .with_pragma_once(true)
        .with_include_version(true)
        .with_cpp_compat(true)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&header_path);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/mic2.hpp");
    println!("cargo:rerun-if-changed=src/mic2.cpp");
    println!("cargo:rerun-if-changed=../../Cargo.lock");

    // Copy the C++ files
    std::fs::copy("src/mic2.cpp", lib_path.join("mic2.cpp")).unwrap();
    std::fs::copy("src/mic2.hpp", lib_path.join("mic2.hpp")).unwrap();

    // Debugging paths:
    println!("cargo:warning=OUT_PATH:{:#?}", env::var("OUT_DIR").unwrap());
    println!("cargo:warning=HEADER_PATH:{:#?}", &header_path);
    println!("cargo:warning=LIB_PATH:{lib_path:#?}");
}
