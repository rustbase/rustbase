fn main() {
    if cfg!(windows) {
        extern crate winres;
        let mut winresouces = winres::WindowsResource::new();

        winresouces.set_icon("./assets/icon.ico");
        winresouces.compile().unwrap();
    }

    println!(
        "cargo:rustc-env=VERSION_CODE={}",
        std::env::var("VERSION_CODE").unwrap()
    );
}
