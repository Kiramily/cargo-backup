# cargo backup

Backup your installed cargo packages

# installation

## cargo

`$ cargo install cargo-backup`

# usage

`$ cargo backup -o <output file>`

default output file is _cwd_/backup.json

`$ cargo restore -i <input file>`

input file is required

# TODO

- [ ] Sync

- - [ ] gist integration
- - [ ] pull packages
- - [ ] push packages
- - [ ] merge packages
