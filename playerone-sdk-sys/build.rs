use std::env;
use std::path::PathBuf;

use bindgen::EnumVariation;

const HEADER_PATH: &str = "libs/PlayerOneCamera.h";

fn main() {
    if cfg!(target_os = "windows") {
        unimplemented!("Windows is not supported yet");
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            println!("cargo:rustc-link-search=native=libs/linux/arm64");
        } else if cfg!(target_arch = "x86_64") {
            println!("cargo:rustc-link-search=native=libs/linux/x64");
        } else {
            panic!("Unsupported architecture");
        }
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=native=libs/mac");
    } else {
        unimplemented!("Unsupported OS");
    }
    println!("cargo:rustc-link-lib=static=PlayerOneCamera_Static");

    let bindings = bindgen::Builder::default()
        .header(HEADER_PATH)
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
