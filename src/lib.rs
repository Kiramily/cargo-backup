use misc::{pretty_print_packages, Errors};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, vec};

mod misc;
pub mod remote;
mod url;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Package {
    pub name: String,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub version: Version,
}

#[derive(Serialize, Deserialize, Debug)]
struct Crates {
    installs: HashMap<String, Install>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Install {
    #[serde(default)]
    pub features: Vec<String>,
    pub no_default_features: bool,
    pub all_features: bool,
}

/// Returns the path to the .crates2.json file.
fn get_crates_path() -> PathBuf {
    #[cfg(test)]
    {
        use std::env;
        env::current_dir().unwrap().join("tests/.crates2.json")
    }

    #[cfg(not(test))]
    {
        let path = dirs::home_dir().unwrap().join(".cargo/.crates2.json");
        assert!(path.exists());
        path
    }
}

/// Gets the currently installed packages from the .crates2.json file.
///
/// # Examples
/// ```no_run
/// use cargo_backup::get_packages;
///
/// let packages = get_packages();
/// ```
///
/// # Panics
/// * If the .crates2.json file is not valid JSON.
/// * If the .crates2.json file cannot be read.
pub fn get_packages() -> Vec<Package> {
    let path = get_crates_path();
    let crates: Crates = serde_json::from_str(
        &std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{}", Errors::ReadFile)),
    )
    .unwrap_or_else(|_| panic!("{}", Errors::JsonParse));

    let mut packages = vec![];

    for (id, install) in crates.installs {
        let (name, version, skip) = slice_info(&id);

        if skip {
            continue;
        }

        packages.push(Package {
            name: name.to_string(),
            features: install.features,
            all_features: install.all_features,
            no_default_features: install.no_default_features,
            version,
        });
    }

    packages
}

pub fn install_packages(
    packages: &[Package],
    skip_install: bool,
    skip_update: bool,
    skip_remove: bool,
) {
    let installed_packages = get_packages();

    let mut to_update: Vec<Package> = vec![];
    let mut to_install: Vec<Package> = vec![];
    let mut to_remove: Vec<Package> = vec![];

    if !skip_install {
        for package in packages {
            if !installed_packages.iter().any(|p| p.name == package.name) {
                to_install.push(package.clone());
            }
        }
    }

    if !skip_update {
        for package in &installed_packages {
            if let Some(p) = packages.iter().find(|np| np.name == package.name) {
                if p.version > package.version {
                    to_update.push(p.clone());
                }
            }
        }
    }

    if !skip_remove {
        for package in &installed_packages {
            if !packages.iter().any(|np| np.name == package.name) {
                to_remove.push(package.clone());
            }
        }
    }

    pretty_print_packages(&to_install.clone(), &to_update.clone(), &to_remove.clone());

    // Skip the Installation process if it is a test
    #[cfg(not(test))]
    {
        use crate::misc::{execute_cmd, CommandType};
        use dialoguer::Confirm;

        if Confirm::new().with_prompt("Proceed?").interact().unwrap() {
            // TODO: Install

            for package in to_install {
                execute_cmd(&package, CommandType::Install);
            }

            for package in to_update {
                execute_cmd(&package, CommandType::Install);
            }

            for package in to_remove {
                execute_cmd(&package, CommandType::Remove);
            }
        }
    }
}

/// Gets the Package name and Version and from the string.
/// The bool will be true if the Package is a local package and it should be skipped.
///
/// # Examples
/// ```no_run
/// let (name, version, skip) = slice_info("foo 0.1.0 (path+file:///home/user/foo)");
/// ```
fn slice_info(package_str: &str) -> (String, Version, bool) {
    let splits: Vec<&str> = package_str.splitn(3, ' ').collect();
    let name = splits[0].to_string();
    let version = Version::parse(splits[1]).unwrap();
    let local_package = splits[2].contains("path+file://");
    (name, version, local_package)
}

#[test]
fn test_slice_info() {
    use std::str::FromStr;

    let (name, version, skip) = slice_info("foo 0.1.0 (path+file:///home/user/foo)");
    assert_eq!(name, "foo");
    assert_eq!(version, Version::from_str("0.1.0").unwrap());
    assert!(skip);

    let (name, version, skip) = slice_info("foo 0.1.0 (registry+https://example.com/foo)");
    assert_eq!(name, "foo");
    assert_eq!(version, Version::from_str("0.1.0").unwrap());
    assert!(!skip);
}

#[test]
fn test_get_packages() {
    let packages = get_packages();
    assert_eq!(packages.len(), 3);
}

#[test]
fn test_install_packages() {
    let fake_packages: Vec<Package> = vec![
        Package {
            all_features: true,
            features: vec![],
            name: "foo".to_string(),
            no_default_features: false,
            version: Version::parse("0.1.0").unwrap(),
        },
        Package {
            name: "package".to_string(),
            version: Version::parse("0.5.3").unwrap(),
            all_features: false,
            no_default_features: false,
            features: vec!["feature1".to_string(), "feature2".to_string()],
        },
    ];

    install_packages(&fake_packages, false, false, false);
}
