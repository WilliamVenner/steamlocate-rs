use std::path::PathBuf;

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
	pub appid: u32,

	/// The path to the installation directory of this Steam app.
	/// Example: `C:\Program Files (x86)\Steam\steamapps\common\GarrysMod`
	pub path: PathBuf,

	/// A [steamy_vdf::Table](/steamy-vdf/*/steamy_vdf/struct.Table.html)
	pub vdf: steamy_vdf::Table,

	/// The store name of the Steam app.
	pub name: Option<String>,
	
	#[cfg(not(feature="steamid_ng"))]
	/// The SteamID64 of the last Steam user that played this game on the filesystem.
	///
	/// This crate supports [steamid-ng](/steamid-ng) and can automatically convert this to a [SteamID](/steamid-ng/*/steamid-ng/struct.SteamID.html) for you.
	///
	/// To enable this feature, build with `cargo build --features steamid_ng`
	pub last_user: Option<u64>,

	#[cfg(feature="steamid_ng")]
	/// The [SteamID](/steamid-ng/*/steamid-ng/struct.SteamID.html) of the last Steam user that played this game on the filesystem.
	pub last_user: Option<steamid_ng::SteamID>
}

impl SteamApp {
	pub(crate) fn new(steamapps: &PathBuf, vdf: &steamy_vdf::Table) -> Option<SteamApp> {
		// First check if the installation path exists and is a valid directory
		let install_dir = steamapps.join(vdf.get("installdir")?.as_str()?);
		if !install_dir.is_dir() { return None }

		Some(SteamApp {
			vdf: vdf.clone(),
			path: install_dir,

			// Get the appid key, try and parse it as an unsigned 32-bit integer, if we fail, return None
			appid: vdf.get("appid")?.as_value()?.parse::<u32>().ok()?,

			// Get the name key, try and convert it into a String, if we fail, name = None
			name: vdf.get("name").and_then(|entry| entry.as_str().and_then(|str| Some(str.to_string()))),

			// Get the LastOwner key, try and convert it into a SteamID64, if we fail, last_user = None
			#[cfg(not(feature="steamid_ng"))]
			last_user: vdf.get("LastOwner").and_then(
				|entry| entry.as_value().and_then(
					|val| val.parse::<u64>().ok()
				)
			),

			#[cfg(feature="steamid_ng")]
			last_user: vdf.get("LastOwner").and_then(
				|entry| entry.as_value().and_then(
					|val| val.parse::<u64>().ok().and_then(
						|steamid64| Some(steamid_ng::SteamID::from(steamid64))
					)
				)
			),
		})
	}
}