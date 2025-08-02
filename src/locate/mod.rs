use crate::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use crate::locate::linux::locate_steam_dir_helper;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::locate::windows::locate_steam_dir_helper;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::locate::macos::locate_steam_dir_helper;

#[derive(Clone, Debug, Default)]
pub enum InstallationType {
    LinuxStandard,
    LinuxFlatpak,
    LinuxSnap,
    MacosStandard,
    #[default]
    WindowsStandard,
}

#[cfg(target_os = "linux")]
pub fn locate_steam_dir() -> Result<Vec<(std::path::PathBuf, InstallationType)>> {
    locate_steam_dir_helper()
}
#[cfg(not(target_os = "linux"))]
pub fn locate_steam_dir() -> Result<Vec<(std::path::PathBuf, InstallationType)>> {
    locate_steam_dir_helper().map(|path| vec![path])
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn locate_steam_dir_helper() -> Result<(std::path::PathBuf, InstallationType)> {
    use crate::error::{Error, LocateError};
    Err(Error::locate(LocateError::Unsupported))
}
