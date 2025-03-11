use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let binaryen_version = "version_122";
    let binaryen_release_url = format!(
        "https://github.com/WebAssembly/binaryen/archive/refs/tags/{}.tar.gz",
        binaryen_version
    );

    let out_path = PathBuf::from(&out_dir);
    let download_path = out_path.join("binaryen.tar.gz");
    let extract_dir = out_path.join("binaryen");

    // Download Binaryen
    println!("cargo:rerun-if-changed=build.rs");

    // Cache download file if possible.
    if !download_path.exists() {
        let mut resp = ureq::get(binaryen_release_url)
            .call()
            .expect("Failed to download file");
        let mut reader = resp.body_mut().as_reader();
        let mut file = fs::File::create(&download_path).unwrap();
        std::io::copy(&mut reader, &mut file).expect("failed to write downloaded file");
    }

    // Extract the archive
    let tar_gz = fs::File::open(&download_path).expect("failed to open downloaded file");
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive
        .unpack(&extract_dir)
        .expect("failed to unpack archive");

    // Build with CMake
    cmake::Config::new(&extract_dir.join(format!("binaryen-{}", binaryen_version)))
        .no_default_flags(true)
        .define("BUILD_TESTS", "OFF")
        .define("BUILD_TOOLS", "OFF")
        .define("BUILD_STATIC_LIB", "ON")
        .define("ENABLE_WERROR", "OFF")
        .build();

    // Generate bindings
    generate_bindings(&out_path).expect("Failed to generate bindings");

    // Link with the compiled library
    println!("cargo:rustc-link-search=native={}/lib", out_path.display());
    println!("cargo:rustc-link-lib=static=binaryen");
    if let Some(cpp_stdlib) = get_cpp_stdlib() {
        println!("cargo:rustc-link-lib={}", cpp_stdlib);
    }
}

fn generate_bindings(build_dir: &Path) -> std::io::Result<()> {
    let bindings = bindgen::Builder::default()
        .header_contents(
            "wrapper.h",
            r#"
            #include "binaryen-c.h"
            "#,
        )
        .clang_arg(format!("-I{}/include", build_dir.display()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}

// See https://github.com/alexcrichton/gcc-rs/blob/88ac58e25/src/lib.rs#L1197
fn get_cpp_stdlib() -> Option<String> {
    std::env::var("TARGET").ok().and_then(|target| {
        if target.contains("msvc") {
            None
        } else if target.contains("darwin") {
            Some("c++".to_string())
        } else if target.contains("freebsd") {
            Some("c++".to_string())
        } else if target.contains("musl") {
            Some("static=stdc++".to_string())
        } else {
            Some("stdc++".to_string())
        }
    })
}
