use crate::libraryfolders::LibraryFolders;
use crate::steamapp::SteamApp;
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub(crate) struct SteamApps {
    pub(crate) apps: HashMap<u32, Option<SteamApp>>,
    pub(crate) discovered: bool,
}

impl SteamApps {
    pub(crate) fn discover_apps(&mut self, libraryfolders: &LibraryFolders) {
        self.apps.drain();

        for libraryfolder in &libraryfolders.paths {
            let read_dir = libraryfolder.read_dir();
            if read_dir.is_err() {
                continue;
            }

            let library_base = libraryfolder.clone();
            for result in read_dir.unwrap() {
                let file = match result {
                    Err(_) => continue,
                    Ok(file) => file,
                };

                let path = file.path();
                if !path.is_file() {
                    continue;
                }

                let app_id: u32 = match file
                    .file_name()
                    .to_str()
                    .and_then(|name| name.strip_prefix("appmanifest_"))
                    .and_then(|prefixless_name| prefixless_name.strip_suffix(".acf"))
                    .and_then(|app_id_str| app_id_str.parse().ok())
                {
                    Some(app_id) => app_id,
                    None => continue,
                };

                self.apps
                    .insert(app_id, SteamApp::new(&library_base, &path));
            }
        }
    }

    pub(crate) fn discover_app(
        &mut self,
        libraryfolders: &LibraryFolders,
        app_id: &u32,
    ) -> Option<()> {
        for libraryfolder in &libraryfolders.paths {
            let appmanifest_path = libraryfolder.join(format!("appmanifest_{}.acf", app_id));
            if appmanifest_path.is_file() {
                self.apps
                    .insert(*app_id, SteamApp::new(libraryfolder, &appmanifest_path));

                return Some(());
            }
        }

        None
    }
}
