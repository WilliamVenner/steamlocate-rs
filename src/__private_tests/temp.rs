//! `TempDir` at home
//!
//! I want to use temporary directories in doctests, but that works against your public API.
//! Luckily all the functionality we need is very easy to replicate

use std::{collections, env, fs, hash, path};

use super::TestError;

#[derive(Debug)]
pub struct TempDir(Option<path::PathBuf>);

impl TempDir {
    pub fn new() -> Result<Self, TestError> {
        let mut dir = env::temp_dir();
        let random_name = format!("steamlocate-test-{:x}", random_seed());
        dir.push(random_name);
        fs::create_dir_all(&dir)?;
        Ok(Self(Some(dir)))
    }

    pub fn path(&self) -> &path::Path {
        self.0.as_deref().unwrap()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn random_seed() -> u64 {
    hash::Hasher::finish(&hash::BuildHasher::build_hasher(
        &collections::hash_map::RandomState::new(),
    ))
}
