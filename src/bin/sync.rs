use cargo_backup::{
    restore,
    web::{
        github,
        provider::Backup,
        types::github::OAuth,
    },
};
use clap::{command, Arg, Command};

#[tokio::main]
async fn main() {
    let args = Command::new("cargo")
        .about("Restores a backup created by cargo-backup")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .bin_name("cargo")
        .subcommand(
            command!("sync")
                .subcommand(command!("login"))
                .subcommand(command!("push"))
                .subcommand(command!("pull"))
                .subcommand(
                    command!("set-id").arg(
                        Arg::new("id")
                            .help("The ID of the backup to restore from")
                            .required(true)
                            .index(1),
                    ),
                ),
        )
        .get_matches();

    let git = github::Api::new().await;

    match args.subcommand() {
        Some(("sync", args)) => {
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

                    restore(&packages);
                }
                Some(("set-id", args)) => {
                    // println!("Setting ID to {}", args.value_of("id").unwrap());
                    let keyring = github::Api::get_keyring();

                    if let Ok(password) = keyring.get_password() {
                        let mut content: OAuth = serde_json::from_str(&password).expect("Failed to parse json string");
                        content.gist_id = Some(args.value_of("id").unwrap().to_string());
                        keyring.set_password(&serde_json::to_string(&content).expect("Failed to create json string")).expect("Failed to save content to keyring");
                    } else {
                        println!("please login first with \"cargo sync login\"")
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
