The process for cutting a new release

- [ ] Check for unused dependencies
  - `$ cargo +nightly udeps`
- [ ] Bump the `version` in `Cargo.toml`
- [ ] Propagate the change to `Cargo.lock`
  - `$ cargo check`
- [ ] Update `rust-version` in `Cargo.toml`
  - Comment out the existing `rust-version`
  - `$ cargo msrv find [--ignore-lockfile]`
- [ ] Update the `CHANGELOG.md` to reflect any of the changes
- [ ] Merge changes through a PR or directly to make sure CI passes
- [ ] Publish on crates.io
  - `$ cargo publish`
- [ ] Publish on GitHub by pushing a version tag
  - `$ git tag v{VERSION}` (make sure the branch you are on is up to date)
  - `$ git push upstream/origin v{VERSION}`
- [ ] Make a release announcement on GitHub after the release workflow finishes
