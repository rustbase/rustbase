use std::collections::HashMap;
use std::fs;
use std::path;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;

use colored::Colorize;
use dustdata::DustData;

use super::main::default_dustdata_config;

pub fn get_existing_routes(data_path: &Path) -> Vec<String> {
    let mut routes = Vec::new();

    if !path::Path::new(&data_path).exists() {
        fs::create_dir_all(&data_path).unwrap();
        return routes;
    }

    for entry in std::fs::read_dir(data_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let route = path.file_name().unwrap().to_str().unwrap().to_string();
        routes.push(route);
    }

    println!(
        "[Route] {} databases found",
        routes.len().to_string().green()
    );

    routes
}

pub fn initialize_dustdata(path: &Path) -> Arc<RwLock<HashMap<String, DustData>>> {
    let mut routers = HashMap::new();
    let routes = get_existing_routes(path);

    let dd = dustdata::initialize(default_dustdata_config(&path.join("_default")));
    routers.insert("_default".to_string(), dd);

    if !routes.is_empty() {
        for route in routes {
            if route == "_default" {
                continue;
            }

            let dd = dustdata::initialize(default_dustdata_config(&path.join(&route)));

            routers.insert(route, dd);
        }
    }

    Arc::new(RwLock::new(routers))
}

pub fn remove_dustdata(data_path: &Path, route: String) {
    let path = path::Path::new(&data_path).join(route);

    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
}

pub fn create_dustdata(path: &Path) -> DustData {
    dustdata::initialize(default_dustdata_config(path))
}
