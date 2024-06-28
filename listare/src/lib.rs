use std::{
    fmt::{self, Display}, fs::{self, DirEntry, Metadata}, os::unix::fs::MetadataExt, path::PathBuf
};

pub mod posix;
mod tabulate;

use colored::{ColoredString, Colorize};
use tabulate::CharacterLength;

#[derive(Debug)]
pub struct Arguments {
    pub max_line_length: usize,
    pub paths: Vec<String>,
    pub list_dir_content: bool,
    pub show_hidden: bool,
    pub by_lines: bool,
    pub long_format: bool,
}

#[derive(Clone, Debug)]
struct EntryData {
    metadata: Metadata,
    path: PathBuf,
    name: String,
}

impl EntryData {
    fn from_path_str(path_str: &str) -> Result<Self, std::io::Error> {
        let metadata = fs::metadata(&path_str)?;
        let path = fs::canonicalize(&path_str)?;
        let name = path_str.to_string();
        Ok(EntryData {
            metadata,
            path,
            name,
        })
    }

    fn from_direntry(entry: DirEntry) -> Result<Self, std::io::Error> {
        let metadata = entry.metadata()?;
        let path = entry.path();
        let name = path
            .file_name()
            .ok_or(std::io::Error::from(std::io::ErrorKind::InvalidInput))?
            .to_string_lossy()
            .to_string();
        Ok(EntryData {
            metadata,
            path,
            name,
        })
    }

    fn colored_name(&self) -> ColoredString {
        if self.metadata.is_symlink() {
            let link_exists = fs::metadata(&self.path).is_ok();

            if link_exists {
                self.name.bold().cyan()
            } else {
                self.name.bold().red()
            }
        } else if self.metadata.is_dir() {
            self.name.bold().blue()
        } else {
            self.name.normal()
        }
    }
}

impl Display for EntryData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:width$}",
            self.colored_name(),
            width = f.width().unwrap_or(self.characters_long())
        )
    }
}

impl tabulate::CharacterLength for EntryData {
    fn characters_long(&self) -> usize {
        self.name.chars().count()
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    use std::os::unix::ffi::OsStrExt;
    if cfg!(target_os = "linux") {
        // if linux, check if the first byte is a period
        *entry
            .file_name()
            .as_os_str()
            .as_bytes()
            .get(0)
            .unwrap_or(&b' ')
            == b'.'
    } else {
        false
    }
}

fn get_children(dir: fs::ReadDir, include_hidden: bool) -> Vec<EntryData> {
    dir.into_iter()
        .filter_map(|e| {
            let entry = e.ok()?;
            if entry.file_name().is_empty() {
                eprintln!("Could not read file name of {:?}", entry);
                return None;
            }
            if !include_hidden && is_hidden(&entry) {
                // hidden file
                return None;
            }
            Some(EntryData::from_direntry(entry).ok()?)
        })
        .collect()
}

fn tabulate_entries(entries: &[EntryData], args: &Arguments) {
    if entries.is_empty() {
        return;
    }

    println!(
        "{}",
        tabulate::Tabulator::new(
            &entries,
            args.max_line_length,
            if args.by_lines {
                tabulate::TabulateOrientation::Rows
            } else {
                tabulate::TabulateOrientation::Columns
            }
        )
    );
}

fn longformat_tabulate_entries(entries: &[EntryData], _args: &Arguments) {
    for entry in entries {
        if entry.metadata.is_dir() {
            print!("d");
        } else {
            print!("-");
        }
        // print -rwx items for user, group, and other users
        for perm in &[
            (0o400, 'r'),
            (0o200, 'w'),
            (0o100, 'x'),
            (0o040, 'r'),
            (0o020, 'w'),
            (0o010, 'x'),
            (0o004, 'r'),
            (0o002, 'w'),
            (0o001, 'x'),
        ] {
            if entry.metadata.mode() & perm.0 != 0 {
                print!("{}", perm.1);
            } else {
                print!("-");
            }
        }
        

        let links = entry.metadata.nlink();
        let user = users::get_user_by_uid(entry.metadata.uid()).map(|u| u.name().to_string_lossy().to_string()).unwrap_or_default();
        let group = users::get_group_by_gid(entry.metadata.gid()).map(|g| g.name().to_string_lossy().to_string()).unwrap_or_default();
        let size = if entry.metadata.is_dir() { 0 } else { entry.metadata.len() };  // TODO: should have a value for dirs
        let name = entry.colored_name();
        
        let modified = entry.metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
        let modified = modified.map(|t| chrono::DateTime::from_timestamp(t.as_secs() as i64, 0)).expect("Could not get modified time");
        let modified = modified.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default();

        println!(". {} {} {} {} {} {}", links, user, group, size, modified, name);
    }
}

fn list_entries(mut entries: Vec<EntryData>, args: &Arguments) {
    entries.sort_by(|a, b| posix::strcoll(&a.name, &b.name));

    if args.long_format {
        longformat_tabulate_entries(&entries, args);
    } else {
        tabulate_entries(&entries, args);
    }
}

fn list_dirs(dirs: &[EntryData], args: &Arguments, headings: bool) -> Result<(), ListareError> {
    for (i, dir) in dirs.iter().enumerate() {
        if let Ok(dir_iter) = fs::read_dir(&dir.path) {
            if headings {
                println!("{}:", dir.name);
            }

            list_entries(get_children(dir_iter, args.show_hidden), args);

            if i != dirs.len() - 1 {
                println!();
            }
        } else {
            eprintln!("Could not read directory: {}", &dir.name);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum ListareError {
    Unknown,
    Generic(String),
}

impl std::error::Error for ListareError {}
impl fmt::Display for ListareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ListareError::Unknown => write!(f, "An unknown error occurred"),
            ListareError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

fn split_files_dirs(paths: &[String]) -> (Vec<EntryData>, Vec<EntryData>) {
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for path in paths {
        if let Ok(entry) = EntryData::from_path_str(path) {
            if entry.metadata.is_dir() {
                dirs.push(entry);
            } else {
                files.push(entry);
            }
        }
    }

    (files, dirs)
}

pub fn run(args: &Arguments) -> Result<(), ListareError> {
    if args.list_dir_content {
        let (files, dirs) = split_files_dirs(&args.paths);
        let had_files = !files.is_empty();

        if had_files {
            list_entries(files, args);
        }

        if !dirs.is_empty() {
            if had_files {
                println!();
            }

            let headings: bool = had_files || (dirs.len() > 1);
            list_dirs(&dirs, args, headings)?;
        }
    } else {
        let entries = args
            .paths
            .iter()
            .filter_map(|path| EntryData::from_path_str(path).ok())
            .collect();
        list_entries(entries, args);
    }

    Ok(())
}
