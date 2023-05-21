// Prerequisites:
// * Steam must be installed
// * At least two library folders must be setup
// * At least two Steam apps must be installed
// * An installed Steam game's app ID must be specified below
static APP_ID: u32 = 4000;

use super::*;

#[test]
fn find_steam() {
    SteamDir::locate().unwrap();
}

#[test]
fn find_library_folders() {
    let steam_dir = SteamDir::locate().unwrap();
    assert!(steam_dir.libraries().len() > 1);
}

#[test]
fn find_app() {
    let steam_dir = SteamDir::locate().unwrap();
    let steam_app = steam_dir.app(APP_ID).unwrap();
    assert_eq!(steam_app.app_id, APP_ID);
}

#[test]
fn app_details() {
    let steam_dir = SteamDir::locate().unwrap();
    let steam_app = steam_dir.app(APP_ID).unwrap();
    assert_eq!(steam_app.name.unwrap(), "Garry's Mod");
}

#[test]
fn all_apps() {
    let steam_dir = SteamDir::locate().unwrap();
    let steam_apps: Vec<_> = steam_dir
        .libraries()
        .iter()
        .flat_map(Library::apps)
        .collect::<Option<_>>()
        .unwrap();
    assert!(!steam_apps.is_empty());
    assert!(steam_apps.len() > 1);
}

#[test]
fn all_apps_get_one() {
    let steam_dir = SteamDir::locate().unwrap();

    let steam_apps: Vec<_> = steam_dir
        .libraries()
        .iter()
        .flat_map(Library::apps)
        .collect::<Option<_>>()
        .unwrap();
    assert!(!steam_apps.is_empty());
    assert!(steam_apps.len() > 1);

    let steam_app = steam_dir.app(APP_ID).unwrap();
    assert_eq!(
        steam_apps
            .into_iter()
            .find(|app| app.app_id == APP_ID)
            .unwrap(),
        steam_app
    );
}
