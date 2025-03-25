fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=C:\\ProgramData\\chocolatey\\lib\\SQLite\\tools");
        println!("cargo:rustc-link-lib=sqlite3");
    }
    
    // On Linux/Unix platforms, we rely on the system sqlite or bundled-sqlite feature
    #[cfg(not(target_os = "windows"))]
    {
        // No additional link flags needed as bundled-sqlite will handle it
    }
} 