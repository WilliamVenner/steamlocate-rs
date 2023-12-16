pub mod helpers;
#[cfg(test)]
mod legacy;
mod temp;

pub type TestError = Box<dyn std::error::Error>;
pub type TestResult = Result<(), TestError>;
