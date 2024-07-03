use super::{
    helpers::{SampleApp, TempSteamDir},
    TestResult,
};

// Context: https://github.com/WilliamVenner/steamlocate-rs/issues/58
#[test]
fn app_lastupdated_casing() -> TestResult {
    let sample_app = SampleApp::Resonite;
    let temp_steam_dir = TempSteamDir::builder().app(sample_app.into()).finish()?;
    let steam_dir = temp_steam_dir.steam_dir();

    let (app, _library) = steam_dir.find_app(sample_app.id())?.unwrap();
    // Last updated _should_ be `Some(_)` for this app even though it uses lowercase casing
    let _ = app.last_updated.unwrap();

    Ok(())
}
