#[cfg(any(target_os = "linux", target_os = "macos"))]
const LIBTIKA_PATH: &str = "tika_native";
#[cfg(target_os = "windows")]
const LIBTIKA_PATH: &str = "libtika_native";

fn main() {
    /*
     * link to libtika
     */
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", manifest_dir);
    println!("cargo:rustc-link-lib=dylib={}", LIBTIKA_PATH);

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=awt");
    }

    /*
     * Add a rpath to the directory that contains the binary
     */
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
}
