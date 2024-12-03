use std::{env, path};

fn main() {
    bindgen::Builder::default()
        .use_core()
        .header("multiboot.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(ZerocopyDeriveCallback))
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(path::PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Failed to write bindings");
}

#[derive(Debug)]
struct ZerocopyDeriveCallback;

impl bindgen::callbacks::ParseCallbacks for ZerocopyDeriveCallback {
    fn add_derives(&self, _info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        vec![
            "::zerocopy::FromBytes".to_owned(),
            "::zerocopy::KnownLayout".to_owned(),
            "::zerocopy::Immutable".to_owned(),
        ]
    }
}
