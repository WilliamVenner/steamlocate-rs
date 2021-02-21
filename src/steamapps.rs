use regex::Regex;
use crate::steamapp::SteamApp;
use crate::libraryfolders::LibraryFolders;
use std::collections::HashMap;

lazy_static! {
	static ref APPMANIFEST_RE: Regex = Regex::new(r"^appmanifest_(\d+)\.acf$").unwrap();
}

#[derive(Default, Clone, Debug)]
pub(crate) struct SteamApps {
	pub(crate) apps: HashMap<u32, Option<SteamApp>>,
	pub(crate) discovered: bool
}

impl SteamApps {
	pub(crate) fn discover_apps(&mut self, libraryfolders: &LibraryFolders) {
		self.apps.drain();
		
		for libraryfolder in &libraryfolders.paths {
			let read_dir = libraryfolder.read_dir();
			if read_dir.is_err() { continue }
			for result in read_dir.unwrap() {
				let file = match result {
					Err(_) => continue,
					Ok(file) => file
				};

				let mut path = file.path();
				if !path.is_file() { continue }

				let app_id = match APPMANIFEST_RE.captures(&file.file_name().to_string_lossy()) {
					None => continue,
					Some(captures) => match captures.get(1) {
						None => continue,
						Some(group) => match group.as_str().parse::<u32>() {
							Err(_) => continue,
							Ok(app_id) => app_id
						}
					}
				};

				let vdf = match steamy_vdf::load(&path) {
					Err(_) => continue,
					Ok(vdf) => match vdf.get("AppState") {
						None => continue,
						Some(app_state) => match app_state.as_table() {
							None => continue,
							Some(table) => table.to_owned()
						}
					}
				};

				path.pop(); path.push("common");
				
				self.apps.insert(
					app_id,
					SteamApp::new(&path, &vdf)
				);
			}
		}
	}

	pub(crate) fn discover_app(&mut self, libraryfolders: &LibraryFolders, app_id: &u32) -> Option<()> {
		for libraryfolder in &libraryfolders.paths {
			let mut appmanifest_path = libraryfolder.join(format!("appmanifest_{}.acf", app_id));
			if appmanifest_path.is_file() {
				let appmanifest_vdf = steamy_vdf::load(&appmanifest_path).ok()?;

				appmanifest_path.pop(); appmanifest_path.push("common");

				self.apps.insert(
					*app_id,
					SteamApp::new(&appmanifest_path, appmanifest_vdf.get("AppState")?.as_table()?)
				);

				return None
			}
		}

		self.apps.insert(
			*app_id,
			None
		);

		None
	}
}