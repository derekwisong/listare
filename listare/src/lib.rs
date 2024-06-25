use std::{
    fmt::{self, Display},
    fs::{self, DirEntry, Metadata},
    path::PathBuf,
};

mod tabulate;
use colored::{ColoredString, Colorize};

#[derive(Debug)]
pub struct Arguments {
    pub max_line_length: usize,
    pub inputs: InputFiles,
    pub show_hidden: bool,
}

#[derive(Debug)]
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
            width = f.width().unwrap_or(self.name.chars().count())
        )
    }
}

impl tabulate::CharacterLength for EntryData {
    fn characters_long(&self) -> usize {
        self.name.chars().count()
    }
}

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

fn list_dirs(args: &Arguments) {
    for (i, dir) in args.inputs.dirs.iter().enumerate() {
        if let Ok(entries) = fs::read_dir(&dir.name) {
            let multiple_dirs = args.inputs.dirs.len() > 1;
            let last_dir = i == args.inputs.dirs.len() - 1;

            if multiple_dirs {
                println!("{}:", dir.name);
            }
            list_dir_entries(entries, args);
            // if more than one and not the last directory, print a newline
            if multiple_dirs && !last_dir {
                println!();
            }
        } else {
            eprintln!("Could not read directory: {}", &dir.name);
        }
    }
}

fn icompare_name(left: &EntryData, right: &EntryData) -> std::cmp::Ordering {
    left.name.to_lowercase().cmp(&right.name.to_lowercase())
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

fn list_dir_entries(dir: fs::ReadDir, args: &Arguments) {
    // iterate and consume the entries, getting metadata for each entry
    let mut entries = get_children(dir, args.show_hidden);

    // sort them by their name
    entries.sort_by(|a, b| icompare_name(a, b));
    if !entries.is_empty() {
        println!(
            "{}",
            tabulate::Tabulator::new(&entries, args.max_line_length)
        );
    }
}

fn list_files(args: &Arguments) {
    let entries = &args.inputs.files;
    for file in entries {
        println!("{}", file.name);
    }
    if entries.len() > 0 {
        println!();
    }
}

pub fn list(args: &Arguments) {
    list_files(args);
    list_dirs(args);
}
