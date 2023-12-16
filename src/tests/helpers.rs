//! Some test helpers for setting up isolated dummy steam installations.

// TODO: add a test with an env var flag that runs against your real local steam installation?

use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fs, iter,
    path::{Path, PathBuf},
};

use crate::{
    tests::{temp::TempDir, TestError},
    SteamDir,
};

use serde::Serialize;

pub fn expect_test_env() -> TempSteamDir {
    TempSteamDir::builder()
        .app(SampleApp::GarrysMod.into())
		.app(SampleApp::Warframe.into())
        .library(SampleApp::GraveyardKeeper.try_into().unwrap())
        .finish()
        .unwrap()
}

// TODO(cosmic): Add in functionality for providing shortcuts too
pub struct TempSteamDir {
    steam_dir: crate::SteamDir,
    _tmps: Vec<TempDir>,
}

impl TryFrom<AppFile> for TempSteamDir {
    type Error = TestError;

    fn try_from(app: AppFile) -> Result<Self, Self::Error> {
        Self::builder().app(app).finish()
    }
}

impl TryFrom<SampleApp> for TempSteamDir {
    type Error = TestError;

    fn try_from(sample_app: SampleApp) -> Result<Self, Self::Error> {
        Self::try_from(AppFile::from(sample_app))
    }
}

impl TempSteamDir {
    pub fn builder() -> TempSteamDirBuilder {
        TempSteamDirBuilder::default()
    }

    pub fn steam_dir(&self) -> &SteamDir {
        &self.steam_dir
    }
}

#[derive(Default)]
#[must_use]
pub struct TempSteamDirBuilder {
    libraries: Vec<TempLibrary>,
    apps: Vec<AppFile>,
}

impl TempSteamDirBuilder {
    pub fn app(mut self, app: AppFile) -> Self {
        self.apps.push(app);
        self
    }

    pub fn library(mut self, library: TempLibrary) -> Self {
        self.libraries.push(library);
        self
    }

    // Steam dir is also a library, but is laid out slightly differently than a regular library
    pub fn finish(self) -> Result<TempSteamDir, TestError> {
        let tmp = TempDir::new()?;
        let root_dir = tmp.path().join("test-steam-dir");
        let steam_dir = root_dir.join("Steam");
        let apps_dir = steam_dir.join("steamapps");
        fs::create_dir_all(&apps_dir)?;

        setup_steamapps_dir(&apps_dir, &self.apps)?;

        let steam_dir_content_id = i32::MIN;
        let apps = self.apps.iter().map(|app| (app.id, 0)).collect();
        let root_library =
            LibraryFolder::mostly_default(steam_dir.clone(), steam_dir_content_id, apps);
        setup_libraryfolders_file(&apps_dir, root_library, &self.libraries)?;

        let tmps = iter::once(tmp)
            .chain(self.libraries.into_iter().map(|library| library._tmp))
            .collect();

        Ok(TempSteamDir {
            steam_dir: SteamDir::from_steam_dir(&steam_dir)?,
            _tmps: tmps,
        })
    }
}

fn setup_steamapps_dir(apps_dir: &Path, apps: &[AppFile]) -> Result<(), TestError> {
    let apps_common_dir = apps_dir.join("common");
    fs::create_dir_all(&apps_common_dir)?;

    for app in apps {
        let manifest_path = apps_dir.join(app.file_name());
        fs::write(&manifest_path, &app.contents)?;
        let app_install_dir = apps_common_dir.join(&app.install_dir);
        fs::create_dir_all(&app_install_dir)?;
    }

    Ok(())
}

fn setup_libraryfolders_file(
    apps_dir: &Path,
    root_library: LibraryFolder,
    aux_libraries: &[TempLibrary],
) -> Result<(), TestError> {
    let library_folders =
        iter::once(root_library).chain(aux_libraries.iter().map(|temp_library| {
            LibraryFolder::mostly_default(
                temp_library.path.clone(),
                temp_library.content_id,
                temp_library.apps.clone(),
            )
        }));
    let inner: BTreeMap<u32, LibraryFolder> = library_folders
        .into_iter()
        .enumerate()
        .map(|(i, f)| (i.try_into().unwrap(), f))
        .collect();
    let library_folders_contents =
        keyvalues_serde::to_string_with_key(&inner, "libraryfolders").unwrap();
    let library_folders_path = apps_dir.join("libraryfolders.vdf");
    fs::write(library_folders_path, library_folders_contents)?;

    Ok(())
}

#[derive(Serialize)]
struct LibraryFolder {
    path: PathBuf,
    label: String,
    contentid: i32,
    totalsize: u64,
    update_clean_bytes_tally: u64,
    time_last_update_corruption: u64,
    apps: BTreeMap<u32, u64>,
}

impl LibraryFolder {
    fn mostly_default(path: PathBuf, contentid: i32, apps: BTreeMap<u32, u64>) -> Self {
        let totalsize = apps.values().sum();
        Self {
            path,
            contentid,
            apps,
            totalsize,
            label: String::default(),
            update_clean_bytes_tally: 79_799_828_443,
            time_last_update_corruption: 0,
        }
    }
}

pub struct TempLibrary {
    content_id: i32,
    path: PathBuf,
    apps: BTreeMap<u32, u64>,
    _tmp: TempDir,
}

impl TryFrom<AppFile> for TempLibrary {
    type Error = TestError;

    fn try_from(app: AppFile) -> Result<Self, Self::Error> {
        Self::builder().app(app).finish()
    }
}

impl TryFrom<SampleApp> for TempLibrary {
    type Error = TestError;

    fn try_from(sample_app: SampleApp) -> Result<Self, Self::Error> {
        Self::try_from(AppFile::from(sample_app))
    }
}

impl TempLibrary {
    pub fn builder() -> TempLibraryBuilder {
        TempLibraryBuilder::default()
    }
}

#[derive(Default)]
#[must_use]
pub struct TempLibraryBuilder {
    apps: Vec<AppFile>,
}

impl TempLibraryBuilder {
    fn app(mut self, app: AppFile) -> Self {
        self.apps.push(app);
        self
    }

    fn finish(self) -> Result<TempLibrary, TestError> {
        let tmp = TempDir::new()?;
        let root_dir = tmp.path().join("test-library");
        let apps_dir = root_dir.join("steamapps");
        fs::create_dir_all(&apps_dir)?;

        let meta_path = apps_dir.join("libraryfolder.vdf");
        fs::write(
            meta_path,
            include_str!("../../tests/assets/libraryfolder.vdf"),
        )?;

        setup_steamapps_dir(&apps_dir, &self.apps)?;
        let apps = self.apps.iter().map(|app| (app.id, 0)).collect();

        Ok(TempLibrary {
            content_id: 1234,
            path: root_dir,
            apps,
            _tmp: tmp,
        })
    }
}

pub struct AppFile {
    id: u32,
    install_dir: String,
    contents: String,
}

impl From<SampleApp> for AppFile {
    fn from(sample: SampleApp) -> Self {
        Self {
            id: sample.id(),
            install_dir: sample.install_dir().to_owned(),
            contents: sample.contents().to_owned(),
        }
    }
}

impl AppFile {
    fn file_name(&self) -> String {
        format!("appmanifest_{}.acf", self.id)
    }
}

pub enum SampleApp {
    GarrysMod,
    GraveyardKeeper,
	Warframe,
}

impl SampleApp {
    pub const fn id(&self) -> u32 {
        self.data().0
    }

    pub const fn install_dir(&self) -> &'static str {
        self.data().1
    }

    pub const fn contents(&self) -> &'static str {
        self.data().2
    }

    pub const fn data(&self) -> (u32, &'static str, &'static str) {
        match self {
            Self::GarrysMod => (
                4_000,
                "GarrysMod",
                include_str!("../../tests/assets/appmanifest_4000.acf"),
            ),
            Self::GraveyardKeeper => (
                599_140,
                "Graveyard Keeper",
                include_str!("../../tests/assets/appmanifest_599140.acf"),
            ),
			Self::Warframe => (
				230_410,
				"Warframe",
				include_str!("../../tests/assets/appmanifest_230410.acf"),
			),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::TestResult;

    #[test]
    fn sanity() -> TestResult {
        let tmp_steam_dir = TempSteamDir::try_from(SampleApp::GarrysMod)?;
        let steam_dir = tmp_steam_dir.steam_dir();
        assert!(steam_dir.find_app(SampleApp::GarrysMod.id()).unwrap().is_some());

        Ok(())
    }
}
