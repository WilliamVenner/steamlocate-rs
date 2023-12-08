//! **WARN:** This is all hacky and should be replaced with proper binary VDF parsing

use std::{
    fs, io,
    iter::Peekable,
    path::{Path, PathBuf},
    slice,
};

use crate::{
    error::{ParseError, ParseErrorKind},
    Error, Result,
};

/// A added non-Steam game
///
/// Information is parsed from your `userdata/<user_id>/config/shortcuts.vdf` files
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Shortcut {
    /// Steam's provided app id
    pub appid: u32,
    /// The name of the application
    pub app_name: String,
    /// The executable used to launch the app
    ///
    /// This is either the name of the program or the full path to the program
    pub executable: String,
    /// The directory that the application should be run in
    pub start_dir: String,
}

#[cfg(feature = "shortcuts_extras")]
impl Shortcut {
    /// Calculates the shortcut's Steam ID from the executable and app name
    pub fn steam_id(&self) -> u64 {
        let algorithm = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

        let mut digest = algorithm.digest();
        digest.update(self.executable.as_bytes());
        digest.update(self.app_name.as_bytes());

        let top = digest.finalize() | 0x80000000;
        ((top as u64) << 32) | 0x02000000
    }
}

pub struct Iter {
    dir: PathBuf,
    read_dir: fs::ReadDir,
    pending: std::vec::IntoIter<Shortcut>,
}

impl Iter {
    pub(crate) fn new(steam_dir: &Path) -> Result<Self> {
        let user_data = steam_dir.join("userdata");
        if !user_data.is_dir() {
            return Err(Error::parse(
                ParseErrorKind::Shortcut,
                ParseError::missing(),
                &user_data,
            ));
        }

        let read_dir = fs::read_dir(&user_data).map_err(|io| Error::io(io, &user_data))?;
        Ok(Self {
            dir: user_data,
            read_dir,
            pending: Vec::new().into_iter(),
        })
    }
}

impl Iterator for Iter {
    type Item = Result<Shortcut>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = loop {
            if let Some(shortcut) = self.pending.next() {
                break Ok(shortcut);
            }

            // Need to parse the next set of pending shortcuts
            let maybe_entry = self.read_dir.next()?;
            match maybe_entry {
                Ok(entry) => {
                    let shortcuts_path = entry.path().join("config").join("shortcuts.vdf");
                    match fs::read(&shortcuts_path) {
                        Ok(contents) => {
                            if let Some(shortcuts) = parse_shortcuts(&contents) {
                                self.pending = shortcuts.into_iter();
                                continue;
                            } else {
                                break Err(Error::parse(
                                    ParseErrorKind::Shortcut,
                                    ParseError::unexpected_structure(),
                                    &shortcuts_path,
                                ));
                            }
                        }
                        Err(err) => {
                            // Not every directory in here has a shortcuts file
                            if err.kind() == io::ErrorKind::NotFound {
                                continue;
                            } else {
                                break Err(Error::io(err, &shortcuts_path));
                            }
                        }
                    }
                }
                Err(err) => break Err(Error::io(err, &self.dir)),
            }
        };

        Some(item)
    }
}

/// Advances `it` until right after the matching `needle`
///
/// Only works if the starting byte is not used anywhere else in the needle. This works well when
/// finding keys since the starting byte indicates the type and wouldn't be used in the key
#[must_use]
fn after_many_case_insensitive(it: &mut Peekable<slice::Iter<u8>>, needle: &[u8]) -> bool {
    loop {
        let mut needle_it = needle.iter();
        let b = match it.next() {
            Some(b) => b,
            None => return false,
        };

        let maybe_needle_b = needle_it.next();
        if maybe_u8_eq_ignore_ascii_case(maybe_needle_b, Some(b)) {
            loop {
                if needle_it.len() == 0 {
                    return true;
                }

                let maybe_b = it.peek();
                let maybe_needle_b = needle_it.next();
                if maybe_u8_eq_ignore_ascii_case(maybe_needle_b, maybe_b.copied()) {
                    let _ = it.next();
                } else {
                    break;
                }
            }
        }
    }
}

fn maybe_u8_eq_ignore_ascii_case(maybe_b1: Option<&u8>, maybe_b2: Option<&u8>) -> bool {
    maybe_b1
        .zip(maybe_b2)
        .map(|(b1, b2)| b1.eq_ignore_ascii_case(b2))
        .unwrap_or_default()
}

fn parse_value_str(it: &mut Peekable<slice::Iter<u8>>) -> Option<String> {
    let mut buff = Vec::new();
    loop {
        let b = it.next()?;
        if *b == 0x00 {
            break Some(String::from_utf8_lossy(&buff).into_owned());
        }

        buff.push(*b);
    }
}

fn parse_value_u32(it: &mut Peekable<slice::Iter<u8>>) -> Option<u32> {
    let bytes = [*it.next()?, *it.next()?, *it.next()?, *it.next()?];
    Some(u32::from_le_bytes(bytes))
}

fn parse_shortcuts(contents: &[u8]) -> Option<Vec<Shortcut>> {
    let mut it = contents.iter().peekable();
    let mut shortcuts = Vec::new();

    loop {
        if !after_many_case_insensitive(&mut it, b"\x02appid\x00") {
            return Some(shortcuts);
        }
        let appid = parse_value_u32(&mut it)?;

        if !after_many_case_insensitive(&mut it, b"\x01AppName\x00") {
            return None;
        }
        let app_name = parse_value_str(&mut it)?;

        if !after_many_case_insensitive(&mut it, b"\x01Exe\x00") {
            return None;
        }
        let executable = parse_value_str(&mut it)?;

        if !after_many_case_insensitive(&mut it, b"\x01StartDir\x00") {
            return None;
        }
        let start_dir = parse_value_str(&mut it)?;

        let shortcut = Shortcut {
            appid,
            app_name,
            executable,
            start_dir,
        };
        shortcuts.push(shortcut);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let contents = include_bytes!("../tests/sample_data/shortcuts.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        assert_eq!(
            shortcuts,
            vec![
                Shortcut {
                    appid: 2786274309,
                    app_name: "Anki".into(),
                    executable: "\"anki\"".into(),
                    start_dir: "\"./\"".into(),
                },
                Shortcut {
                    appid: 2492174738,
                    app_name: "LibreOffice Calc".into(),
                    executable: "\"libreoffice\"".into(),
                    start_dir: "\"./\"".into(),
                },
                Shortcut {
                    appid: 3703025501,
                    app_name: "foo.sh".into(),
                    executable: "\"/usr/local/bin/foo.sh\"".into(),
                    start_dir: "\"/usr/local/bin/\"".into(),
                }
            ],
        );

        let contents = include_bytes!("../tests/sample_data/shortcuts_different_key_case.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        assert_eq!(
            shortcuts,
            vec![Shortcut {
                appid: 2931025216,
                app_name: "Second Life".into(),
                executable: "\"/Applications/Second Life Viewer.app\"".into(),
                start_dir: "\"/Applications/\"".into(),
            }]
        );
    }

    #[cfg_attr(
        not(feature = "shortcuts_extras"),
        ignore = "Needs `shortcuts_extras` feature"
    )]
    #[test]
    fn shortcuts_extras() {
        #[cfg(not(feature = "shortcuts_extras"))]
        unreachable!();
        #[cfg(feature = "shortcuts_extras")]
        {
            let contents = include_bytes!("../tests/sample_data/shortcuts.vdf");
            let shortcuts = parse_shortcuts(contents).unwrap();
            let ideal_ids = vec![0xe89614fe02000000, 0xdb01c79902000000, 0x9d55017302000000];
            for (id, shortcut) in ideal_ids.into_iter().zip(shortcuts.iter()) {
                assert_eq!(id, shortcut.steam_id());
            }
        }
    }
}
