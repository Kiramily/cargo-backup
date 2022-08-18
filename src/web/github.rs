use super::provider::Backup;
use crate::web::types::github::{Code, Gist, OAuth, OAuthErrorCode};
use crate::{get_packages, Package};
use async_trait::async_trait;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::thread;

pub struct Api {
    config_dir: PathBuf,
    keyring: keyring::Entry,
}

#[async_trait]
impl Backup for Api {
    fn get_keyring() -> keyring::Entry { 
        keyring::Entry::new("cargo-backup", "github")
    }

    async fn new() -> Self {
        let mut config_dir = dirs::config_dir().unwrap();

        config_dir.push("cargo-backup");

        if !config_dir.exists() {
            fs::create_dir(&config_dir).unwrap();
        }

        let keyring = Self::get_keyring();

        Api { config_dir, keyring }
    }

    async fn login(self) -> Result<(), Box<dyn Error>> {
        if let Ok(_) = self.keyring.get_password() {
            return Ok(());
        }

        // Request a device code from GitHub
        let device_login: Code = ureq::post(&format!(
            "https://github.com/login/device/code?client_id={}&scope={}",
            "65102f4f3d896bfc9c1a", "gist"
        ))
        .set("Accept", "application/json")
        .call()
        .expect("Failed to send request")
        .into_json()?;

        println!("Open the following URL in your browser and enter the code.");
        println!("{}", device_login.verification_uri);
        println!("{}", device_login.user_code);

        let mut has_token = false;
        let mut auth: Option<OAuth> = None;

        while !has_token {
            thread::sleep(std::time::Duration::from_secs(u64::from(
                device_login.interval,
            )));
            // Check if the user has entered the code
            let poll_req: OAuth = ureq::post(&format!(
                "https://github.com/login/oauth/access_token?client_id={}&grant_type={}&device_code={}",
                "65102f4f3d896bfc9c1a", "urn:ietf:params:oauth:grant-type:device_code",
				device_login.device_code
            ))
            .set("Accept", "application/json")
            .call()
            .expect("Failed to send request")
            .into_json()?;

            match poll_req.error {
                OAuthErrorCode::AuthorizationPending => {
                    //? Maybe check if 15 min has passed and exit the loop
                }
                OAuthErrorCode::SlowDown => todo!("Handle SlowDown"),
                OAuthErrorCode::ExpiredToken => panic!("Token expired"),
                OAuthErrorCode::UnsupportedGrantType => panic!("Unsupported grant type"),
                OAuthErrorCode::IncorrectClientCredentials => {
                    panic!("Incorrect client credentials")
                }
                OAuthErrorCode::IncorrectDeviceCode => panic!("Incorrect device code"),
                OAuthErrorCode::AccessDenied => panic!("Access denied"),
                OAuthErrorCode::None => {
                    // We have a token
                    auth = Some(poll_req);
                    has_token = true;
                }
            }
        }

        assert!(auth.is_some());
        let auth = auth.unwrap();

        self.keyring.set_password(&serde_json::to_string(&auth).expect("Failed to create json string")).expect("Can't save Access Token in the Keyring");

        Ok(())
    }

    async fn fetch_backup(self) -> Result<Vec<Package>, Box<dyn Error>> {
        let auth: OAuth = serde_json::from_str(&self.keyring.get_password().expect("login first")).expect("Failed to parse json string");


        if auth.gist_id.is_none() {
            println!("Set the gist id via 'set-id' command before pulling");
            exit(0);
        }


        // Read the gist
        let gist: Gist = ureq::get(&format!(
            "https://api.github.com/gists/{}",
            auth.gist_id.unwrap()
        ))
        .set("Accept", "application/json")
        .set("Authorization", &format!("token {}", auth.access_token))
        .set("Content-Type", "application/json")
        .set(
            "User-Agent",
            &format!("CargoBackup/{}", env!("CARGO_PKG_VERSION")),
        )
        .set("Authorization", &format!("token {}", auth.access_token))
        .call()?
        .into_json()?;

        let mut packages: Vec<Package> = Vec::new();
        for file in gist.files.values() {
            if file.filename == "backup.json" {
                packages = serde_json::from_str(file.content.as_ref().unwrap())?;
            }
        }
        Ok(packages)
    }

    async fn push_backup(self) -> Result<(), Box<dyn Error>> {
        let mut auth: OAuth =
            serde_json::from_str(&self.keyring.get_password().expect("Failed to read keyring")).expect("Failed to parse json string");

        let packages = get_packages();

        if auth.gist_id.is_some() {
            let result = ureq::patch(&format!(
                "https://api.github.com/gists/{}",
                auth.gist_id.unwrap()
            ))
            .set("Accept", "application/json")
            .set("Authorization", &format!("token {}", auth.access_token))
            .set("Content-Type", "application/json")
            .set(
                "User-Agent",
                &format!("CargoBackup/{}", env!("CARGO_PKG_VERSION")),
            )
            .set("Authorization", &format!("token {}", auth.access_token))
            .send_json(&json!({
                "description": "Cargo Package Backup (Created by CargoBackup)",
                "public": false,
                "files": {
                    "backup.json": {
                        "content": serde_json::to_string(&packages).unwrap()
                    }
                }
            }))?;

            if result.status() == 200 {
                println!("Successfully updated backup");
            } else {
                println!("Failed to update backup");
            }
        } else {
            // Create a new gist
            let new_gist = ureq::post("https://api.github.com/gists")
                .set("Accept", "application/json")
                .set("Content-Type", "application/json")
                .set(
                    "User-Agent",
                    &format!("CargoBackup/{}", env!("CARGO_PKG_VERSION")),
                )
                .set(
                    "Authorization",
                    &format!("{} {}", &auth.token_type, &auth.access_token),
                )
                .send_json(&json!({
                    "description": "Cargo Package Backup (Created by CargoBackup)",
                    "public": false,
                    "files": {
                        "backup.json": {
                            "filename": "backup.json",
                            "content": serde_json::to_string(&packages).unwrap()
                        }
                    }
                }))?;

            let new_gist: Gist = new_gist.into_json()?;
            let gist_id = new_gist.id;
            auth.gist_id = Some(gist_id.clone());
            self.keyring.set_password(&serde_json::to_string(&auth).expect("Failed to create string")).expect("Failed to write keyring");
            println!("Created gist {} ", gist_id);
        }
        Ok(())
    }
}

