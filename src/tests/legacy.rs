// TODO: steamlocate_tempfile cfg for docs. Otherwise rely on a env var to get passed in

use crate::{
    tests::{
        helpers::{expect_test_env, SampleApp},
        TestResult,
    },
    Error,
};

static GMOD_ID: u32 = SampleApp::GarrysMod.id();

#[test]
fn find_library_folders() -> TestResult {
    let tmp_steam_dir = expect_test_env();
    let steam_dir = tmp_steam_dir.steam_dir();
    assert!(steam_dir.libraries().unwrap().len() > 1);
    Ok(())
}

#[test]
fn find_app() -> TestResult {
    let tmp_steam_dir = expect_test_env();
    let steam_dir = tmp_steam_dir.steam_dir();
    let steam_app = steam_dir.find_app(GMOD_ID).unwrap();
    assert_eq!(steam_app.unwrap().0.app_id, GMOD_ID);
    Ok(())
}

#[test]
fn app_details() -> TestResult {
    let tmp_steam_dir = expect_test_env();
    let steam_dir = tmp_steam_dir.steam_dir();
    let steam_app = steam_dir.find_app(GMOD_ID)?.unwrap();
    assert_eq!(steam_app.0.name.unwrap(), "Garry's Mod");
    Ok(())
}

#[test]
fn all_apps() -> TestResult {
    let tmp_steam_dir = expect_test_env();
    let steam_dir = tmp_steam_dir.steam_dir();
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
    Ok(())
}

#[test]
fn all_apps_get_one() -> TestResult {
    let tmp_steam_dir = expect_test_env();
    let steam_dir = tmp_steam_dir.steam_dir();

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

    let steam_app = steam_dir.find_app(GMOD_ID).unwrap().unwrap();
    assert_eq!(
        all_apps
            .into_iter()
            .find(|app| app.app_id == GMOD_ID)
            .unwrap(),
        steam_app.0,
    );

    Ok(())
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
