use cargo_backup::{
    restore,
    web::github::{BackupProvider, GithubApi},
};
use clap::{command, Command};

#[tokio::main]
async fn main() {
    let args = Command::new("cargo")
        .about("Restores a backup created by cargo-backup")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .bin_name("cargo")
        .subcommand(command!("login"))
        .subcommand(command!("push"))
        .subcommand(command!("pull"))
        .get_matches();

    let git = GithubApi::new().await;

    match args.subcommand() {
        Some(("login", _)) => {
            git.login().await.expect("Failed to login");
            println!("Successfully logged in");
        }
        Some(("push", _)) => {
            git.push_backup().await.expect("Failed to push backup");
        }
        Some(("pull", _)) => {
            let packages = git.fetch_backup().await.expect("Failed to pull backup");

            restore(packages)
        }
        _ => unreachable!(),
    }
}
