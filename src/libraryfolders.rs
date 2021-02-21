use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub(crate) struct LibraryFolders {
	pub(crate) paths: Vec<PathBuf>,
	pub(crate) discovered: bool
}

impl LibraryFolders {
	pub(crate) fn discover(&mut self, path: &PathBuf) {
		let steamapps = path.join("SteamApps");
		self.paths.push(steamapps.clone());

		let libraryfolders_vdf_path = steamapps.join("libraryfolders.vdf");
		if libraryfolders_vdf_path.is_file() {

			// Load LibraryFolders table
			match
				steamy_vdf::load(libraryfolders_vdf_path).as_ref()

				.and_then(|vdf| vdf.get("LibraryFolders")
					.ok_or(&steamy_vdf::Error::Parse)

					.and_then(|entry| entry.as_table()
						.ok_or(&steamy_vdf::Error::Parse)
					)
				)
			{
				Err(_) => {},
				Ok(libraryfolders_vdf) => {
					self.paths.append(
						// Filter out non-numeric keys and convert library folder Strings to PathBufs
						&mut libraryfolders_vdf.keys().filter_map(|key| {
							key.parse::<u32>().ok()?;
							Some(PathBuf::from(
								libraryfolders_vdf.get(key)?.as_str()?.to_string()
							).join("SteamApps"))
						}).collect::<Vec<PathBuf>>()
					)
				}
			}

		}
		
		self.discovered = true;
	}
}
