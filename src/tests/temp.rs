#[cfg(not(any(steamlocate_doctest, test)))]
pub use fake::TempDir;
#[cfg(any(steamlocate_doctest, test))]
pub use real::TempDir;

#[cfg(not(any(steamlocate_doctest, test)))]
mod fake {
    pub struct TempDir;

    impl TempDir {
        // TODO: I think that this can be added to a `.cargo` config file for this project?
        pub fn new() -> Result<TempDir, crate::tests::TestError> {
            unimplemented!("Pass RUSTFLAGS='--cfg steamlocate_doctest' to run doctests");
        }

        pub fn path(&self) -> &std::path::Path {
            panic!();
        }
    }
}

#[cfg(any(steamlocate_doctest, test))]
mod real {
    pub struct TempDir(tempfile::TempDir);

    impl TempDir {
        pub fn new() -> Result<Self, crate::tests::TestError> {
            let temp_dir = tempfile::Builder::new()
                .prefix("steamlocate-test-")
                .tempdir()?;
            Ok(Self(temp_dir))
        }

        pub fn path(&self) -> &std::path::Path {
            self.0.path()
        }
    }
}
