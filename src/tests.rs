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

    assert!(steamdir.app(&u32::MAX).is_none());
}

#[test]
fn app_details() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());

    let mut steamdir = steamdir_found.unwrap();

    let steamapp = steamdir.app(&APP_ID);
    assert!(steamapp.is_some());

    assert!(steamapp.unwrap().name.is_some());
    assert!(steamapp.unwrap().last_user.is_some());
}

#[test]
fn all_apps() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());

    let mut steamdir = steamdir_found.unwrap();

    let steamapps = steamdir.apps();
    assert!(!steamapps.is_empty());
    assert!(steamapps.keys().len() > 1);

    // println!("{:#?}", steamapps);
}

#[test]
fn all_apps_get_one() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());

    let mut steamdir = steamdir_found.unwrap();

    let steamapps = steamdir.apps();
    assert!(!steamapps.is_empty());
    assert!(steamapps.keys().len() > 1);

    let steamapp = steamdir.app(&APP_ID);
    assert!(steamapp.is_some());

    assert!(steamapp.unwrap().name.is_some());
    assert!(steamapp.unwrap().last_user.is_some());
}

#[test]
fn find_compatibility_tool() {
    let steamdir_found = SteamDir::locate();
    assert!(steamdir_found.is_some());

    let mut steamdir = steamdir_found.unwrap();

    let tool = steamdir.compat_tool(&APP_ID);
    assert!(tool.is_some());

    println!("{:#?}", tool.unwrap());
}
