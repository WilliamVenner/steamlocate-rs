use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    slice, time,
};

use crate::{
    error::{ParseError, ParseErrorKind},
    Error, Library, Result,
};

use serde::Deserialize;

pub struct Iter<'library> {
    library: &'library Library,
    app_ids: slice::Iter<'library, u32>,
}

impl<'library> Iter<'library> {
    pub(crate) fn new(library: &'library Library) -> Self {
        Self {
            library,
            app_ids: library.app_ids().iter(),
        }
    }
}

impl<'library> Iterator for Iter<'library> {
    type Item = Result<App>;

    fn next(&mut self) -> Option<Self::Item> {
        let app_id = *self.app_ids.next()?;
        if let some_res @ Some(_) = self.library.app(app_id) {
            some_res
        } else {
            // We use the listing from libraryfolders, so all apps should be accounted for
            Some(Err(Error::MissingExpectedApp { app_id }))
        }
    }
}

/// An instance of an installed Steam app.
/// # Example
/// ```ignore
/// # use steamlocate::InstallDir;
/// let mut steamdir = InstallDir::locate().unwrap();
/// let gmod = steamdir.app(&4000);
/// println!("{:#?}", gmod.unwrap());
/// ```
/// ```ignore
/// App (
///     appid: u32: 4000,
///     path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
///     vdf: <steamy_vdf::Table>,
///     name: Some(String: "Garry's Mod"),
///     last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
/// )
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
#[non_exhaustive]
pub struct App {
    /// The app ID of this Steam app.
    pub app_id: u32,

    /// The path to the installation directory of this Steam app.
    ///
    /// Example: `C:\Program Files (x86)\Steam\steamapps\common\GarrysMod`
    pub path: PathBuf,

    /// The store name of the Steam app.
    pub name: Option<String>,

    pub universe: Option<Universe>,
    pub launcher_path: Option<PathBuf>,
    pub state_flags: Option<StateFlags>,
    pub last_updated: Option<time::SystemTime>,
    // Can't find anything on what these values mean. I've seen 0, 2, 4, 6, and 7
    pub update_result: Option<u64>,
    pub size_on_disk: Option<u64>,
    pub build_id: Option<u64>,
    pub bytes_to_download: Option<u64>,
    pub bytes_downloaded: Option<u64>,
    pub bytes_to_stage: Option<u64>,
    pub bytes_staged: Option<u64>,
    pub staging_size: Option<u64>,
    pub target_build_id: Option<u64>,
    pub auto_update_behavior: AutoUpdateBehavior,
    pub allow_other_downloads_while_running: AllowOtherDownloadsWhileRunning,
    pub scheduled_auto_update: Option<time::SystemTime>,
    pub full_validate_before_next_update: bool,
    pub full_validate_after_next_update: bool,
    pub installed_depots: BTreeMap<u64, Depot>,
    pub staged_depots: BTreeMap<u64, Depot>,
    pub user_config: BTreeMap<String, String>,
    pub mounted_config: BTreeMap<String, String>,
    pub install_scripts: BTreeMap<u64, PathBuf>,
    pub shared_depots: BTreeMap<u64, u64>,

    /// The SteamID64 of the last Steam user that played this game on the filesystem.
    ///
    /// This crate supports [steamid-ng](https://docs.rs/steamid-ng) and can automatically convert this to a [SteamID](https://docs.rs/steamid-ng/*/steamid_ng/struct.SteamID.html) for you.
    ///
    /// To enable this support, [use the  `steamid_ng` Cargo.toml feature](https://docs.rs/steamlocate/*/steamlocate#using-steamlocate).
    pub last_user: Option<u64>,
}

impl App {
    pub(crate) fn new(library_path: &Path, manifest: &Path) -> Result<Self> {
        let contents = fs::read_to_string(manifest).map_err(|io| Error::io(io, manifest))?;
        let internal = keyvalues_serde::from_str(&contents).map_err(|err| {
            Error::parse(
                ParseErrorKind::App,
                ParseError::from_serde(err),
                manifest,
            )
        })?;
        let app = Self::from_internal_steam_app(internal, library_path);

        // Check if the installation path exists and is a valid directory
        // TODO: this one check really shapes a lot of the API (in terms of how the data for the
        // `App` is resolved. Maybe move this to something like
        // ```rust
        // library.resolve_install_dir(&app)?;
        // ```
        if app.path.is_dir() {
            Ok(app)
        } else {
            Err(Error::MissingAppInstall {
                app_id: app.app_id,
                path: app.path,
            })
        }
    }

    pub(crate) fn from_internal_steam_app(internal: InternalApp, library_path: &Path) -> Self {
        let InternalApp {
            app_id,
            universe,
            launcher_path,
            name,
            state_flags,
            install_dir,
            last_updated,
            update_result,
            size_on_disk,
            build_id,
            last_user,
            bytes_to_download,
            bytes_downloaded,
            bytes_to_stage,
            bytes_staged,
            staging_size,
            target_build_id,
            auto_update_behavior,
            allow_other_downloads_while_running,
            scheduled_auto_update,
            full_validate_before_next_update,
            full_validate_after_next_update,
            installed_depots,
            staged_depots,
            user_config,
            mounted_config,
            install_scripts,
            shared_depots,
        } = internal;

        let path = library_path
            .join("steamapps")
            .join("common")
            .join(install_dir);

        let universe = universe.map(Universe::from);
        let state_flags = state_flags.map(StateFlags);
        let last_updated = last_updated.and_then(time_as_secs_from_unix_epoch);
        let scheduled_auto_update = if scheduled_auto_update == Some(0) {
            None
        } else {
            scheduled_auto_update.and_then(time_as_secs_from_unix_epoch)
        };
        let allow_other_downloads_while_running = allow_other_downloads_while_running
            .map(AllowOtherDownloadsWhileRunning::from)
            .unwrap_or_default();
        let auto_update_behavior = auto_update_behavior
            .map(AutoUpdateBehavior::from)
            .unwrap_or_default();
        let full_validate_before_next_update = full_validate_before_next_update.unwrap_or_default();
        let full_validate_after_next_update = full_validate_after_next_update.unwrap_or_default();

        Self {
            app_id,
            universe,
            launcher_path,
            name,
            state_flags,
            path,
            last_updated,
            update_result,
            size_on_disk,
            build_id,
            last_user,
            bytes_to_download,
            bytes_downloaded,
            bytes_to_stage,
            bytes_staged,
            staging_size,
            target_build_id,
            auto_update_behavior,
            allow_other_downloads_while_running,
            scheduled_auto_update,
            full_validate_before_next_update,
            full_validate_after_next_update,
            installed_depots,
            staged_depots,
            user_config,
            mounted_config,
            install_scripts,
            shared_depots,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum Universe {
    Invalid,
    Public,
    Beta,
    Internal,
    Dev,
    Unknown(u64),
}

// More info:
// https://developer.valvesoftware.com/wiki/SteamID#Universes_Available_for_Steam_Accounts
impl From<u64> for Universe {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Invalid,
            1 => Self::Public,
            2 => Self::Beta,
            3 => Self::Internal,
            4 => Self::Dev,
            unknown => Self::Unknown(unknown),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct StateFlags(pub u64);

impl StateFlags {
    pub fn flags(self) -> FlagIter {
        self.into()
    }
}

#[derive(Clone, Debug, Default)]
pub struct FlagIter(Option<FlagIterInner>);

impl FlagIter {
    fn from_valid(valid: ValidIter) -> Self {
        Self(Some(FlagIterInner::Valid(valid)))
    }
}

impl From<StateFlags> for FlagIter {
    fn from(state: StateFlags) -> Self {
        Self(Some(state.into()))
    }
}

impl Iterator for FlagIter {
    type Item = StateFlag;

    fn next(&mut self) -> Option<Self::Item> {
        // Tiny little state machine:
        // - None indicates the iterator is done (trap state)
        // - Invalid will emit invalid once and finish
        // - Valid will pull on the inner iterator till it's finished
        let current = std::mem::take(self);
        let (next, ret) = match current.0? {
            FlagIterInner::Invalid => (Self::default(), StateFlag::Invalid),
            FlagIterInner::Valid(mut valid) => {
                let ret = valid.next()?;
                (Self::from_valid(valid), ret)
            }
        };
        *self = next;
        Some(ret)
    }
}

#[derive(Clone, Debug, Default)]
enum FlagIterInner {
    #[default]
    Invalid,
    Valid(ValidIter),
}

impl From<StateFlags> for FlagIterInner {
    fn from(state: StateFlags) -> Self {
        if state.0 == 0 {
            Self::Invalid
        } else {
            Self::Valid(state.into())
        }
    }
}

#[derive(Clone, Debug)]
struct ValidIter {
    state: StateFlags,
    offset: u8,
}

impl From<StateFlags> for ValidIter {
    fn from(state: StateFlags) -> Self {
        Self { state, offset: 0 }
    }
}

impl Iterator for ValidIter {
    type Item = StateFlag;

    fn next(&mut self) -> Option<Self::Item> {
        // Rotate over each bit and emit each one that is set
        loop {
            let flag = 1u64.checked_shl(self.offset.into())?;
            self.offset = self.offset.checked_add(1)?;
            if self.state.0 & flag != 0 {
                break Some(StateFlag::from_bit_offset(self.offset - 1));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum StateFlag {
    Invalid,
    Uninstalled,
    UpdateRequired,
    FullyInstalled,
    Encrypted,
    Locked,
    FilesMissing,
    AppRunning,
    FilesCorrupt,
    UpdateRunning,
    UpdatePaused,
    UpdateStarted,
    Uninstalling,
    BackupRunning,
    Reconfiguring,
    Validating,
    AddingFiles,
    Preallocating,
    Downloading,
    Staging,
    Committing,
    UpdateStopping,
    Unknown(u8),
}

// More info: https://github.com/lutris/lutris/blob/master/docs/steam.rst
impl StateFlag {
    fn from_bit_offset(offset: u8) -> Self {
        match offset {
            0 => Self::Uninstalled,
            1 => Self::UpdateRequired,
            2 => Self::FullyInstalled,
            3 => Self::Encrypted,
            4 => Self::Locked,
            5 => Self::FilesMissing,
            6 => Self::AppRunning,
            7 => Self::FilesCorrupt,
            8 => Self::UpdateRunning,
            9 => Self::UpdatePaused,
            10 => Self::UpdateStarted,
            11 => Self::Uninstalling,
            12 => Self::BackupRunning,
            16 => Self::Reconfiguring,
            17 => Self::Validating,
            18 => Self::AddingFiles,
            19 => Self::Preallocating,
            20 => Self::Downloading,
            21 => Self::Staging,
            22 => Self::Committing,
            23 => Self::UpdateStopping,
            unknown @ (13..=15 | 24..) => Self::Unknown(unknown),
        }
    }
}

fn time_as_secs_from_unix_epoch(secs: u64) -> Option<time::SystemTime> {
    let offset = time::Duration::from_secs(secs);
    time::SystemTime::UNIX_EPOCH.checked_add(offset)
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum AllowOtherDownloadsWhileRunning {
    UseGlobalSetting,
    Allow,
    Never,
    Unknown(u64),
}

impl From<u64> for AllowOtherDownloadsWhileRunning {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::UseGlobalSetting,
            1 => Self::Allow,
            2 => Self::Never,
            unknown => Self::Unknown(unknown),
        }
    }
}

impl Default for AllowOtherDownloadsWhileRunning {
    fn default() -> Self {
        Self::UseGlobalSetting
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum AutoUpdateBehavior {
    KeepUpToDate,
    OnlyUpdateOnLaunch,
    UpdateWithHighPriority,
    Unknown(u64),
}

impl From<u64> for AutoUpdateBehavior {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::KeepUpToDate,
            1 => Self::OnlyUpdateOnLaunch,
            2 => Self::UpdateWithHighPriority,
            unknown => Self::Unknown(unknown),
        }
    }
}

impl Default for AutoUpdateBehavior {
    fn default() -> Self {
        Self::KeepUpToDate
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
#[non_exhaustive]
pub struct Depot {
    pub manifest: u64,
    pub size: u64,
    #[serde(rename = "dlcappid")]
    pub dlc_app_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InternalApp {
    #[serde(rename = "appid")]
    app_id: u32,
    #[serde(rename = "installdir")]
    install_dir: String,
    #[serde(rename = "Universe")]
    universe: Option<u64>,
    #[serde(rename = "LauncherPath")]
    launcher_path: Option<PathBuf>,
    name: Option<String>,
    #[serde(rename = "StateFlags")]
    state_flags: Option<u64>,
    #[serde(rename = "LastUpdated")]
    last_updated: Option<u64>,
    #[serde(rename = "UpdateResult")]
    update_result: Option<u64>,
    #[serde(rename = "SizeOnDisk")]
    size_on_disk: Option<u64>,
    #[serde(rename = "buildid")]
    build_id: Option<u64>,
    #[serde(rename = "LastOwner")]
    last_user: Option<u64>,
    #[serde(rename = "BytesToDownload")]
    bytes_to_download: Option<u64>,
    #[serde(rename = "BytesDownloaded")]
    bytes_downloaded: Option<u64>,
    #[serde(rename = "BytesToStage")]
    bytes_to_stage: Option<u64>,
    #[serde(rename = "BytesStaged")]
    bytes_staged: Option<u64>,
    #[serde(rename = "StagingSize")]
    staging_size: Option<u64>,
    #[serde(rename = "TargetBuildID")]
    target_build_id: Option<u64>,
    #[serde(rename = "AutoUpdateBehavior")]
    auto_update_behavior: Option<u64>,
    #[serde(rename = "AllowOtherDownloadsWhileRunning")]
    allow_other_downloads_while_running: Option<u64>,
    #[serde(rename = "ScheduledAutoUpdate")]
    scheduled_auto_update: Option<u64>,
    #[serde(rename = "FullValidateBeforeNextUpdate")]
    full_validate_before_next_update: Option<bool>,
    #[serde(rename = "FullValidateAfterNextUpdate")]
    full_validate_after_next_update: Option<bool>,
    #[serde(rename = "InstalledDepots")]
    installed_depots: BTreeMap<u64, Depot>,
    #[serde(default, rename = "StagedDepots")]
    staged_depots: BTreeMap<u64, Depot>,
    #[serde(default, rename = "SharedDepots")]
    shared_depots: BTreeMap<u64, u64>,
    #[serde(rename = "UserConfig")]
    user_config: BTreeMap<String, String>,
    #[serde(default, rename = "MountedConfig")]
    mounted_config: BTreeMap<String, String>,
    #[serde(default, rename = "InstallScripts")]
    install_scripts: BTreeMap<u64, PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_from_manifest_str(s: &str, library_path: &Path) -> App {
        let internal: InternalApp = keyvalues_serde::from_str(s).unwrap();
        App::from_internal_steam_app(internal, library_path)
    }

    #[test]
    fn sanity() {
        let manifest = include_str!("../tests/assets/appmanifest_230410.acf");
        let app = app_from_manifest_str(manifest, Path::new("C:\\redact\\me"));
        // Redact the path because the path separator used is not cross-platform
        insta::assert_ron_snapshot!(app, { ".path" => "[path]" });
    }

    #[test]
    fn more_sanity() {
        let manifest = include_str!("../tests/assets/appmanifest_599140.acf");
        let app = app_from_manifest_str(manifest, Path::new("/redact/me"));
        insta::assert_ron_snapshot!(app, { ".path" => "[path]" });
    }

    #[test]
    fn state_flags() {
        let mut it = StateFlags(0).flags();
        assert_eq!(it.next(), Some(StateFlag::Invalid));
        assert_eq!(it.next(), None);

        let mut it = StateFlags(4).flags();
        assert_eq!(it.next(), Some(StateFlag::FullyInstalled));
        assert_eq!(it.next(), None);

        let mut it = StateFlags(6).flags();
        assert_eq!(it.next(), Some(StateFlag::UpdateRequired));
        assert_eq!(it.next(), Some(StateFlag::FullyInstalled));
        assert_eq!(it.next(), None);
    }
}
