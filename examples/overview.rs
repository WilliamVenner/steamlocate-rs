use steamlocate::SteamDir;

fn main() {
    let steamdir = SteamDir::locate().unwrap();
    println!("Steam Dir - {:?}", steamdir.path());

    for maybe_library in steamdir.libraries().unwrap() {
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
