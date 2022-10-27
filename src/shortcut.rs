//! **WARN:** This is all hacky and should be replaced with proper binary VDF parsing

use std::{fs, iter::Peekable, path::Path, slice::Iter};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Shortcut {
    pub appid: u32,
    pub app_name: String,
    pub executable: String,
    pub start_dir: String,
}

#[cfg(feature = "shortcuts_extras")]
impl Shortcut {
    pub fn steam_id(&self) -> u64 {
        let algorithm = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

        let mut digest = algorithm.digest();
        digest.update(self.executable.as_bytes());
        digest.update(self.app_name.as_bytes());

        let top = digest.finalize() | 0x80000000;
        ((top as u64) << 32) | 0x02000000
    }
}

/// Discovers any shorcuts stored within `userdata`
pub fn discover_shortcuts(steam_dir: &Path) -> Vec<Shortcut> {
    fn inner(steam_dir: &Path) -> Option<Vec<Shortcut>> {
        let mut shortcuts = Vec::new();

        // Find and parse each `userdata/<user_id>/config/shortcuts.vdf` file
        let user_data = steam_dir.join("userdata");
        for entry in fs::read_dir(&user_data).ok()?.filter_map(|e| e.ok()) {
            let shortcuts_path = entry.path().join("config").join("shortcuts.vdf");
            if !shortcuts_path.is_file() {
                continue;
            }

            if let Ok(contents) = fs::read(&shortcuts_path) {
                if let Some(parsed) = parse_shortcuts(&contents) {
                    shortcuts.extend(parsed);
                }
            }
        }

        Some(shortcuts)
    }

    inner(steam_dir).unwrap_or_default()
}

/// Advances `it` until right after the matching `needle`
///
/// Only works if the starting byte is not used anywhere else in the needle. This works well when
/// finding keys since the starting byte indicates the type and wouldn't be used in the key
#[must_use]
fn after_many(it: &mut Peekable<Iter<u8>>, needle: &[u8]) -> bool {
    loop {
        loop {
            let mut needle_it = needle.iter();
            let b = match it.next() {
                Some(b) => b,
                None => return false,
            };

            if Some(b) == needle_it.next() {
                loop {
                    if needle_it.len() == 0 {
                        return true;
                    }

                    let b = it.peek();
                    let needle_b = needle_it.next();
                    if b == needle_b.as_ref() {
                        let _ = it.next();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

fn parse_value_str(it: &mut Peekable<Iter<u8>>) -> Option<String> {
    let mut buff = Vec::new();
    loop {
        let b = it.next()?;
        if *b == 0x00 {
            break Some(String::from_utf8_lossy(&buff).into_owned());
        }

        buff.push(*b);
    }
}

fn parse_value_u32(it: &mut Peekable<Iter<u8>>) -> Option<u32> {
    let bytes = [*it.next()?, *it.next()?, *it.next()?, *it.next()?];
    Some(u32::from_le_bytes(bytes))
}

// The performance of this is likely terrible, but also the files we're parsing are tiny so it
// won't matter
fn parse_shortcuts(contents: &[u8]) -> Option<Vec<Shortcut>> {
    let mut it = contents.iter().peekable();
    let mut shortcuts = Vec::new();

    loop {
        if !after_many(&mut it, b"\x02appid\x00") {
            return Some(shortcuts);
        }
        let appid = parse_value_u32(&mut it)?;

        if !after_many(&mut it, b"\x01AppName\x00") {
            return None;
        }
        let app_name = parse_value_str(&mut it)?;

        if !after_many(&mut it, b"\x01Exe\x00") {
            return None;
        }
        let executable = parse_value_str(&mut it)?;

        if !after_many(&mut it, b"\x01StartDir\x00") {
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
    }

    #[cfg(feature = "shortcuts_extras")]
    #[test]
    fn shortcuts_extras() {
        let contents = include_bytes!("../tests/sample_data/shortcuts.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        let ideal_ids = vec![0xe89614fe02000000, 0xdb01c79902000000, 0x9d55017302000000];
        for (id, shortcut) in ideal_ids.into_iter().zip(shortcuts.iter()) {
            assert_eq!(id, shortcut.steam_id());
        }
    }
}
