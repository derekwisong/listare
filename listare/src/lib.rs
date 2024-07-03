use std::{
    fmt::{self, Display}, fs::{self, DirEntry, Metadata}, path::{self, PathBuf}
};

pub mod posix;
mod tabulate;
mod longformat;

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
        let path = path::PathBuf::from(path_str);
        let metadata = fs::symlink_metadata(&path)?;
        Ok(EntryData {
            metadata,
            path,
            name: path_str.to_string(),
        })
    }

    fn from_relative_path(root: &path::Path, relpath: path::PathBuf) -> Result<Self, std::io::Error> {
        let name = relpath
            .file_name()
            .ok_or(std::io::Error::from(std::io::ErrorKind::InvalidInput))?
            .to_string_lossy()
            .to_string();
        let abspath = root.join(&relpath);
        Ok(EntryData {
            metadata: fs::symlink_metadata(&abspath)?,
            path: relpath,
            name: name,
        })
    }

    fn from_path(path: path::PathBuf) -> Result<Self, std::io::Error> {
        let metadata = fs::symlink_metadata(&path)?;
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
        self.colored(&self.name)
    }

    fn colored_path(&self) -> ColoredString {
        self.colored(&self.path.to_string_lossy())
    }

    fn colored(&self, text: &str) -> ColoredString {
        if self.metadata.is_symlink() {
            let link_exists = fs::metadata(&self.path).is_ok();

            if link_exists {
                text.bold().cyan()
            } else {
                text.bold().red()
            }
        } else if self.metadata.is_dir() {
            text.bold().blue()
        } else {
            text.normal()
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



fn list_entries(mut entries: Vec<EntryData>, args: &Arguments) {
    entries.sort_by(|a, b| posix::strcoll(&a.name, &b.name));

    if args.long_format {
        longformat::longformat_tabulate_entries(&entries, args);
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
