use cargo_backup::{restore, Package};
use clap::{command, Arg, Command};
use std::fs;

fn main() {
    let args = Command::new("cargo")
        .about("Restores a backup created by cargo-backup")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .bin_name("cargo")
        .subcommand(
            command!("restore").arg(
                Arg::new("input")
                    .long("input")
                    .short('i')
                    .takes_value(true)
                    .required(true)
                    .help("The input file to restore from"),
            ),
        )
        .get_matches();

    match args.subcommand() {
        Some(("restore", args)) => {
            let input =
                shellexpand::full(args.value_of("input").unwrap()).expect("Failed to expand path");

            let backup = fs::read_to_string(input.to_string()).expect("Failed to read backup");

            let packages: Vec<Package> =
                serde_json::from_str(&backup).expect("Failed to deserialize");

            restore(&packages);
        }
        _ => unreachable!(),
    }
}
