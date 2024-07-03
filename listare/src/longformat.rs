use crate::{Arguments, EntryData};
use std::fmt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::time::SystemTime;


struct Config {
    size_width: usize,
    user_width: usize,
    group_width: usize,
    nlinks_width: usize,
}

#[allow(dead_code)]
struct EntryDisplayer<'a> {
    entry: &'a EntryData,
    arguments: &'a Arguments,
    config: &'a Config,
}

impl<'a> EntryDisplayer<'a> {
    //! Display long format details for an entry
    //! https://www.gnu.org/software/coreutils/manual/html_node/What-information-is-listed.html
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

    fn write_file_mode(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

        for perm in perms.iter() {
            if mode & perm.0 != 0 {
                write!(f, "{}", perm.1)?;
            } else {
                write!(f, "-")?;
            }
        }

        Ok(())
    }

    fn write_nlinks(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // right align the nlinks using the config width
        write!(f, "{:width$}", self.entry.metadata.nlink(), width = self.config.nlinks_width)
    }
    
    fn write_user(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // left align the user
        let user = users::get_user_by_uid(self.entry.metadata.uid())
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_default();
        write!(f, "{:width$}", user, width = self.config.user_width)
    }
    
    fn write_group(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let group = users::get_group_by_gid(self.entry.metadata.gid())
            .map(|g| g.name().to_string_lossy().to_string())
            .unwrap_or_default();
        write!(f, "{:width$}", group, width = self.config.group_width)
    }
    
    fn write_size(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size = if self.entry.metadata.is_dir() {
            0
        } else {
            self.entry.metadata.len()
        };
        write!(f, "{:width$}", size, width = self.config.size_width)
    }

    fn write_timestamp(&self, f: &mut fmt::Formatter, timestamp: &std::time::SystemTime) -> fmt::Result {
        // a timestamp is considered recent if it is less than 6 months old, and is not dated in the future
        let now = SystemTime::now();
        let six_months = 60 * 60 * 24 * 30 * 6;
        let is_recent = now.duration_since(*timestamp).unwrap().as_secs() < six_months;
        let durn = timestamp.duration_since(SystemTime::UNIX_EPOCH).expect("Could not get duration");
        let dt = chrono::DateTime::from_timestamp(durn.as_secs() as i64, 0).expect("Could not create datetime");
        let dt = dt.with_timezone(&chrono::Local);
        
        if is_recent {
            write!(f, "{}", dt.format("%b %e %H:%M"))
        } else {
            write!(f, "{}", dt.format("%b %e  %Y"))
        }
    }

    fn write_modified(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_timestamp(f, &self.entry.metadata.modified().expect("Coult not get modified time"))
    }

    fn get_link_target(&self) -> Result<EntryData, std::io::Error> {
        let link = std::fs::read_link(&self.entry.path)?;
        if link.is_absolute() {
            EntryData::from_path(link)
        }
        else {
            let parent = self.entry.path.parent().ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))?;
            EntryData::from_relative_path(parent, link)
        }
    }
    
    fn write_name(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // get the colored name of the entry
        let name = self.entry.colored_name();
        // if the entry is a symlink use a format of "name -> target"
        // otherwise, just print the name
        if self.entry.metadata.file_type().is_symlink() {
            let target = self.get_link_target().map(|e| e.colored_path()).map_err(|_| fmt::Error)?;
            write!(f, "{} -> {}", name, target)
        } else {
            write!(f, "{}", name)
        }
    }
    
}

impl<'a> fmt::Display for EntryDisplayer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_file_type(f)?;
        self.write_file_mode(f)?;
        write!(f, " ")?;
        self.write_nlinks(f)?;
        write!(f, " ")?;
        self.write_user(f)?;
        write!(f, " ")?;
        self.write_group(f)?;
        write!(f, " ")?;
        self.write_size(f)?;
        write!(f, " ")?;
        self.write_modified(f)?;
        write!(f, " ")?;
        self.write_name(f)?;
        Ok(())
    }
}

pub fn longformat_tabulate_entries(entries: &[EntryData], _args: &Arguments) {
    let mut cfg = Config {
        size_width: 1,
        user_width: 1,
        group_width: 1,
        nlinks_width: 1,
    };

    // go through the etries and find the max width for each field
    for entry in entries {
        cfg.size_width = cfg.size_width.max(entry.metadata.len().to_string().len());
        // todo USER AND GROUP is slow - extract this
        cfg.user_width = cfg.user_width.max(
            users::get_user_by_uid(entry.metadata.uid())
                .map(|u| u.name().len())
                .unwrap_or_default(),
        );
        cfg.group_width = cfg.group_width.max(
            users::get_group_by_gid(entry.metadata.gid())
                .map(|g| g.name().len())
                .unwrap_or_default(),
        );
        cfg.nlinks_width = cfg.nlinks_width.max(entry.metadata.nlink().to_string().len());
    }

    for entry in entries {
        println!(
            "{}",
            EntryDisplayer {
                entry,
                arguments: _args,
                config: &cfg
            }
        );
    }
}
