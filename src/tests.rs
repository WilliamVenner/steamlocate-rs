// Steam must be installed and at least two library folders must be setup for the tests to succeed
static APP_ID: u32 = 4000; // Specify a Steam app ID

use super::*;

#[test]
fn find_steam() {
	let steamdir_found = SteamDir::locate();
	assert!(steamdir_found.is_some());
	println!("{:?}", steamdir_found.unwrap());
}

#[test]
fn find_library_folders() {
	let steamdir_found = SteamDir::locate();
	assert!(steamdir_found.is_some());

	let mut steamdir = steamdir_found.unwrap();
	
	steamdir.libraryfolders.discover(&steamdir.path);
	assert!(steamdir.libraryfolders.paths.len() > 1);

	println!("{:?}", steamdir.libraryfolders.paths);
}

#[test]
fn find_app() {
	let steamdir_found = SteamDir::locate();
	assert!(steamdir_found.is_some());

	let mut steamdir = steamdir_found.unwrap();
	
	let steamapp = steamdir.get_app(&APP_ID);
	assert!(steamapp.is_some());

	println!("{:?}", steamapp.unwrap());
}

#[test]
fn app_details() {
	let steamdir_found = SteamDir::locate();
	assert!(steamdir_found.is_some());

	let mut steamdir = steamdir_found.unwrap();
	
	let steamapp = steamdir.get_app(&APP_ID);
	assert!(steamapp.is_some());
	
	assert!(steamapp.unwrap().name.is_some());
	assert!(steamapp.unwrap().last_user.is_some());
}