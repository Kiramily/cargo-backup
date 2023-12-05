use crate::Package;
use serde::{de, ser};
use std::{
    error::Error,
    fs::{create_dir, read_to_string, write},
};

pub mod github;

pub trait RemoteProvider {
    /// Get the keyring for the provider.
    fn get_keyring() -> keyring::Entry;
    /// Initializes a new `RemoteProvider`
    fn new() -> Self;
    /// Pulls a backup from a remote server.
    fn pull(&self) -> Result<Vec<Package>, Box<dyn Error>>;
    /// Pushes a backup to a remote server.
    fn push(&self, backup: &[Package]) -> Result<(), Box<dyn Error>>;
    /// Obtain a access token for the remote server.
    fn login(&self, relogin: bool) -> Result<(), Box<dyn Error>>;
    /// Set the id for the backup.
    fn set_id(&self, id: String) -> Result<(), Box<dyn Error>>;
}

/// # Example
/// ```
/// struct SomeProvider;
///
/// impl ProviderConfig for SomeProvider {
///     fn get_name() -> String {
///         String::from("provider")
///     }
/// }
/// ```
pub(crate) trait ProviderConfig {
    /// Gets the name of the Provider.
    /// Used for the config file name.
    fn get_name() -> String;
}

pub(crate) fn get_config<T>() -> T
where
    T: de::DeserializeOwned,
    T: ser::Serialize,
    T: Default,
    T: ProviderConfig,
{
    let path = dirs::config_dir().unwrap().join("cargo-backup");

    if !path.exists() {
        create_dir(&path).unwrap();
    }

    let path = path.join(format!("{}.toml", T::get_name()));

    if path.exists() {
        let content = read_to_string(&path).unwrap();
        let config: T = toml::from_str(&content).unwrap();
        config
    } else {
        let config = T::default();
        let content = toml::to_string(&config).unwrap();
        write(&path, content).unwrap();
        config
    }
}

pub(crate) fn save_config<T>(config: T)
where
    T: ser::Serialize,
    T: ProviderConfig,
{
    let path = dirs::config_dir().unwrap().join("cargo-backup");

    if !path.exists() {
        create_dir(&path).unwrap();
    }

    let path = path.join(format!("{}.toml", T::get_name()));

    write(path, toml::to_string(&config).unwrap()).unwrap();
}
