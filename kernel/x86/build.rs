use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo:rerun-if-changed=kernel/x86/{}.ld", target);
    println!("cargo:rustc-link-arg=-Tkernel/x86/{}.ld", target);
}
