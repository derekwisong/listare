use std::{
    fmt::{self, Display},
    fs::{self, DirEntry, Metadata},
    path::PathBuf,
};

mod tabulate;
pub mod posix;

use colored::{ColoredString, Colorize};
use tabulate::CharacterLength;

#[derive(Debug)]
pub struct Arguments {
    pub max_line_length: usize,
    pub inputs: InputFiles,
    pub show_hidden: bool,
    pub by_lines: bool,
}

#[derive(Clone, Debug)]
struct EntryData {
    name: String,
    metadata: Metadata,
    path: PathBuf,
}

impl EntryData {
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

/// The files and directories that the user passed in as cli arguments
#[derive(Debug)]
pub struct InputFiles {
    files: Vec<EntryData>,
    dirs: Vec<EntryData>,
}

impl From<Vec<String>> for InputFiles {
    fn from(value: Vec<String>) -> Self {
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for file in value {
            if let Ok(metadata) = fs::metadata(&file) {
                let entry = EntryData {
                    name: file.clone(),
                    metadata: metadata,
                    path: PathBuf::from(file),
                };
                if entry.metadata.is_dir() {
                    dirs.push(entry);
                } else {
                    files.push(entry);
                }
            } else {
                eprintln!("Could not read metadata for file: {}", file);
            }
        }

        InputFiles { files, dirs }
    }
}

impl InputFiles {
    pub fn from_args(files: Vec<String>) -> Self {
        InputFiles::from(files)
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
            Some(EntryData {
                name: entry.file_name().to_string_lossy().to_string(),
                metadata: entry.metadata().ok()?,
                path: entry.path(),
            })
        })
        .collect::<Vec<EntryData>>()
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
        if let Ok(entries) = fs::read_dir(&dir.name) {
            if headings {
                println!("{}:", dir.name);
            }

            list_entries(get_children(entries, args.show_hidden), args);

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
