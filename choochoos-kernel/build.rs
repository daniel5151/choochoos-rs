use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // assemble and link assembly routines

    let as_result = Command::new("arm-none-eabi-as")
        .args(&["-g", "-o", &(out_dir.clone() + "/asm.o"), "src/asm.s"])
        .status()
        .unwrap();
    let ar_result = Command::new("arm-none-eabi-ar")
        .args(&[
            "-crus",
            &(out_dir.clone() + "/libasm.a"),
            &(out_dir.clone() + "/asm.o"),
        ])
        .status()
        .unwrap();

    if !as_result.success() || !ar_result.success() {
        panic!("failed to compile assembly files");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=asm");
    println!("cargo:rerun-if-changed=src/asm.s");

    // link with pre-compiled userspace (bin/libuserspace.a)

    println!("cargo:rustc-link-search=native=bin");
    println!("cargo:rustc-link-lib=static=userspace");
    println!("cargo:rerun-if-changed=src/libuserspace.a");
}
