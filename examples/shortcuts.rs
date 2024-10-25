//! Just prints all discovered shortcuts aka all non-Steam added games

fn main() {
    let steamdir = steamlocate::SteamDir::locate().unwrap();
    println!("Shortcuts:");
    for maybe_shortcut in steamdir.shortcuts().unwrap() {
        match maybe_shortcut {
            Ok(shortcut) => println!("    - {} {}", shortcut.app_id, shortcut.app_name),
            Err(err) => println!("Failed reading potential shortcut: {err}"),
        }
    }
}
