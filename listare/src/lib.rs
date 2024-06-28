use std::{
    fmt::{self, Display},
    fs::{self, DirEntry, Metadata},
    path::PathBuf,
};

pub mod posix;
mod tabulate;

use colored::{ColoredString, Colorize};
use tabulate::CharacterLength;

#[derive(Debug)]
pub struct Arguments {
    pub max_line_length: usize,
    pub paths: Vec<String>,
    pub show_hidden: bool,
    pub by_lines: bool,
}

#[derive(Clone, Debug)]
struct EntryData {
    metadata: Metadata,
    path: PathBuf,
    name: String,
}

impl EntryData {
    fn from_path_str(path: &str) -> Result<Self, std::io::Error> {
        let metadata = fs::metadata(&path)?;
        let path = PathBuf::from(path);
        let name = path
            .file_name()
            .ok_or(std::io::Error::from(std::io::ErrorKind::InvalidInput))?
            .to_str()
            .ok_or(std::io::Error::from(std::io::ErrorKind::InvalidInput))?
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
            .to_str()
            .ok_or(std::io::Error::from(std::io::ErrorKind::InvalidInput))?
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

    // based on the type of view, display them (tabulate only for now)
    tabulate_entries(&entries, args);
}

fn list_dirs(args: &Arguments, headings: bool) -> Result<(), ListareError> {
    for (i, dir) in args.inputs.dirs.iter().enumerate() {
        if let Ok(dir_iter) = fs::read_dir(&dir.name) {
            if headings {
                println!("{}:", dir.name);
            }

            list_entries(get_children(dir_iter, args.show_hidden), args);

            if i != args.inputs.dirs.len() - 1 {
                println!();
            }
        } else {
            eprintln!("Could not read directory: {}", &dir.name);
        }
    }
    Ok(())
}

pub enum ListareError {
    Unknown,
    Generic(String),
}

pub fn run(args: &Arguments) -> Result<(), ListareError> {
    if !args.inputs.files.is_empty() {
        list_entries(args.inputs.files.clone(), args);
    }

    if !args.inputs.dirs.is_empty() {
        // show headings when there are multiple dirs or files and one or more dirs
        let had_files = !args.inputs.files.is_empty();

        if had_files {
            println!();
        }

        let headings: bool = had_files || (args.inputs.dirs.len() > 1);
        list_dirs(args, headings)?;
    }

    Ok(())
}
