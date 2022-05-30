use steamlocate::SteamDir;

fn main() {
    let steamdir = SteamDir::locate().unwrap();
    println!("Steam Dir - {:?}", steamdir.path());

    for library in steamdir.libraries() {
        println!("    Library - {:?}", library.path());
        for app in library.apps() {
            match app {
                Some(app) => println!(
                    "        App {} - {}",
                    app.appid,
                    app.name.as_deref().unwrap_or("<no-name>")
                ),
                None => println!("        There was a FAILURE!"),
            }
        }
    }
}
