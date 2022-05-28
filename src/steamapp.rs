use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

/// An instance of an installed Steam app.
/// # Example
/// ```rust
/// # use steamlocate::SteamDir;
/// let mut steamdir = SteamDir::locate().unwrap();
/// let gmod = steamdir.app(&4000);
/// println!("{:#?}", gmod.unwrap());
/// ```
/// ```ignore
/// SteamApp (
/// 	appid: u32: 4000,
/// 	path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
/// 	vdf: <steamy_vdf::Table>,
/// 	name: Some(String: "Garry's Mod"),
/// 	last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
/// )
/// ```
#[derive(Debug, Clone)]
pub struct SteamApp {
    /// The app ID of this Steam app.
    pub app_id: u32,

    /// The path to the installation directory of this Steam app.
    ///
    /// Example: `C:\Program Files (x86)\Steam\steamapps\common\GarrysMod`
    pub path: PathBuf,

    /// The store name of the Steam app.
    pub name: String,

    pub universe: u64,
    pub launcher_path: Option<PathBuf>,
    pub state_flags: u64,
    pub last_updated: u64,
    pub update_result: Option<u64>,
    pub size_on_disk: u64,
    pub build_id: u64,
    pub bytes_to_download: Option<u64>,
    pub bytes_downloaded: Option<u64>,
    pub bytes_to_stage: Option<u64>,
    pub bytes_staged: Option<u64>,
    pub staging_size: Option<u64>,
    pub target_build_id: Option<u64>,
    pub auto_update_behavior: u64,
    pub allow_other_downloads_while_running: u64,
    pub scheduled_auto_update: u64,
    pub full_validate_before_next_update: Option<bool>,
    pub full_validate_after_next_update: Option<bool>,
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
    pub last_user: u64,

    #[cfg(feature = "steamid_ng")]
    /// The [SteamID](https://docs.rs/steamid-ng/*/steamid_ng/struct.SteamID.html) of the last Steam user that played this game on the filesystem.
    pub last_user: steamid_ng::SteamID,
}

impl SteamApp {
    pub(crate) fn new(library_path: &Path, manifest: &Path) -> Option<Self> {
        let contents = fs::read_to_string(manifest).ok()?;
        let InternalSteamApps {
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
        } = keyvalues_serde::from_str(&contents).ok()?;

        // First check if the installation path exists and is a valid directory
        let path = library_path.join("common").join(install_dir);
        if !path.is_dir() {
            return None;
        }

        #[cfg(feature = "steamid_ng")]
        let last_user = last_user.map(steamid_ng::SteamID::from);

        Some(Self {
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

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Depot {
    pub manifest: u64,
    pub size: u64,
    #[serde(rename = "dlcappid")]
    pub dlc_app_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct InternalSteamApps {
    #[serde(rename = "appid")]
    app_id: u32,
    #[serde(rename = "Universe")]
    universe: u64,
    #[serde(rename = "LauncherPath")]
    launcher_path: Option<PathBuf>,
    name: String,
    #[serde(rename = "StateFlags")]
    state_flags: u64,
    #[serde(rename = "installdir")]
    install_dir: String,
    #[serde(rename = "LastUpdated")]
    last_updated: u64,
    #[serde(rename = "UpdateResult")]
    update_result: Option<u64>,
    #[serde(rename = "SizeOnDisk")]
    size_on_disk: u64,
    #[serde(rename = "buildid")]
    build_id: u64,
    #[serde(rename = "LastOwner")]
    last_user: u64,
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
    auto_update_behavior: u64,
    #[serde(rename = "AllowOtherDownloadsWhileRunning")]
    allow_other_downloads_while_running: u64,
    #[serde(rename = "ScheduledAutoUpdate")]
    scheduled_auto_update: u64,
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
