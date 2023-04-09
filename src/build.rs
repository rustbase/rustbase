fn main() {
    println!(
        "cargo:rustc-env=VERSION_CODE={}",
        std::env::var("VERSION_CODE").unwrap()
    );
}
