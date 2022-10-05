use cargo_backup::{get_packages, Package};
use clap::{builder::ValueParser, command, Arg, Command};
use std::{fs, path::PathBuf};

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
                    .value_parser(ValueParser::path_buf())
                    .help("The output file to write to")
                    .default_value("./backup.json"),
            ),
        )
        .get_matches();

    match args.subcommand() {
        Some(("backup", args)) => {
            let packages: Vec<Package> = get_packages();

            let out = args.get_one::<PathBuf>("out").unwrap();

            let backup = serde_json::to_string(&packages).expect("Failed to serialize");

            fs::write(out, backup).expect("Failed to write backup");
        }
        _ => unreachable!(),
    }
}
