//! Just prints all discovered shortcuts aka all non-Steam added games

fn main() {
    let mut steamdir = steamlocate::SteamDir::locate().unwrap();
    println!("Shortcuts:");
    for maybe_shortcut in steamdir.shortcuts().unwrap() {
        match maybe_shortcut {
            Some(shortcut) => println!("    - {} {}", shortcut.appid, shortcut.app_name),
            None => println!("Failed reading potential shortcut"),
        }
    }
}
