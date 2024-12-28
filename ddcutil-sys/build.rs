#[cfg(feature = "bindgen")]
pub fn main() {
    println!("cargo:rustc-link-lib=ddcutil");
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));

    let bindings = bindgen::Builder::default()
        .header("headers/ddcutil.h")
        .header("headers/version.h")
        .allowlist_file("ddcutil_c_api.h")
        .generate()
        .expect("Bindings");
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
#[cfg(not(feature = "bindgen"))]
pub fn main() {
    if pkg_config::probe_library("ddcutil").is_ok() {
        println!("ddcutil found via pkg-config");
    } else {
        println!("cargo:rustc-link-lib=ddcutil");
    }
}
