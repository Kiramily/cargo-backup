#![allow(dead_code)]

use crate::Package;
use owo_colors::OwoColorize;
use std::{
    fmt,
    process::{Command, Stdio},
};

pub(crate) enum CommandType {
    Remove,
    Install,
}

pub(crate) fn execute_cmd(package: &Package, cmd_type: CommandType) {
    let mut args: Vec<String> = Vec::new();

    match cmd_type {
        CommandType::Remove => {
            args.push("uninstall".to_string());
            args.push(package.name.clone());
        }

        CommandType::Install => {
            args.push("install".to_string());
            args.push(package.name.clone());
            args.push("--version".to_string());
            let version = package.version.to_string();
            args.push(version);

            if package.all_features {
                args.push("--all-features".to_string());
            }

            if package.no_default_features {
                args.push("--no-default-features".to_string());
            }

            if !package.features.is_empty() {
                args.push("--features".to_string());
                let features = package.features.join(",");
                args.push(features);
            }
        }
    }

    let mut child = Command::new("cargo")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child.wait().unwrap();
}

pub(crate) fn pretty_print_packages(
    to_install: &[Package],
    to_update: &[Package],
    to_remove: &[Package],
) {
    if !to_install.is_empty() {
        println!("{}", "┌ Installing:".green().bold());
        let mut to_install_iter = to_install.iter().peekable();

        while let Some(package) = to_install_iter.next() {
            if to_install_iter.peek().is_some() {
                print!("{}", "├ ".green())
            } else {
                print!("{}", "└ ".green());
            }

            print!(
                "{} [{}]",
                package.name.cyan().bold(),
                package.version.to_string().green()
            );
            println!()
        }
        println!();
    }

    if !to_update.is_empty() {
        println!("{}", "┌ Updating:".yellow().bold());
        let mut to_update_iter = to_update.iter().peekable();

        while let Some(package) = to_update_iter.next() {
            if to_update_iter.peek().is_some() {
                print!("{}", "├ ".yellow())
            } else {
                print!("{}", "└ ".yellow());
            }
            print!(
                "{} {} -> {}",
                package.name.cyan().bold(),
                to_update
                    .iter()
                    .find(|np| np.name == package.name)
                    .unwrap()
                    .version
                    .to_string()
                    .red()
                    .strikethrough(),
                package.version.to_string().green()
            );
            println!()
        }
        println!();
    }

    if !to_remove.is_empty() {
        println!("{}", "┌ Removing:".red().bold());

        let mut to_remove_iter = to_remove.iter().peekable();

        while let Some(package) = to_remove_iter.next() {
            if to_remove_iter.peek().is_some() {
                print!("{}", "├ ".red())
            } else {
                print!("{}", "└ ".red());
            }

            print!(
                "{} [{}]",
                package.name.cyan().bold(),
                package.version.to_string().red()
            );

            println!()
        }
    }
}

pub(crate) enum Errors {
    JsonParse,
    ReadFile,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonParse => write!(f, "failed to parse json string"),
            Self::ReadFile => write!(f, "failed to read file"),
        }
    }
}
