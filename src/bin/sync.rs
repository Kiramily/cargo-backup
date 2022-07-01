use cargo_backup::{
    restore,
    web::{
        github::{self, BackupProvider},
        types::github::OAuth,
    },
};
use clap::{command, Arg, Command};
use std::fs;

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
        .subcommand(
            command!("set-id").arg(
                Arg::new("id")
                    .help("The ID of the backup to restore")
                    .required(true)
                    .index(1),
            ),
        )
        .get_matches();

    let git = github::Api::new().await;

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

            restore(&packages)
        }
        Some(("set-id", args)) => {
            // println!("Setting ID to {}", args.value_of("id").unwrap());
            let mut auth_file = dirs::config_dir().unwrap();
            auth_file.push("cargo-backup/github.auth");

            let mut auth_content: OAuth = OAuth::default();

            if auth_file.exists() {
                auth_content = serde_json::from_reader(
                    fs::File::open(&auth_file).expect("Failed to open auth file"),
                )
                .expect("Failed to parse auth file");
            }

            auth_content.gist_id = Some(args.value_of("id").unwrap().to_string());

            let mut auth_file = fs::File::create(&auth_file).expect("Failed to create auth file");
            serde_json::to_writer_pretty(&mut auth_file, &auth_content)
                .expect("Failed to write auth file");
        }
        _ => unreachable!(),
    }
}
