use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
#[cfg_attr(not(feature = "locate"), ignore = "Needs `locate` feature")]
fn locate() {
    #[cfg(not(feature = "locate"))]
    unreachable!("Don't run ignored tests silly");
    #[cfg(feature = "locate")]
    let _ = crate::SteamDir::locate().unwrap_err();
}
