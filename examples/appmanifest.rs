use std::{env, process::exit};

use steamlocate::SteamDir;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 || args[1].parse::<u32>().is_err() {
        eprintln!("Usage: cargo run --example appmanifest -- <STEAM_APP_ID>");
        exit(1);
    }
    let app_id: u32 = args[1].parse().unwrap();

    let steam_dir = SteamDir::locate().unwrap();
    match steam_dir.app(app_id) {
        Some(app) => println!("Found app - {:#?}", app),
        None => println!("No app found for {}", app_id),
    }
}
