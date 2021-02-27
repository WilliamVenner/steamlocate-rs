[![crates.io](https://meritbadge.herokuapp.com/steamlocate)](https://crates.io/crates/steamlocate)
[![docs.rs](https://docs.rs/steamlocate/badge.svg)](https://docs.rs/steamlocate/)
[![license](https://img.shields.io/crates/l/steamlocate)](https://github.com/WilliamVenner/steamlocate/blob/master/LICENSE)
[![Workflow Status](https://github.com/WilliamVenner/steamlocate-rs/workflows/build/badge.svg)](https://github.com/WilliamVenner/steamlocate-rs/actions?query=workflow%3A%22build%22)

# steamlocate

A crate which efficiently locates any Steam application on the filesystem, and/or the Steam installation itself.

This crate is best used when you do not want to depend on the Steamworks API for your program. In some cases the Steamworks API may be more appropriate to use, in which case I recommend the fantastic [steamworks](https://github.com/Thinkofname/steamworks-rs) crate. You don't need to be a Steamworks partner to get installation directory locations from the Steamworks API.

**This crate supports Windows, macOS and Linux.**

## Using steamlocate
Simply add to your [Cargo.toml](https://doc.rust-lang.org/cargo/reference/manifest.html) file:
```toml
[dependencies]
steamlocate = "0.*"
```

To use [steamid-ng](#steamid-ng-support) with steamlocate, add this to your [Cargo.toml](https://doc.rust-lang.org/cargo/reference/manifest.html) file:
```toml
[dependencies]
steamid-ng = "1.*"

[dependencies.steamlocate]
version = "0.*"
features = ["steamid_ng"]
```

## Caching
All functions in this crate cache their results, meaning you can call them as many times as you like and they will always return the same reference.

If you need to get uncached results, simply instantiate a new [SteamDir](https://docs.rs/steamlocate/*/steamlocate/struct.SteamDir.html).

## steamid-ng Support
This crate supports [steamid-ng](https://docs.rs/steamid-ng) and can automatically convert [SteamApp::last_user](struct.SteamApp.html#structfield.last_user) to a [SteamID](https://docs.rs/steamid-ng/*/steamid_ng/struct.SteamID.html) for you.

To enable this support, [use the  `steamid_ng` Cargo.toml feature](#using-steamlocate).

## Examples

#### Locate the installed Steam directory
```rust
extern crate steamlocate;
use steamlocate::SteamDir;

match SteamDir::locate() {
	Some(steamdir) => println!("{:#?}", steamdir),
	None => panic!("Couldn't locate Steam on this computer!")
}
```
```rust
SteamDir (
	path: PathBuf: "C:\\Program Files (x86)\\Steam"
)
```

#### Locate an installed Steam app by its app ID
This will locate Garry's Mod anywhere on the filesystem.
```rust
extern crate steamlocate;
use steamlocate::SteamDir;

let mut steamdir = SteamDir::locate().unwrap();
match steamdir.app(&4000) {
	Some(app) => println!("{:#?}", app),
	None => panic!("Couldn't locate Garry's Mod on this computer!")
}
```
```rust
SteamApp (
	appid: u32: 4000,
	path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
	vdf: <steamy_vdf::Table>,
	name: Some(String: "Garry's Mod"),
	last_user: Some(u64: 76561198040894045)
)
```

#### Locate all Steam apps on this filesystem
```rust
extern crate steamlocate;
use steamlocate::{SteamDir, SteamApp};
use std::collections::HashMap;

let mut steamdir = SteamDir::locate().unwrap();
let apps: &HashMap<u32, Option<SteamApp>> = steamdir.apps();

println!("{:#?}", apps);
```
```rust
{
	4000: SteamApp (
		appid: u32: 4000,
		path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
		vdf: <steamy_vdf::Table>,
		name: Some(String: "Garry's Mod"),
		last_user: Some(u64: 76561198040894045)
	)
	...
}
```

#### Locate all Steam library folders
```rust
extern crate steamlocate;
use steamlocate::{SteamDir, LibraryFolders};
use std::{vec, path::PathBuf};

let mut steamdir: SteamDir = SteamDir::locate().unwrap();
let libraryfolders: &LibraryFolders = steamdir.libraryfolders();
let paths: &Vec<PathBuf> = &libraryfolders.paths;

println!("{:#?}", paths);
```
```rust
{
	"C:\\Program Files (x86)\\Steam\\steamapps",
	"D:\\Steam\\steamapps",
	"E:\\Steam\\steamapps",
	"F:\\Steam\\steamapps",
	...
}
```

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the MIT license,
shall be dual licensed as above, without any additional terms or conditions.
