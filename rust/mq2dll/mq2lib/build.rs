// build.rs
fn main() {
    // Add the directory where the `MQ2Rust.dll` is located to the library search path
    println!("cargo:rustc-link-search=G:\\workspace\\Macroquest\\openvanilla\\build\\bin\\debug\\plugins");

    // Link against the `MQ2Rust` DLL
    println!("cargo:rustc-link-lib=dylib=MQ2Rust");
}
