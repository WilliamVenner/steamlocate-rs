[![crates.io](https://img.shields.io/crates/v/steamlocate.svg)](https://crates.io/crates/steamlocate)
[![docs.rs](https://docs.rs/steamlocate/badge.svg)](https://docs.rs/steamlocate/)
[![license](https://img.shields.io/crates/l/steamlocate)](https://github.com/WilliamVenner/steamlocate/blob/master/LICENSE)
[![Workflow Status](https://github.com/WilliamVenner/steamlocate-rs/workflows/ci/badge.svg)](https://github.com/WilliamVenner/steamlocate-rs/actions?query=workflow%3A%22ci%22)

# steamlocate

A crate which efficiently locates any Steam application on the filesystem,
and/or the Steam installation itself.

This crate is best used when you do not want to depend on the Steamworks API
for your program. In some cases the Steamworks API may be more appropriate to
use, in which case I recommend the fantastic
[steamworks](https://github.com/Thinkofname/steamworks-rs) crate. You don't
need to be a Steamworks partner to get installation directory locations from
the Steamworks API.

# Using steamlocate

Simply add `steamlocate` using
[`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```console
$ cargo add steamlocate
```

## Feature flags

Default: `locate`

| Feature flag | Description |
| :---: | :--- |
| `locate` | Enables automatically detecting the Steam installation on supported platforms (currently Windows, MacOS, and Linux). Unsupported platforms will return a runtime error. |

# Examples

## Locate the Steam installation and a specific game

The `SteamDir` is going to be your entrypoint into _most_ parts of the API.
After you locate it you can access related information.

```rust,ignore
let steam_dir = steamlocate::SteamDir::locate()?;
println!("Steam installation - {}", steam_dir.path().display());
// ^^ prints something like `Steam installation - C:\Program Files (x86)\Steam`

const GMOD_APP_ID: u32 = 4_000;
let (garrys_mod, _lib) = steam_dir
    .find_app(GMOD_APP_ID)?
    .expect("Of course we have G Mod");
assert_eq!(garrys_mod.name.as_ref().unwrap(), "Garry's Mod");
println!("{garrys_mod:#?}");
// ^^ prints something like vv
```
```rust,ignore
App {
    app_id: 4_000,
    install_dir: "GarrysMod",
    name: Some("Garry's Mod"),
    universe: Some(Public),
    // much much more data
}
```

## Get an overview of all libraries and apps on the system

You can iterate over all of Steam's libraries from the steam dir. Then from each library you
can iterate over all of its apps.

```rust,ignore
let steam_dir = steamlocate::SteamDir::locate()?;

for library in steam_dir.libraries()? {
    let library = library?;
    println!("Library - {}", library.path().display());

    for app in library.apps() {
        let app = app?;
        println!("    App {} - {:?}", app.app_id, app.name);
    }
}
```

On my laptop this prints

```text
Library - /home/wintermute/.local/share/Steam
    App 1628350 - Steam Linux Runtime 3.0 (sniper)
    App 1493710 - Proton Experimental
    App 4000 - Garry's Mod
Library - /home/wintermute/temp steam lib
    App 391540 - Undertale
    App 1714040 - Super Auto Pets
    App 2348590 - Proton 8.0
```

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the MIT license,
shall be licensed as above, without any additional terms or conditions.
