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
    assert!(steam_dir.libraries().unwrap().len() > 1);
}

#[test]
fn find_app() {
    let steam_dir = SteamDir::locate().unwrap();
    let steam_app = steam_dir.app(APP_ID).unwrap();
    assert_eq!(steam_app.unwrap().app_id, APP_ID);
}

#[test]
fn app_details() {
    let steam_dir = SteamDir::locate().unwrap();
    // TODO(cosmic): I don't like the double `.unwrap()` here. Represent missing as an error or no?
    let steam_app = steam_dir.app(APP_ID).unwrap().unwrap();
    assert_eq!(steam_app.name.unwrap(), "Garry's Mod");
}

#[test]
fn all_apps() {
    let steam_dir = SteamDir::locate().unwrap();
    let mut libraries = steam_dir.libraries().unwrap();
    let all_apps: Vec<_> = libraries
        .try_fold(Vec::new(), |mut acc, maybe_library| {
            let library = maybe_library?;
            for maybe_app in library.apps() {
                let app = maybe_app?;
                acc.push(app);
            }
            Ok::<_, Error>(acc)
        })
        .unwrap();
    assert!(all_apps.len() > 1);
}

#[test]
fn all_apps_get_one() {
    let steam_dir = SteamDir::locate().unwrap();

    let mut libraries = steam_dir.libraries().unwrap();
    let all_apps: Vec<_> = libraries
        .try_fold(Vec::new(), |mut acc, maybe_library| {
            let library = maybe_library?;
            for maybe_app in library.apps() {
                let app = maybe_app?;
                acc.push(app);
            }
            Ok::<_, Error>(acc)
        })
        .unwrap();
    assert!(!all_apps.is_empty());
    assert!(all_apps.len() > 1);

    let steam_app = steam_dir.app(APP_ID).unwrap().unwrap();
    assert_eq!(
        all_apps
            .into_iter()
            .find(|app| app.app_id == APP_ID)
            .unwrap(),
        steam_app
    );
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
