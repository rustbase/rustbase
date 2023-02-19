use crate::utils;
use std::io::Cursor;

const RELEASE_DOWNLOAD_URL: &str = "https://github.com/rustbase/rustbase/releases";

fn parse_current_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let platform = match os {
        "linux" => "linux",
        "windows" => "windows",
        _ => panic!("Unsupported platform: {}", os),
    };

    let arch = match arch {
        "x86_64" => "x64",
        "x86" => "x86",
        _ => panic!("Unsupported architecture: {}", arch),
    };

    format!("{}-{}", platform, arch)
}

pub async fn upgrade_rustbase(version: Option<String>) {
    let version = version.unwrap_or_else(|| "latest".to_string());
    let platform = parse_current_platform();

    get_release(version, platform).await;
}

async fn get_release(version: String, platform: String) {
    let url = if version == "latest" {
        format!("{RELEASE_DOWNLOAD_URL}/{version}/download/rustbase-{platform}.zip")
    } else {
        format!("{RELEASE_DOWNLOAD_URL}/download/{version}/rustbase-{platform}.zip")
    };

    println!("[Upgrade] Upgrading to version: {}", version);

    let response = reqwest::get(&url).await.unwrap();
    let status = response.status().as_u16();
    let mut content = Cursor::new(response.bytes().await.unwrap());

    if status != 200 {
        match status {
            404 => {
                println!("[Upgrade] Release not found");
                std::process::exit(1);
            }

            _ => {
                println!("[Upgrade] Error: {}", status);
                std::process::exit(1);
            }
        }
    }

    let tempdir = utils::get_current_path().join("tmp");

    if !tempdir.exists() {
        std::fs::create_dir_all(tempdir.clone()).unwrap();
    }

    let filepath = tempdir.join("rustbase.zip");

    let mut file = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(filepath)
        .unwrap();
    std::io::copy(&mut content, &mut file).unwrap();

    let mut unzip = zip::ZipArchive::new(file).unwrap();
    let rustbase_bin = unzip.by_index(0).unwrap().name().to_string();
    unzip.extract(tempdir.clone()).unwrap();

    let rustbase_bin = tempdir.join(rustbase_bin);

    let current_exe = std::env::current_exe().unwrap();

    let current_exe_temp = tempdir.join("rustbase_server.old");

    std::fs::rename(current_exe.clone(), current_exe_temp).unwrap();
    std::fs::rename(rustbase_bin, current_exe).unwrap();

    std::fs::remove_dir_all(tempdir).unwrap();
    println!("[Upgrade] Done!");
}
