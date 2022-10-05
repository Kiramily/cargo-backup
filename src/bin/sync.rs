use cargo_backup::get_packages;
use cargo_backup::remote::RemoteProvider;
use cargo_backup::{install_packages, remote::github::Github};
use clap::builder::ValueParser;
use clap::{command, Arg, ArgAction, Command};

fn main() {
    let args = Command::new("cargo")
        .about("Restores a backup created by cargo-backup")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .bin_name("cargo")
        .subcommand(
            command!("sync")
                .arg(
                    Arg::new("provider")
                        .short('p')
                        .long("provider")
                        .help("The Remote provider to use (not implemented yet)")
                        .default_value("github")
                        // TODO: Create a value parser
                        .value_parser(ValueParser::string()),
                )
                .subcommand(
                    command!("login").arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .help("Ignores current credentials")
                            .action(ArgAction::SetTrue),
                    ),
                )
                .subcommand(command!("push"))
                .subcommand(
                    command!("pull")
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

    match args.subcommand() {
        Some(("sync", args)) => {
            let provider = Github::new();

            match args.subcommand() {
                Some(("pull", args)) => {
                    let packages = provider.pull().unwrap();
                    install_packages(
                        &packages,
                        args.get_flag("skip-install"),
                        args.get_flag("skip-update"),
                        args.get_flag("skip-remove"),
                    )
                }
                Some(("push", _)) => {
                    let packages = get_packages();
                    provider.push(&packages).unwrap();
                }
                Some(("login", args)) => {
                    let force = args.get_flag("force");
                    provider.login(force).unwrap();
                }
                Some(("set-id", args)) => provider
                    .set_id(args.get_one::<String>("id").unwrap().to_string())
                    .unwrap(),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
