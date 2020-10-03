use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

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
}
