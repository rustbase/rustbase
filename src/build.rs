fn main() {
    if cfg!(target_os = "windows") {
        extern crate winres;
        let mut winresouces = winres::WindowsResource::new();

        winresouces.set_icon("./assets/icon.ico");
        winresouces.compile().unwrap();
    }

    tonic_build::configure()
        .compile(&["proto/rustbase.proto"], &["proto"])
        .unwrap();
}
