use anyhow::{Context, Ok, Result};
use steamlocate::SteamDir;
fn main() -> Result<()> {
    let steamdir = SteamDir::locate().unwrap();
    println!("Steam Dir - {:?}", steamdir.path());

    // TODO: use `anyhow` to make error handling here simpler
    for maybe_library in steamdir.libraries().unwrap() {
        let lib = maybe_library.with_context(|| "Failed reading library")?;
        println!("     Library - {:?}", lib.path());
        for app in lib.apps() {
            let app = app.with_context(|| "Failed reading app")?;
            println!(
                "        App {} - {}",
                app.app_id,
                app.name.as_deref().unwrap_or("<no-name>")
            );
        }
    }
    Ok(())
}
