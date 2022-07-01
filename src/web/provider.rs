use crate::Package;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait BackupProvider {
    async fn new() -> Self;
    async fn login(self) -> Result<(), Box<dyn Error>>;
    async fn fetch_backup(self) -> Result<Vec<Package>, Box<dyn Error>>;
    async fn push_backup(self) -> Result<(), Box<dyn Error>>;
}
