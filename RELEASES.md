# Release Checklist

* [ ] Update `CHANGELOG.md` using `./scripts/generate_changelog.py --version 0.NEW.VERSION`
* [ ] Bump version numbers in `Cargo.toml`
* [ ] run `cargo clippy`
* [ ] run `typos`
* [ ] `git commit -m 'Release Eon 0.x.0 - summary'`
* [ ] publish
    ```
    cargo publish --quiet -p eon_syntax
    cargo publish --quiet -p eon
    cargo publish --quiet -p eonfmt
    ```
* [ ] `git tag -a 0.x.0 -m 'Release Eon 0.x.0 - summary'`
* [ ] `git pull --tags ; git tag -d latest && git tag -a latest -m 'Latest release' && git push --tags origin latest --force ; git push --tags`
* [ ] Do a GitHub release: https://github.com/emilk/eon/releases/new
