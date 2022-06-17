# cargo backup
<p align="center">
	<img src="https://img.shields.io/crates/v/cargo-backup" />
	<img src="https://img.shields.io/crates/d/cargo-backup" />
	<img src="https://img.shields.io/crates/l/cargo-backup" />
</p>

Backup your installed cargo packages

# installation

## cargo

`$ cargo install cargo-backup`

# usage

`$ cargo backup -o <output file>`

default output file is `<current directory>/backup.json`

`$ cargo restore -i <input file>`

input file is required

## Sync
You need to login first with

`$ cargo sync login`

you can use `push` and `pull` after that
