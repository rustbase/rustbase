#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut winresouces = winres::WindowsResource::new();

    winresouces.set_icon("./assets/icon.ico");
    winresouces.compile().unwrap();
}

// #[cfg(unix)]
// fn main() {

// }