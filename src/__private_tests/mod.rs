pub mod helpers;
#[cfg(test)]
mod legacy;
mod temp;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wasm;

pub type TestError = Box<dyn std::error::Error>;
pub type TestResult = Result<(), TestError>;

pub mod prelude {
    pub use super::{
        helpers::{
            expect_test_env, AppFile, SampleApp, SampleShortcuts, TempLibrary, TempSteamDir,
        },
        TestError, TestResult,
    };
}
