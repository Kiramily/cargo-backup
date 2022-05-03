use std::fs;

use cargo_backup::{get_packages, Package};
use clap::{command, Arg, Command};

fn main() {
    let args = Command::new("cargo")
        .bin_name("cargo")
        .about("Creates a backup of your installed cargo packages")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand(
            command!("backup").arg(
                Arg::new("out")
                    .long("out")
                    .short('o')
                    .takes_value(true)
                    .help("The output file to write to")
                    .default_value("backup.json"),
            ),
        )
        .get_matches();

    match args.subcommand() {
        Some(("backup", args)) => {
            let packages: Vec<Package> = get_packages();

            let out = shellexpand::full(args.value_of("out").unwrap_or("./backup.json"))
                .expect("Failed to expand path");

            let backup = serde_json::to_string_pretty(&packages).expect("Failed to serialize");

            fs::write(out.to_string(), backup).expect("Failed to write backup");
        }
        _ => unreachable!(),
    }
}
