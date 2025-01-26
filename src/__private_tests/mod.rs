pub mod helpers;
mod temp;

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
