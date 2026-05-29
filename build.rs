use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=ext/wrapper_gstd.h");

    println!("cargo:rerun-if-env-changed=GSTD_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=GSTD_LIB_DIR");

    let include_dir =
        env::var("GSTD_INCLUDE_DIR").unwrap_or_else(|_| "/usr/local/include/gstd".to_string());
    let lib_dir = env::var("GSTD_LIB_DIR").unwrap_or_else(|_| "/usr/local/lib".to_string());

    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=gstd-1.0");

    let gstreamer = pkg_config::Config::new()
        .probe("gstreamer-1.0")
        .expect("failed to find gstreamer-1.0 via pkg-config");

    // Generate bindings from the wrapper header.
    let builder = bindgen::Builder::default()
        .header("ext/wrapper_gstd.h")
        .clang_arg(format!("-I{}", include_dir));

    let builder = gstreamer
        .include_paths
        .iter()
        .fold(builder, |builder, path| {
            builder.clang_arg(format!("-I{}", path.display()))
        });

    let bindings = builder
        .allowlist_function("gstd_new")
        .allowlist_function("gstd_start")
        .allowlist_function("gstd_stop")
        .allowlist_function("gstd_free")
        .allowlist_type("GstD")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("failed to generate bindings for gstd");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings.rs");
}
