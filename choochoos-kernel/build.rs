fn main() {
    // link with pre-compiled userspace (bin/libuserspace.a)
    println!("cargo:rustc-link-search=native=bin");
    println!("cargo:rustc-link-lib=static=userspace");
    println!("cargo:rerun-if-changed=src/libuserspace.a");
}
