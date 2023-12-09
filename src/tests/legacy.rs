use std::convert::TryInto;

use super::helpers::{SampleApp, TempSteamDir, TestError, TestResult};
use crate::Error;

static GMOD_ID: u32 = SampleApp::GarrysMod.id();

// The legacy test env assumed the following prerequisites:
// - Steam must be installed
// - At least two library folders must be setup (the steam dir acts as one)
// - Garry's Mod along with at least one other steam app must be installed
pub fn legacy_test_env() -> std::result::Result<TempSteamDir, TestError> {
    TempSteamDir::builder()
        .app(SampleApp::GarrysMod.into())
        .library(SampleApp::GraveyardKeeper.try_into()?)
        .finish()
}

#[test]
fn find_library_folders() -> TestResult {
    let tmp_steam_dir = legacy_test_env()?;
    let steam_dir = tmp_steam_dir.steam_dir();
    assert!(steam_dir.libraries().unwrap().len() > 1);
    Ok(())
}

#[test]
fn find_app() -> TestResult {
    let tmp_steam_dir = legacy_test_env()?;
    let steam_dir = tmp_steam_dir.steam_dir();
    let steam_app = steam_dir.app(GMOD_ID).unwrap();
    assert_eq!(steam_app.unwrap().app_id, GMOD_ID);
    Ok(())
}

#[test]
fn app_details() -> TestResult {
    let tmp_steam_dir = legacy_test_env()?;
    let steam_dir = tmp_steam_dir.steam_dir();
    let steam_app = steam_dir.app(GMOD_ID)?.unwrap();
    assert_eq!(steam_app.name.unwrap(), "Garry's Mod");
    Ok(())
}

#[test]
fn all_apps() -> TestResult {
    let tmp_steam_dir = legacy_test_env()?;
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
    let tmp_steam_dir = legacy_test_env()?;
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

    let steam_app = steam_dir.app(GMOD_ID).unwrap().unwrap();
    assert_eq!(
        all_apps
            .into_iter()
            .find(|app| app.app_id == GMOD_ID)
            .unwrap(),
        steam_app
    );

    Ok(())
}
