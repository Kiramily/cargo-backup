use cargo_backup::{install_packages, Package};
use clap::{builder::ValueParser, command, Arg, ArgAction, Command};
use std::{fs, path::PathBuf};

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
                        .value_parser(ValueParser::path_buf())
                        .required(true)
                        .help("The input file to restore from"),
                )
                .arg(
                    Arg::new("skip-install")
                        .short('i')
                        .long("skip-install")
                        .help("Skip package installation")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("skip-update")
                        .short('u')
                        .long("skip-update")
                        .help("Skip update for outdated Packages")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("skip-remove")
                        .short('r')
                        .long("skip-remove")
                        .help("Skip removal of Packages not found in the backup")
                        .action(ArgAction::SetTrue),
                ),
        )
        .get_matches();

    match args.subcommand() {
        Some(("restore", args)) => {
            let input = args.get_one::<PathBuf>("input").unwrap();
            // let input =
            //     shellexpand::full(args.value_of("input").unwrap()).expect("Failed to expand path");

            let backup = fs::read_to_string(input).expect("Failed to read backup");

            let packages: Vec<Package> =
                serde_json::from_str(&backup).expect("Failed to parse JSON");

            install_packages(
                &packages,
                args.get_flag("skip-install"),
                args.get_flag("skip-update"),
                args.get_flag("skip-remove"),
            )
        }
        _ => unreachable!(),
    }
}
