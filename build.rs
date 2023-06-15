use std::{env, path};

fn main() {
    let manifest_path = env::var("CARGO_MANIFEST_DIR")
        .map(path::PathBuf::from)
        .expect("Env variable `CARGO_MANIFEST_DIR` should be set");

    #[cfg(feature = "generate-bindings")]
    generate_bindings(&manifest_path).expect("Failed to generate bindings");
    link_libraries(&manifest_path);
}

#[cfg(feature = "generate-bindings")]
fn generate_bindings(manifest_path: &path::Path) -> std::io::Result<()> {
    fn collect_headers_in(path: &path::Path) -> std::io::Result<Vec<String>> {
        let mut headers = Vec::new();
        for entry in path.read_dir()? {
            let path = entry?.path();
            if path.is_file() && path.extension().map(|ext| ext == "h").unwrap_or_default() {
                headers.push(path.to_str().expect("Invalid UTF-8 string").to_string());
            }
        }
        Ok(headers)
    }

    let sdk_path = manifest_path.join("SDK").join("CHeaders");
    let mut headers = Vec::new();
    let widgets_sdk_path = sdk_path.join("Widgets");
    headers.extend(collect_headers_in(&widgets_sdk_path)?);
    let xplm_sdk_path = sdk_path.join("XPLM");
    headers.extend(collect_headers_in(&xplm_sdk_path)?);

    // https://developer.x-plane.com/sdk/plugin-sdk-downloads
    let mut builder = bindgen::Builder::default();

    for header in headers {
        builder = builder.header(header);
    }

    let bindings = builder
        .allowlist_function("XP.*")
        .allowlist_type("XP.*")
        .allowlist_var("XPLM_VK_.*")
        .allowlist_var("xplm_Command.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args([
            "-fparse-all-comments",
            "-DLIN=1",
            "-DXPLM200", // X-Plane 9.00 & newer
            "-DXPLM210", // X-Plane 10.00 & newer (10.20 required for 64-bit plugins)
            "-DXPLM300", // X-Plane 11.10 & newer (64-bit only)
            "-DXPLM301", // X-Plane 11.20 & newer (64-bit only)
            "-DXPLM303", // X-Plane 11.50 & newer (64-bit only)
            "-DXPLM400", // X-Plane 12.04 & newer (64-bit only)
            &format!("-I{}", xplm_sdk_path.display()),
            &format!("-I{}", widgets_sdk_path.display()),
        ])
        .generate()
        .expect("Failed to generate bindings");

    let out_path = manifest_path.join("src").join("bindings.rs");
    bindings.write_to_file(out_path)
}

fn link_libraries(manifest_path: &path::Path) {
    let library_path = manifest_path.join("SDK").join("Libraries");

    if cfg!(target_os = "windows") {
        println!(
            "cargo:rustc-link-search={}",
            library_path.join("Win").display()
        );
        println!("cargo:rustc-link-lib=XPLM_64");
        println!("cargo:rustc-link-lib=XPWidgets_64");
    } else if cfg!(target_os = "macos") {
        println!(
            "cargo:rustc-link-search=framework={}",
            library_path.join("Mac").display()
        );
        println!("cargo:rustc-link-lib=framework=XPLM");
        println!("cargo:rustc-link-lib=framework=XPWidgets");
    }
}
