[![crates.io](https://meritbadge.herokuapp.com/steamlocate)](https://crates.io/crates/steamlocate)
[![docs.rs](https://docs.rs/steamlocate/badge.svg)](https://docs.rs/steamlocate/)
[![license](https://img.shields.io/crates/l/steamlocate)](https://github.com/linebender/steamlocate/blob/master/LICENSE)
[![Workflow Status](https://github.com/WilliamVenner/steamlocate-rs/workflows/main/badge.svg)](https://github.com/WilliamVenner/steamlocate-rs/actions?query=workflow%3A%22main%22)

# steamlocate

A crate which efficiently locates any Steam application on the filesystem, and/or the Steam installation itself.

**This crate supports Windows, macOS and Linux.**

## Caching
All functions in this crate cache their results, meaning you can call them as many times as you like and they will always return the same reference.

If you need to get uncached results, simply instantiate a new [SteamDir](struct.SteamDir.html).

## steamid-ng Support
This crate supports [steamid-ng](/steamid-ng) and can automatically convert [SteamApp::last_user](struct.SteamApp.html#structfield.last_user) to a [SteamID](/steamid-ng/*/steamid-ng/struct.SteamID.html) for you.

To enable this feature, build with `cargo build --features steamid_ng`

## Examples

### Locate the installed Steam directory
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

### Locate an installed Steam app by its app ID
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
	last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
)
```

### Locate all Steam apps on this filesystem
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
		last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
	)
	...
}
```

### Locate all Steam library folders
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

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
