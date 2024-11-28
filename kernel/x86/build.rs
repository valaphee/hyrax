fn main() {
    let target = std::env::var("TARGET").unwrap();
    println!("cargo:rerun-if-changed=krnl/{}.ld", target);
    println!("cargo:rustc-link-arg=-Tkrnl/{}.ld", target);
}
