fn main() {
    #[allow(clippy::single_match)]
    match std::env::var("EXTERN_DISTRO") {
        Ok(distro) => {
            // link with pre-compiled userspace that's in the `bin/` folder
            println!("cargo:rustc-link-search=native=bin");
            println!("cargo:rustc-link-lib=static={}", distro);
            println!("cargo:rerun-if-changed=src/lib{}.a", distro);
        }
        Err(_) => {
            // this _should_ be a panic, but this would break tools like
            // `clippy`, so we just do nothing and let the linker error occur.
        }
    }
}
