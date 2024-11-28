fn main() {
    bindgen::Builder::default()
        .use_core()
        .header("multiboot.h")
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(
            std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bindings.rs"),
        )
        .expect("Failed to write bindings");
}
