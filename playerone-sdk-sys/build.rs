use std::env;
use std::path::{Path, PathBuf};

use bindgen::EnumVariation;

const HEADER_PATH: &str = "libs/PlayerOneCamera.h";

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&dir).join("libs");

    match env::consts::OS {
        "windows" => unimplemented!("Windows is not supported yet"),
        "linux" => match env::consts::ARCH {
            "aarch64" => {
                println!(
                    "cargo:rustc-link-search=native={}",
                    path.join("linux").join("arm64").display()
                );
            }
            "x86_64" => {
                println!(
                    "cargo:rustc-link-search=native={}",
                    path.join("linux").join("x64").display()
                );
            }
            _ => unimplemented!("Unsupported architecture"),
        },
        "macos" => {
            println!(
                "cargo:rustc-link-search=native={}",
                path.join("mac").display()
            );
        }
        _ => unimplemented!("Unsupported OS"),
    }

    println!("cargo:rustc-link-lib=PlayerOneCamera");

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
