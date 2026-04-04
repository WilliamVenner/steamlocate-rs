fn main() {
    let steam_dirs = steamlocate::locate_all().unwrap();
    for steam_dir in steam_dirs {
        println!("Steam Dir - {}", steam_dir.path().display());

        for maybe_library in steam_dir.libraries().unwrap() {
            match maybe_library {
                Err(err) => eprintln!("Failed reading library: {err}"),
                Ok(library) => {
                    println!("    Library - {:?}", library.path());
                    for app in library.apps() {
                        match app {
                            Ok(app) => println!(
                                "        App {} - {}",
                                app.app_id,
                                app.name.as_deref().unwrap_or("<no-name>")
                            ),
                            Err(err) => println!("        Failed reading app: {err}"),
                        }
                    }
                }
            }
        }
    }
}
