use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time,
};

use crate::{error::ParseErrorKind, Error, ParseError, Result};

use serde::Deserialize;

/// An instance of an installed Steam app.
/// # Example
/// ```ignore
/// # use steamlocate::SteamDir;
/// let mut steamdir = SteamDir::locate().unwrap();
/// let gmod = steamdir.app(&4000);
/// println!("{:#?}", gmod.unwrap());
/// ```
/// ```ignore
/// SteamApp (
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
pub struct SteamApp {
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
    pub state_flags: Option<Vec<StateFlag>>,
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

    #[cfg(not(feature = "steamid_ng"))]
    /// The SteamID64 of the last Steam user that played this game on the filesystem.
    ///
    /// This crate supports [steamid-ng](https://docs.rs/steamid-ng) and can automatically convert this to a [SteamID](https://docs.rs/steamid-ng/*/steamid_ng/struct.SteamID.html) for you.
    ///
    /// To enable this support, [use the  `steamid_ng` Cargo.toml feature](https://docs.rs/steamlocate/*/steamlocate#using-steamlocate).
    pub last_user: Option<u64>,

    #[cfg(feature = "steamid_ng")]
    /// The [SteamID](https://docs.rs/steamid-ng/*/steamid_ng/struct.SteamID.html) of the last Steam user that played this game on the filesystem.
    pub last_user: Option<steamid_ng::SteamID>,
}

impl SteamApp {
    pub(crate) fn new(library_path: &Path, manifest: &Path) -> Result<Self> {
        let contents = fs::read_to_string(manifest).map_err(Error::Io)?;
        let app = Self::from_manifest_str(library_path, &contents)?;

        // Check if the installation path exists and is a valid directory
        // TODO: lint against printing
        println!("{:?}", app.path);
        if app.path.is_dir() {
            Ok(app)
        } else {
            // TODO: app id here
            Err(Error::MissingExpectedApp { app_id: 1 })
        }
    }

    pub(crate) fn from_manifest_str(library_path: &Path, manifest: &str) -> Result<Self> {
        let InternalSteamApp {
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
        } = keyvalues_serde::from_str(manifest).map_err(|err| Error::Parse {
            kind: ParseErrorKind::SteamApp,
            error: ParseError::from_serde(err),
        })?;

        let path = library_path
            .join("steamapps")
            .join("common")
            .join(install_dir);

        #[cfg(feature = "steamid_ng")]
        let last_user = last_user.map(steamid_ng::SteamID::from);
        let universe = universe.map(Universe::from);
        let state_flags = state_flags.map(StateFlag::flags_from_packed);
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

        Ok(Self {
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
        })
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
    // TODO: should this just store the bit offset?
    Unknown(u64),
}

// More info: https://github.com/lutris/lutris/blob/master/docs/steam.rst
impl StateFlag {
    fn flags_from_packed(bit_flags: u64) -> Vec<StateFlag> {
        const FLAG_TO_MASK: &[(StateFlag, u64)] = &[
            (StateFlag::Uninstalled, 1),
            (StateFlag::UpdateRequired, 2),
            (StateFlag::FullyInstalled, 4),
            (StateFlag::Encrypted, 8),
            (StateFlag::Locked, 16),
            (StateFlag::FilesMissing, 32),
            (StateFlag::AppRunning, 64),
            (StateFlag::FilesCorrupt, 128),
            (StateFlag::UpdateRunning, 256),
            (StateFlag::UpdatePaused, 512),
            (StateFlag::UpdateStarted, 1024),
            (StateFlag::Uninstalling, 2048),
            (StateFlag::BackupRunning, 4096),
            (StateFlag::Reconfiguring, 65536),
            (StateFlag::Validating, 131072),
            (StateFlag::AddingFiles, 262144),
            (StateFlag::Preallocating, 524288),
            (StateFlag::Downloading, 1048576),
            (StateFlag::Staging, 2097152),
            (StateFlag::Committing, 4194304),
            (StateFlag::UpdateStopping, 8388608),
        ];
        if bit_flags == 0 {
            vec![StateFlag::Invalid]
        } else {
            FLAG_TO_MASK
                .iter()
                .filter_map(|&(flag, mask)| {
                    if bit_flags & mask > 0 {
                        Some(flag)
                    } else {
                        // TODO: this should be `Unknown`
                        None
                    }
                })
                .collect()
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
struct InternalSteamApp {
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

    #[test]
    fn sanity() {
        let manifest = include_str!("../tests/assets/appmanifest_230410.acf");
        let app = SteamApp::from_manifest_str(Path::new("C:\\redact\\me"), manifest).unwrap();
        // Redact the path because the path separator used is not cross-platform
        insta::assert_ron_snapshot!(app, { ".path" => "[path]" });
    }

    #[test]
    fn more_sanity() {
        let manifest = include_str!("../tests/assets/appmanifest_599140.acf");
        let app = SteamApp::from_manifest_str(Path::new("/redact/me"), manifest).unwrap();
        insta::assert_ron_snapshot!(app, { ".path" => "[path]" });
    }
}
