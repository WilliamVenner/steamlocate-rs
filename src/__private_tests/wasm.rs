use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
#[cfg_attr(
    not(any(target_os = "windows", target_os = "macos", target_os = "linux")),
    ignore = "Needs `locate` feature"
)]
fn locate() {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    unreachable!("Don't run ignored tests silly");
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let _ = crate::SteamDir::locate().unwrap_err();
}
