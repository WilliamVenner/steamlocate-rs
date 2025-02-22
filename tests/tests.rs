use steamlocate::__private_tests::prelude::*;

// Context: https://github.com/WilliamVenner/steamlocate-rs/issues/58
#[test]
fn app_lastupdated_casing() -> TestResult {
    let sample_app = SampleApp::Resonite;
    let temp_steam_dir: TempSteamDir = sample_app.try_into()?;
    let steam_dir = temp_steam_dir.steam_dir();

    let (app, _library) = steam_dir.find_app(sample_app.id())?.unwrap();
    // Last updated _should_ be `Some(_)` for this app even though it uses lowercase casing
    let _ = app.last_updated.unwrap();

    Ok(())
}
