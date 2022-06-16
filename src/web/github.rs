use crate::web::types::github::{GitHubCode, GithubGist, GithubOauth, OAuthErrorCode};
use crate::{get_packages, Package};
use async_trait::async_trait;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::thread::{self};

#[async_trait]
pub trait BackupProvider {
    async fn new() -> Self;
    async fn login(self) -> Result<(), Box<dyn Error>>;
    async fn fetch_backup(self) -> Result<Vec<Package>, Box<dyn Error>>;
    async fn push_backup(self) -> Result<(), Box<dyn Error>>;
}

pub struct GithubApi {
    config_dir: PathBuf,
}

pub struct Backup {}

#[async_trait]
impl BackupProvider for GithubApi {
    async fn new() -> Self {
        let mut config_dir = dirs::config_dir().unwrap();
        config_dir.push("cargo-backup");
        if !config_dir.exists() {
            fs::create_dir(&config_dir).unwrap();
        }
        GithubApi { config_dir }
    }

    async fn login(self) -> Result<(), Box<dyn Error>> {
        let mut auth_file = self.config_dir.clone();
        auth_file.push("github.auth");
        if auth_file.exists() {
            return Ok(());
        }

        // Request a device code from GitHub
        let device_login: GitHubCode = ureq::post(&format!(
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
        let mut auth: Option<GithubOauth> = None;

        while !has_token {
            thread::sleep(std::time::Duration::from_secs(device_login.interval as u64));
            // Check if the user has entered the code
            let poll_req: GithubOauth = ureq::post(&format!(
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

        write_auth(auth, &self.config_dir.clone());

        Ok(())
    }

    async fn fetch_backup(self) -> Result<Vec<Package>, Box<dyn Error>> {
        let auth = read_auth(&self.config_dir.clone())?;

        // Read the gist
        let gist: GithubGist = ureq::get(&format!(
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
                packages = serde_json::from_str(&file.content.as_ref().unwrap())?;
            }
        }
        Ok(packages)
    }

    async fn push_backup(self) -> Result<(), Box<dyn Error>> {
        let mut config = self.config_dir.clone();
        config.push("github.auth");
        let mut auth: GithubOauth =
            serde_json::from_reader(std::fs::File::open(config).unwrap()).unwrap();

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
            let new_gist = ureq::post(&format!("https://api.github.com/gists",))
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

            let new_gist: GithubGist = new_gist.into_json()?;
            let gist_id = new_gist.id;
            auth.gist_id = Some(gist_id.clone());
            write_auth(auth, &self.config_dir.clone());
            println!("Created gist {} ", gist_id);
        }
        Ok(())
    }
}

fn read_auth(config_dir: &PathBuf) -> Result<GithubOauth, Box<dyn Error>> {
    let mut config = config_dir.clone();
    config.push("github.auth");
    let auth: GithubOauth = serde_json::from_reader(std::fs::File::open(config).unwrap()).unwrap();
    Ok(auth)
}

/// Write the auth to a file
fn write_auth(auth: GithubOauth, dir: &PathBuf) {
    let mut auth_file = dir.clone();
    if !dir.exists() {
        fs::create_dir(dir).unwrap();
    }
    auth_file.push("github.auth");

    let mut auth_file = std::fs::File::create(auth_file).unwrap();

    auth_file
        .write_all(serde_json::to_string(&auth).unwrap().as_bytes())
        .unwrap();
}
