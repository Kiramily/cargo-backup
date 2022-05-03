use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Crates {
    installs: HashMap<String, Install>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Install {
    #[serde(default)]
    pub features: Vec<String>,
    pub no_default_features: bool,
    pub all_features: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
}

fn slice_name(name: String) -> String {
    name[..name.find(' ').unwrap_or(name.len())].to_string()
}

pub fn get_packages() -> Vec<Package> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".cargo/.crates2.json");

    if !path.exists() {
        panic!("{} does not exist", path.display());
    }

    let packages: Crates = serde_json::from_str(std::fs::read_to_string(path).unwrap().as_str())
        .expect("Failed to parse crates2.json");

    let mut pkgs = Vec::new();

    for (name, install) in packages.installs {
        let pkg = Package {
            name: slice_name(name),
            features: install.features,
            all_features: install.all_features,
            no_default_features: install.no_default_features,
        };
        pkgs.push(pkg);
    }

    pkgs
}
