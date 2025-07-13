//! Just prints all discovered shortcuts aka all non-Steam added games

fn main() {
    let steamdir = steamlocate::SteamDir::locate_multiple().unwrap();
    println!("Dirs:");
    for dir in steamdir {
        println!(
            "{:?} : {}",
            dir.installation_type(),
            dir.path().to_str().unwrap_or_default()
        )
    }
}
