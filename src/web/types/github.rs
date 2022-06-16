use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubOauth {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub token_type: String,
    #[serde(default, skip_serializing)]
    pub error: OAuthErrorCode,
    pub gist_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthErrorCode {
    AuthorizationPending,
    SlowDown,
    ExpiredToken,
    UnsupportedGrantType,
    IncorrectClientCredentials,
    IncorrectDeviceCode,
    AccessDenied,
    None,
}

impl Default for OAuthErrorCode {
    fn default() -> Self {
        OAuthErrorCode::None
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubGist {
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
