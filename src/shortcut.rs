// HACK: This is all hacky and should be replaced with proper binary VDF parsing

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    error::{ParseError, ParseErrorKind},
    Error, Result,
};

/// A non-Steam game that has been added to Steam
///
/// Information is parsed from your `userdata/<user_id>/config/shortcuts.vdf` files
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Shortcut {
    /// Steam's provided app id
    pub app_id: u32,
    /// The name of the application
    pub app_name: String,
    /// The executable used to launch the app
    ///
    /// This is either the name of the program or the full path to the program
    pub executable: String,
    /// The directory that the application should be run in
    pub start_dir: String,
}

impl Shortcut {
    /// Calculates the shortcut's Steam ID from the executable and app name
    pub fn new(app_id: u32, app_name: String, executable: String, start_dir: String) -> Self {
        Self {
            app_id,
            app_name,
            executable,
            start_dir,
        }
    }

    /// The shortcut's Steam ID calculated from the executable path and app name
    pub fn steam_id(&self) -> u64 {
        let executable = self.executable.as_bytes();
        let app_name = self.app_name.as_bytes();

        let algorithm = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

        let mut digest = algorithm.digest();
        digest.update(executable);
        digest.update(app_name);

        let top = digest.finalize() | 0x80000000;
        ((top as u64) << 32) | 0x02000000
    }
}

/// An [`Iterator`] over a Steam installation's [`Shortcut`]s
///
/// Returned from calling [`SteamDir::shortcuts()`][super::SteamDir::shortcuts]
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

fn parse_shortcuts(contents: &[u8]) -> Option<Vec<Shortcut>> {
    let mut shortcuts = Vec::new();
    let mut factory = ShortcutFactory::default();

    for maybe_field_start in contents
        .iter()
        .enumerate()
        .filter(|(_, b)| [1, 2].contains(b))
        .map(|(i, _)| i)
    {
        let kind = contents[maybe_field_start];
        let null_at = contents[1 + maybe_field_start..]
            .iter()
            .enumerate()
            .filter(|(_, &b)| b == 0)
            .map(|(i, _)| i)
            .next()?;
        let (field, the_rest) = contents[1 + maybe_field_start..].split_at(null_at);
        let the_rest = &the_rest[1..];
        match kind {
            1 if field.eq_ignore_ascii_case(b"appname") => {
                let name = the_rest.split(|&b| b == 0).next()?;
                let name = String::from_utf8_lossy(name).into_owned();
                if let Some(res) = factory.set_app_name(name) {
                    match res {
                        Ok(shortcut) => shortcuts.push(shortcut),
                        Err(_) => return None,
                    }
                }
            }
            1 if field.eq_ignore_ascii_case(b"exe") => {
                let exe = the_rest.split(|&b| b == 0).next()?;
                let exe = String::from_utf8_lossy(exe).into_owned();
                if let Some(res) = factory.set_executable(exe) {
                    match res {
                        Ok(shortcut) => shortcuts.push(shortcut),
                        Err(_) => return None,
                    }
                }
            }
            1 if field.eq_ignore_ascii_case(b"startdir") => {
                let dir = the_rest.split(|&b| b == 0).next()?;
                let dir = String::from_utf8_lossy(dir).into_owned();
                if let Some(res) = factory.set_start_dir(dir) {
                    match res {
                        Ok(shortcut) => shortcuts.push(shortcut),
                        Err(_) => return None,
                    }
                }
            }
            2 if field.eq_ignore_ascii_case(b"appid") => {
                let bytes = the_rest.get(..4)?.try_into().unwrap();
                let app_id = u32::from_le_bytes(bytes);
                if let Some(res) = factory.set_app_id(app_id) {
                    match res {
                        Ok(shortcut) => shortcuts.push(shortcut),
                        Err(_) => return None,
                    }
                }
            }
            _ => {}
        }
    }

    // ensure we don't have a partial shortcut lingering
    (factory == ShortcutFactory::default()).then_some(shortcuts)
}

struct TooMany;

type FieldResult = Option<std::result::Result<Shortcut, TooMany>>;

#[derive(Debug, Default, PartialEq, Eq)]
struct ShortcutFactory {
    app_id: Option<u32>,
    app_name: Option<String>,
    executable: Option<String>,
    start_dir: Option<String>,
}

impl ShortcutFactory {
    fn set_app_id(&mut self, id: u32) -> FieldResult {
        let was = self.app_id.replace(id);
        if was.is_some() {
            Some(Err(TooMany))
        } else {
            self.finish().map(Ok)
        }
    }

    fn set_app_name(&mut self, name: String) -> FieldResult {
        let was = self.app_name.replace(name);
        if was.is_some() {
            Some(Err(TooMany))
        } else {
            self.finish().map(Ok)
        }
    }

    fn set_executable(&mut self, exe: String) -> FieldResult {
        let was = self.executable.replace(exe);
        if was.is_some() {
            Some(Err(TooMany))
        } else {
            self.finish().map(Ok)
        }
    }

    fn set_start_dir(&mut self, dir: String) -> FieldResult {
        let was = self.start_dir.replace(dir);
        if was.is_some() {
            Some(Err(TooMany))
        } else {
            self.finish().map(Ok)
        }
    }

    fn finish(&mut self) -> Option<Shortcut> {
        match self {
            Self {
                app_id: Some(app_id),
                app_name: Some(app_name),
                executable: Some(executable),
                start_dir: Some(start_dir),
            } => {
                let shortcut = Shortcut {
                    app_id: *app_id,
                    app_name: app_name.to_owned(),
                    executable: executable.to_owned(),
                    start_dir: start_dir.to_owned(),
                };
                std::mem::take(self);
                Some(shortcut)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let contents = include_bytes!("../tests/assets/shortcuts.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        assert_eq!(
            shortcuts,
            vec![
                Shortcut {
                    app_id: 2786274309,
                    app_name: "Anki".into(),
                    executable: "\"anki\"".into(),
                    start_dir: "\"./\"".into(),
                },
                Shortcut {
                    app_id: 2492174738,
                    app_name: "LibreOffice Calc".into(),
                    executable: "\"libreoffice\"".into(),
                    start_dir: "\"./\"".into(),
                },
                Shortcut {
                    app_id: 3703025501,
                    app_name: "foo.sh".into(),
                    executable: "\"/usr/local/bin/foo.sh\"".into(),
                    start_dir: "\"/usr/local/bin/\"".into(),
                }
            ],
        );
        let steam_ids: Vec<_> = shortcuts
            .iter()
            .map(|shortcut| shortcut.steam_id())
            .collect();
        assert_eq!(
            steam_ids,
            [0xe89614fe02000000, 0xdb01c79902000000, 0x9d55017302000000,]
        );
    }

    /// Shortcut fields parse regardless of case
    #[test]
    fn different_case() {
        let contents = include_bytes!("../tests/assets/shortcuts_different_key_case.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        assert_eq!(
            shortcuts,
            vec![Shortcut {
                app_id: 2931025216,
                app_name: "Second Life".into(),
                executable: "\"/Applications/Second Life Viewer.app\"".into(),
                start_dir: "\"/Applications/\"".into(),
            }]
        );
    }

    /// Shortcuts fields can be in an arbitrary order
    #[test]
    fn different_order() {
        let contents = include_bytes!("../tests/assets/shortcuts_different_order.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        assert_eq!(
            shortcuts,
            vec![Shortcut {
                app_id: 2797129511,
                app_name: "The Wolf Among Us".into(),
                executable: "\"/opt/Heroic/heroic\"".into(),
                start_dir: "\"/home/spencer\"".into(),
            }]
        );
    }
}
