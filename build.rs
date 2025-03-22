fn main() {
    println!("cargo:rustc-link-search=C:\\ProgramData\\chocolatey\\lib\\SQLite\\tools");
    println!("cargo:rustc-link-lib=sqlite3");
} 