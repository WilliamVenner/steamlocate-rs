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

use serde::{Deserialize, Deserializer};

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

impl Iterator for Iter<'_> {
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

/// Metadata for an installed Steam app
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
#[non_exhaustive]
#[serde(rename_all = "PascalCase")]
pub struct App {
    /// The app ID of this Steam app
    #[serde(rename = "appid")]
    pub app_id: u32,
    /// The name of the installation directory of this Steam app e.g. `"GarrysMod"`
    ///
    /// If you're trying to get the app's installation directory then take a look at
    /// [`Library::resolve_app_dir()`][crate::Library::resolve_app_dir]
    #[serde(rename = "installdir")]
    pub install_dir: String,
    /// The store name of the Steam app
    #[serde(rename = "name")]
    pub name: Option<String>,
    /// The SteamID64 of the last Steam user that played this game on the filesystem
    #[serde(rename = "LastOwner")]
    pub last_user: Option<u64>,

    pub universe: Option<Universe>,
    pub launcher_path: Option<PathBuf>,
    pub state_flags: Option<StateFlags>,
    // NOTE: Need to handle this for serializing too before `App` can `impl Serialize`
    #[serde(
        alias = "lastupdated",
        default,
        deserialize_with = "de_time_as_secs_from_unix_epoch"
    )]
    pub last_updated: Option<time::SystemTime>,
    // Can't find anything on what these values mean. I've seen 0, 2, 4, 6, and 7
    pub update_result: Option<u64>,
    pub size_on_disk: Option<u64>,
    #[serde(rename = "buildid")]
    pub build_id: Option<u64>,
    pub bytes_to_download: Option<u64>,
    pub bytes_downloaded: Option<u64>,
    pub bytes_to_stage: Option<u64>,
    pub bytes_staged: Option<u64>,
    pub staging_size: Option<u64>,
    #[serde(rename = "TargetBuildID")]
    pub target_build_id: Option<u64>,
    pub auto_update_behavior: Option<AutoUpdateBehavior>,
    pub allow_other_downloads_while_running: Option<AllowOtherDownloadsWhileRunning>,
    pub scheduled_auto_update: Option<ScheduledAutoUpdate>,
    pub full_validate_before_next_update: Option<bool>,
    pub full_validate_after_next_update: Option<bool>,
    #[serde(default)]
    pub installed_depots: BTreeMap<u64, Depot>,
    #[serde(default)]
    pub staged_depots: BTreeMap<u64, Depot>,
    #[serde(default)]
    pub user_config: BTreeMap<String, String>,
    #[serde(default)]
    pub mounted_config: BTreeMap<String, String>,
    #[serde(default)]
    pub install_scripts: BTreeMap<u64, PathBuf>,
    #[serde(default)]
    pub shared_depots: BTreeMap<u64, u64>,
}

impl App {
    pub(crate) fn new(manifest: &Path) -> Result<Self> {
        let contents = fs::read_to_string(manifest).map_err(|io| Error::io(io, manifest))?;
        keyvalues_serde::from_str(&contents)
            .map_err(|err| Error::parse(ParseErrorKind::App, ParseError::from_serde(err), manifest))
    }
}

macro_rules! impl_deserialize_from_u64 {
    ( $ty_name:ty ) => {
        impl<'de> Deserialize<'de> for $ty_name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = u64::deserialize(deserializer)?;
                Ok(Self::from(value))
            }
        }
    };
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

impl_deserialize_from_u64!(Universe);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct StateFlags(pub u64);

impl StateFlags {
    pub fn flags(self) -> StateFlagIter {
        self.into()
    }
}

#[derive(Clone, Debug)]
pub struct StateFlagIter(Option<StateFlagIterInner>);

impl From<StateFlags> for StateFlagIter {
    fn from(state: StateFlags) -> Self {
        Self(Some(state.into()))
    }
}

impl Iterator for StateFlagIter {
    type Item = StateFlag;

    fn next(&mut self) -> Option<Self::Item> {
        // Tiny little state machine:
        // - None indicates the iterator is done (trap state)
        // - Invalid will emit invalid once and finish
        // - Valid will pull on the inner iterator till it's finished
        let current = std::mem::take(&mut self.0);
        let (next, ret) = match current? {
            StateFlagIterInner::Invalid => (None, StateFlag::Invalid),
            StateFlagIterInner::Valid(mut valid) => {
                let ret = valid.next()?;
                (Some(StateFlagIterInner::Valid(valid)), ret)
            }
        };
        self.0 = next;
        Some(ret)
    }
}

#[derive(Clone, Debug)]
enum StateFlagIterInner {
    Invalid,
    Valid(ValidIter),
}

impl From<StateFlags> for StateFlagIterInner {
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

fn de_time_as_secs_from_unix_epoch<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<time::SystemTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let maybe_time =
        <Option<u64>>::deserialize(deserializer)?.and_then(time_as_secs_from_unix_epoch);
    Ok(maybe_time)
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

impl_deserialize_from_u64!(AllowOtherDownloadsWhileRunning);

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

impl_deserialize_from_u64!(AutoUpdateBehavior);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub enum ScheduledAutoUpdate {
    Zero,
    Time(time::SystemTime),
}

impl<'de> Deserialize<'de> for ScheduledAutoUpdate {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let sched_auto_upd = match u64::deserialize(deserializer)? {
            0 => Self::Zero,
            secs => {
                let time = time_as_secs_from_unix_epoch(secs)
                    .ok_or_else(|| serde::de::Error::custom("Exceeded max time"))?;
                Self::Time(time)
            }
        };
        Ok(sched_auto_upd)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn app_from_manifest_str(s: &str) -> App {
        keyvalues_serde::from_str(s).unwrap()
    }

    #[test]
    fn minimal() {
        let minimal = r#"
"AppState"
{
	"appid"		"2519830"
	"installdir" "Resonite"
}
"#;

        let app = app_from_manifest_str(minimal);
        insta::assert_ron_snapshot!(app);
    }

    #[test]
    fn sanity() {
        let manifest = include_str!("../tests/assets/appmanifest_230410.acf");
        let app = app_from_manifest_str(manifest);
        insta::assert_ron_snapshot!(app);
    }

    #[test]
    fn more_sanity() {
        let manifest = include_str!("../tests/assets/appmanifest_599140.acf");
        let app = app_from_manifest_str(manifest);
        insta::assert_ron_snapshot!(app);
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
