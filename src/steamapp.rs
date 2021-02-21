use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SteamApp {
	pub appid: u32,
	pub path: PathBuf,
	pub vdf: steamy_vdf::Table,
	pub name: Option<String>,
	
	#[cfg(not(feature="steamid_ng"))]
	pub last_user: Option<u64>,
	#[cfg(feature="steamid_ng")]
	pub last_user: Option<steamid_ng::SteamID>
}

impl SteamApp {
	pub fn new(steamapps: &PathBuf, vdf: &steamy_vdf::Table) -> Option<SteamApp> {
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