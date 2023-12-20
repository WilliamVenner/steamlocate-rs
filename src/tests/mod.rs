pub mod helpers;
#[cfg(test)]
mod legacy;
mod temp;
#[cfg(test)]
mod wasm;

pub type TestError = Box<dyn std::error::Error>;
pub type TestResult = Result<(), TestError>;
