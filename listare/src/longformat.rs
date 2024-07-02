use crate::{Arguments, EntryData};
use std::fmt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};

struct EntryDisplayer<'a> {
    entry: &'a EntryData,
    arguments: &'a Arguments,
}

impl<'a> EntryDisplayer<'a> {
    fn write_file_type(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ft = self.entry.metadata.file_type();
        write!(
            f,
            "{}",
            if ft.is_dir() {
                'd'
            } else if ft.is_symlink() {
                'l'
            } else if ft.is_char_device() {
                'c'
            } else if ft.is_block_device() {
                'b'
            } else if ft.is_fifo() {
                'p'
            } else if ft.is_socket() {
                's'
            } else if ft.is_file() {
                '-'
            } else {
                '?'
            }
        )
    }

    fn write_perms(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mode = self.entry.metadata.mode();
        let perms = [
            (0o400, 'r'),
            (0o200, 'w'),
            (0o100, 'x'),
            (0o040, 'r'),
            (0o020, 'w'),
            (0o010, 'x'),
            (0o004, 'r'),
            (0o002, 'w'),
            (0o001, 'x'),
        ];

        for (i, perm) in perms.iter().enumerate() {
            if mode & perm.0 != 0 {
                write!(f, "{}", perm.1)?;
            } else {
                write!(f, "-")?;
            }

            if (i + 1) % 3 == 0 {
                write!(f, " ")?;
            }
        }

        Ok(())
    }
}

impl<'a> fmt::Display for EntryDisplayer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_file_type(f)?;
        self.write_perms(f)
    }
}

pub fn longformat_tabulate_entries(entries: &[EntryData], _args: &Arguments) {
    for entry in entries {
        println!(
            "{}",
            EntryDisplayer {
                entry,
                arguments: _args
            }
        );
    }
    // for entry in entries {
    //     if entry.metadata.is_dir() {
    //         print!("d");
    //     } else {
    //         print!("-");
    //     }
    //     // print -rwx items for user, group, and other users
    //     for perm in &[
    //         (0o400, 'r'),
    //         (0o200, 'w'),
    //         (0o100, 'x'),
    //         (0o040, 'r'),
    //         (0o020, 'w'),
    //         (0o010, 'x'),
    //         (0o004, 'r'),
    //         (0o002, 'w'),
    //         (0o001, 'x'),
    //     ] {
    //         if entry.metadata.mode() & perm.0 != 0 {
    //             print!("{}", perm.1);
    //         } else {
    //             print!("-");
    //         }
    //     }

    //     let links = entry.metadata.nlink();
    //     let user = users::get_user_by_uid(entry.metadata.uid())
    //         .map(|u| u.name().to_string_lossy().to_string())
    //         .unwrap_or_default();
    //     let group = users::get_group_by_gid(entry.metadata.gid())
    //         .map(|g| g.name().to_string_lossy().to_string())
    //         .unwrap_or_default();
    //     let size = if entry.metadata.is_dir() {
    //         0
    //     } else {
    //         entry.metadata.len()
    //     }; // TODO: should have a value for dirs
    //     let name = entry.colored_name();

    //     let modified = entry
    //         .metadata
    //         .modified()
    //         .ok()
    //         .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
    //     let modified = modified
    //         .map(|t| chrono::DateTime::from_timestamp(t.as_secs() as i64, 0))
    //         .expect("Could not get modified time");
    //     let modified = modified
    //         .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
    //         .unwrap_or_default();

    //     println!(
    //         ". {} {} {} {} {} {}",
    //         links, user, group, size, modified, name
    //     );
}
