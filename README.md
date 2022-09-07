<h1 align="center">
Cargo Backup
</h1>
<p align="center">
	<img src="https://img.shields.io/crates/v/cargo-backup" />
	<img src="https://img.shields.io/crates/d/cargo-backup" />
	<img src="https://img.shields.io/crates/l/cargo-backup" />
</p>

<p align="center">
	Backup your installed cargo packages
</p>


# installation
```sh
cargo install cargo-backup
```

# Usage
## Backup
```sh
cargo backup <args>
```
### Arguments
* `--out | -o` - The output file where the backup will be written to. default `backup.json`

## Restore
```sh
cargo restore --backup path/to/backup <args>
```
### Arguments
* `--backup | -b` - The backup file. *required*
* `--skip-install | -i` - Skips the installation of new packages.
* `--skip-update | -u` - Skips the packages to update. 
* `--skip-remove | -r` - Skips the removal of packages not found in the backup. 

## Sync
Requires a Github account.
```sh
cargo sync <sub-command> <args>
```

### Login
```sh
cargo sync login <args>
```

#### Arguments
* `--force | -f` - Ignores the current Credentials.

### Push
Either push a new backup or Updates the old one.
```sh
cargo sync push <args>
```

### Pull
Pulls the backup from the gist repository.
**A valid gist id needs to be set for this.**
```sh
cargo sync pull <args>
```

#### Arguments
* `--skip-install | -i` - Skips the installation of new packages.
* `--skip-update | -u` - Skips the packages to update. 
* `--skip-remove | -r` - Skips the removal of packages not found in the backup. 

### set-id
```sh
cargo sync set-id <gist-id>
```

# License
[MIT](https://choosealicense.com/licenses/mit/)
