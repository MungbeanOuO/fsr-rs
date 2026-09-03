use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=include/wrapper.h");
    println!("cargo:rerun-if-changed=vendor/api/include");
    println!("cargo:rerun-if-changed=vendor/upscalers/include");
    println!("cargo:rerun-if-changed=vendor/framegeneration/include");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let wrapper_path = manifest_dir.join("include").join("wrapper.h");

    let bindings = bindgen::Builder::default()
        .header(wrapper_path.to_str().unwrap())
        .clang_arg("-Iinclude")
        .clang_arg("-Ivendor/api/include")
        .clang_arg("-Ivendor/upscalers/include")
        .clang_arg("-Ivendor/framegeneration/include")
        .allowlist_type("ffx.*")
        .allowlist_type("Ffx.*")
        .allowlist_type("PfnFfx.*")
        .allowlist_function("ffx.*")
        .allowlist_var("FFX_.*")
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .prepend_enum_name(false)
        .layout_tests(true)
        .generate()
        .expect("Unable to generate bindings for AMD FidelityFX / ffx_api");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings.rs");
}
