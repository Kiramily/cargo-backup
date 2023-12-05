use std::{collections::HashMap, thread, time::Duration};

use super::{get_config, save_config, ProviderConfig, RemoteProvider};
use crate::{url::UrlBuilder, Package};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct Github {
    keyring: keyring::Entry,
    config: Config,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    gist_id: Option<String>,
}

impl ProviderConfig for Config {
    fn get_name() -> String {
        String::from("github")
    }
}

impl Github {
    fn get_auth(&self) -> Option<String> {
        if let Ok(content) = self.keyring.get_password() {
            return Some(content);
        }
        None
    }
}

impl RemoteProvider for Github {
    fn get_keyring() -> keyring::Entry {
        keyring::Entry::new("cargo-backup", "github").expect("Could not open keyring")
    }

    fn new() -> Self {
        Self {
            keyring: Self::get_keyring(),
            config: get_config(),
        }
    }

    fn pull(&self) -> Result<Vec<crate::Package>, Box<dyn std::error::Error>> {
        let auth = self
            .get_auth()
            .unwrap_or_else(|| panic!("Please login first with \"cargo sync login\""));

        let gist_id: String = self
            .config
            .gist_id
            .as_ref()
            .unwrap_or_else(|| panic!("Gist Id not set"))
            .to_owned();

        let response: Gist = ureq::get(
            &UrlBuilder::new(&format!("https://api.github.com/gists/{}", gist_id)).build(),
        )
        .set("Accept", "application/json")
        .set("Authorization", &format!("token {}", auth))
        .set("Content-Type", "application/json")
        .set(
            "User-Agent",
            &format!("CargoBackup/{}", env!("CARGO_PKG_VERSION")),
        )
        .set("Authorization", &format!("token {}", auth))
        .call()?
        .into_json()?;

        let mut packages: Vec<Package> = Vec::new();
        for file in response.files.values() {
            if file.filename == "backup.json" {
                packages = serde_json::from_str(file.content.as_ref().unwrap())?;
            }
        }
        Ok(packages)
    }

    fn push(&self, backup: &[crate::Package]) -> Result<(), Box<dyn std::error::Error>> {
        let auth = self
            .get_auth()
            .unwrap_or_else(|| panic!("Please login first with \"cargo sync login\""));

        let gist_id = self.config.gist_id.as_ref();

        let request = match gist_id {
            Some(id) => ureq::patch(&format!("https://api.github.com/gists/{}", id)),
            None => ureq::post("https://api.github.com/gists"),
        };

        let result = request
            .set("Accept", "application/json")
            .set("Authorization", &format!("token {}", auth))
            .set("Content-Type", "application/json")
            .set(
                "User-Agent",
                &format!("CargoBackup/{}", env!("CARGO_PKG_VERSION")),
            )
            .send_json(json!({
                "description": "Cargo Package Backup (Created by cargo-backup https://github.com/Kiramily/cargo-backup)",
                "public": false,
                "files": {
                    "backup.json": {
                        "content": serde_json::to_string(&backup).unwrap()
                    }
                }
            }))?;

        if result.status() == 200 {
            println!("Successfully pushed backup");
        } else {
            println!("Failed to push backup");
        }
        Ok(())
    }

    fn login(&self, relogin: bool) -> Result<(), Box<dyn std::error::Error>> {
        if relogin {
            let _ = self.keyring.delete_password();
        }

        if self.keyring.get_password().is_ok() {
            return Ok(());
        }

        let device_login: Code = ureq::post(
            &UrlBuilder::new("https://github.com/login/device/code")
                .add_param("client_id", "65102f4f3d896bfc9c1a")
                .add_param("scope", "gist")
                .build(),
        )
        .set("Accept", "application/json")
        .call()?
        .into_json()?;

        println!("Open the following URL in your browser and enter the code.");
        println!("{}", device_login.verification_uri.green());
        println!("{}", device_login.user_code.blue().bold());

        let mut has_token = false;

        while !has_token {
            thread::sleep(Duration::from_secs(device_login.interval));
            let poll_request = ureq::post(
                &UrlBuilder::new("https://github.com/login/oauth/access_token")
                    .add_param("client_id", "65102f4f3d896bfc9c1a")
                    .add_param("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
                    .add_param("device_code", &device_login.device_code)
                    .build(),
            )
            .set("Accept", "application/json")
            .call()?;

            let poll_request: LoginStatus = poll_request.into_json()?;

            match poll_request.error {
                LoginError::None => {
                    self.keyring
                        .set_password(&poll_request.access_token.unwrap())
                        .unwrap();
                    has_token = true;
                    println!("Successfull Login");
                }
                LoginError::AccessDenied => panic!("Access Denied"),
                LoginError::IncorrectClientCredentials => panic!("Incorrect Client Credentials"),
                LoginError::ExpiredToken => panic!("Token expired"),
                LoginError::IncorrectDeviceCode => panic!("Incorrect Device code"),
                LoginError::SlowDown => todo!("Handle Slow down request"),
                LoginError::AuthorizationPending => {}
                LoginError::UnsupportedGrantType => panic!("Unsupported grant type"),
            }
        }

        Ok(())
    }

    fn set_id(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut config: Config = self.config.to_owned();
        config.gist_id = Some(id);
        save_config(config);
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct Code {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u64,
}

#[derive(Deserialize)]
struct LoginStatus {
    #[serde(default)]
    error: LoginError,
    access_token: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoginError {
    AuthorizationPending,
    SlowDown,
    ExpiredToken,
    UnsupportedGrantType,
    IncorrectClientCredentials,
    IncorrectDeviceCode,
    AccessDenied,
    #[default]
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Gist {
    #[serde(rename = "id", skip_serializing)]
    pub id: String,

    #[serde(rename = "files", default)]
    pub files: HashMap<String, File>,

    #[serde(rename = "public", default)]
    pub public: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    #[serde(rename = "filename")]
    pub filename: String,

    #[serde(rename = "raw_url", skip_serializing)]
    pub raw_url: Option<String>,

    pub content: Option<String>,
}
