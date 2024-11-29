use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo:rerun-if-changed=kernel/{}.ld", target);
    println!("cargo:rustc-link-arg=-Tkernel/{}.ld", target);
}
