fn main() {
    // Use VERSION env var if set (from CI), otherwise fall back to Cargo.toml
    let version = std::env::var("VERSION")
        .map(|v| v.trim_start_matches('v').to_string())
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

    println!("cargo:rustc-env=BUILD_VERSION={}", version);
    println!("cargo:rerun-if-env-changed=VERSION");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set(
            "FileDescription",
            "Convert OpenAPI schemas to Zod/TypeScript",
        );
        res.set("ProductName", "dtolator");
        res.set("ProductVersion", &version);
        res.set("FileVersion", &version);
        res.set("LegalCopyright", "Copyright (c) 2025-2026 Ville M. Vainio");
        res.compile().unwrap();
    }
}
