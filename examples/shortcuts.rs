//! Just prints all discovered shortcuts aka all non-Steam added games

fn main() {
    let mut steamdir = steamlocate::SteamDir::locate().unwrap();
    let shortcuts = steamdir.shortcuts();
    println!("Shortcuts - {:#?}", shortcuts);
}
