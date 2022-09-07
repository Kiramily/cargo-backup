use cargo_backup::{install_packages, Package};
use clap::{command, Arg, Command};
use std::fs;

fn main() {
    let args = Command::new("cargo")
        .about("Restores a backup created by cargo-backup")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .bin_name("cargo")
        .subcommand(
            command!("restore")
                .arg(
                    Arg::new("input")
                        .long("backup")
                        .short('b')
                        .takes_value(true)
                        .required(true)
                        .help("The input file to restore from"),
                )
                .arg(
                    Arg::new("skip-install")
                        .short('i')
                        .long("skip-install")
                        .help("Skip package installation"),
                )
                .arg(
                    Arg::new("skip-update")
                        .short('u')
                        .long("skip-update")
                        .help("Skip update for outdated Packages"),
                )
                .arg(
                    Arg::new("skip-remove")
                        .short('r')
                        .long("skip-remove")
                        .help("Skip removal of Packages not found in the backup"),
                ),
        )
        .get_matches();

    match args.subcommand() {
        Some(("restore", args)) => {
            let input =
                shellexpand::full(args.value_of("input").unwrap()).expect("Failed to expand path");

            let backup = fs::read_to_string(input.to_string()).expect("Failed to read backup");

            let packages: Vec<Package> =
                serde_json::from_str(&backup).expect("Failed to parse JSON");

            install_packages(
                &packages,
                args.is_present("skip-install"),
                args.is_present("skip-update"),
                args.is_present("skip-remove"),
            )
        }
        _ => unreachable!(),
    }
}
