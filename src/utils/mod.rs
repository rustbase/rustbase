use std::path::{Path, PathBuf};

pub fn get_current_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap();

    Path::new(&exe).parent().unwrap().to_path_buf()
}
