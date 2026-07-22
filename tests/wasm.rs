// TODO: add tests that we can load a steam install using `::from_dir()`
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
#[wasm_bindgen_test::wasm_bindgen_test]
fn locate() {
    let _ = steamlocate::locate().unwrap_err();
}
