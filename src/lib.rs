#![warn(clippy::pedantic)]
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use duct::cmd;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod web {
    pub mod types {
        pub mod github;
    }
    pub mod github;
    pub mod provider;
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub version: Version,
}

fn slice_name(name: &str) -> (String, Version) {
    // Slice the name and Version
    let name = name.split(' ').collect::<Vec<&str>>();
    let version = Version::parse(name[1]).unwrap();

    (name[0].to_string(), version)
}

/// Get a list of installed packages
///
/// # Returns
///
/// A list of installed packages
///
/// # Panics
/// Uuuh i don't know
#[must_use]
pub fn get_packages() -> Vec<Package> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".cargo/.crates2.json");

    assert!(path.exists(), "{} does not exist", path.display());

    let packages: Crates = serde_json::from_str(std::fs::read_to_string(path).unwrap().as_str())
        .expect("Failed to parse crates2.json");

    let mut pkgs = Vec::new();

    for (name, install) in packages.installs {
        let (name, version) = slice_name(&name);

        let pkg = Package {
            name,
            version,
            features: install.features,
            all_features: install.all_features,
            no_default_features: install.no_default_features,
        };
        pkgs.push(pkg);
    }

    pkgs
}

/// Restore a backup of installed packages
///
/// # Arguments
///
/// * `packages` - A vector of packages to restore
///
fn install_package(package: &Package) {
    let name = &package.name;
    let mut args = vec!["install".to_string(), name.clone()];

    if package.no_default_features {
        args.push("--no-default-features".to_string());
    }

    if !package.features.is_empty() {
        args.push("--features".to_string());
        args.push(package.features.join(" "));
    }

    if let Err(err) = cmd("cargo", args).read() {
        eprintln!("Error while installing ({}): {}", name, err);
    }
}

/// Prompts the user to select which packages to install
///
/// # Arguments
///
/// * `packages` - The list of packages to select from
///
/// # Panics
/// Uuuh i don't know x2
///
pub fn restore(packages: &[Package]) {
    let selected_packages = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Packages to install")
        .items(
            &packages
                .iter()
                .map(|x| x.name.as_str())
                .collect::<Vec<&str>>(),
        )
        .interact()
        .unwrap();

    for selected in selected_packages {
        let package = packages.get(selected).unwrap();
        install_package(package);
    }
}
