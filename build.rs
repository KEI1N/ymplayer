fn main() {
    // Try pkg-config first
    if pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("mpv")
        .is_ok()
    {
        return;
    }

    println!("cargo:warning=pkg-config failed, trying Homebrew path");

    // Try brew --prefix mpv for dynamic path
    if let Ok(output) = std::process::Command::new("brew")
        .args(["--prefix", "mpv"])
        .output()
    {
        if output.status.success() {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let lib_path = std::path::Path::new(&prefix).join("lib");
            if lib_path.join("libmpv.2.dylib").exists() {
                println!("cargo:rustc-link-search={}", lib_path.display());
                println!("cargo:rustc-link-lib=mpv");
                return;
            }
        }
    }

    // Fallback: search common Homebrew paths (Apple Silicon + Intel)
    let homebrew_prefixes = [
        "/opt/homebrew",      // Apple Silicon
        "/usr/local",         // Intel
    ];

    for prefix in &homebrew_prefixes {
        let cellar_path = std::path::Path::new(prefix).join("Cellar/mpv");
        if cellar_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&cellar_path) {
                for entry in entries.flatten() {
                    let lib_path = entry.path().join("lib");
                    if lib_path.join("libmpv.2.dylib").exists() {
                        println!("cargo:rustc-link-search={}", lib_path.display());
                        println!("cargo:rustc-link-lib=mpv");
                        return;
                    }
                }
            }
        }
        // Also check direct lib path
        let lib_path = std::path::Path::new(prefix).join("lib");
        if lib_path.join("libmpv.2.dylib").exists() {
            println!("cargo:rustc-link-search={}", lib_path.display());
            println!("cargo:rustc-link-lib=mpv");
            return;
        }
    }

    println!("cargo:warning=Could not find libmpv. Install it: brew install mpv");
}
