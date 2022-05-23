// Prerequisites:
// * Steam must be installed
// * At least two library folders must be setup
// * At least two Steam apps must be installed
// * An installed Steam game's app ID must be specified below
static APP_ID: u32 = 4000;

use super::*;

#[test]
fn find_steam() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());
    println!("{:#?}", steamdir_found.unwrap());
}

#[test]
fn find_library_folders() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());

    let mut steamdir = steamdir_found.unwrap();

    steamdir.libraryfolders.discover(&steamdir.path);
    assert!(steamdir.libraryfolders().paths.len() > 1);

    println!("{:#?}", steamdir.libraryfolders.paths);
}

#[test]
fn find_app() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());

    let mut steamdir = steamdir_found.unwrap();

    let steamapp = steamdir.app(&APP_ID);
    assert!(steamapp.is_some());

    println!("{:#?}", steamapp.unwrap());
}

#[test]
fn app_details() {
    let mut steamdir = SteamDir::locate().unwrap();
    let steamapp = steamdir.app(&APP_ID).unwrap();
    assert_eq!(steamapp.name, "Garry's Mod");
}

#[test]
fn all_apps() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());

    let mut steamdir = steamdir_found.unwrap();

    let steamapps = steamdir.apps();
    assert!(!steamapps.is_empty());
    assert!(steamapps.keys().len() > 1);
}

#[test]
fn all_apps_get_one() {
    let mut steamdir = SteamDir::locate().unwrap();
    let steamapps = steamdir.apps();
    assert!(steamapps.keys().len() > 1);

    let steamapp = steamdir.app(&APP_ID).unwrap();

    assert_eq!(steamapp.name, "Garry's Mod");
}
