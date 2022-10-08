use std::collections::BTreeMap;
use std::fs;
use std::path;
use std::sync::Arc;
use std::sync::Mutex;

use colored::Colorize;
use dustdata::DustData;

use super::main::default_dustdata_config;

pub fn get_existing_routes(data_path: String) -> Vec<String> {
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

pub fn initialize_dustdata(path: String) -> Arc<Mutex<BTreeMap<String, DustData>>> {
    let mut routers = BTreeMap::new();
    let routes = get_existing_routes(path.clone());

    if !routes.is_empty() {
        for route in routes {
            let dd = dustdata::initialize(default_dustdata_config(
                path::Path::new(&path)
                    .join(&route)
                    .to_str()
                    .unwrap()
                    .to_string(),
            ));

            routers.insert(route, dd);
        }
    }

    Arc::new(Mutex::new(routers))
}
