fn main() {
    let distro = std::env::var("DISTRO").unwrap_or(String::new());

    // link with pre-compiled userspace that's in the `bin/` folder
    println!("cargo:rustc-link-search=native=bin");
    println!("cargo:rustc-link-lib=static={}", distro);
    println!("cargo:rerun-if-changed=src/lib{}.a", distro);
}
