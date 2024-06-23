use std::{
    fmt::{self, Display},
    fs::{self, DirEntry, Metadata},
};

use crate::tabulate;

#[derive(Debug)]
struct EntryData {
    name: String,
    metadata: Metadata,
}

impl Display for EntryData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

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
                    name: file,
                    metadata: metadata,
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

fn list_dirs(dirs: &[EntryData]) {
    for (i, dir) in dirs.iter().enumerate() {
        if let Ok(entries) = fs::read_dir(&dir.name) {
            let multiple_dirs = dirs.len() > 1;
            let last_dir = i == dirs.len() - 1;

            if multiple_dirs {
                println!("{}:", dir.name);
            }
            list_dir_entries(entries);
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
        *entry.file_name().as_os_str().as_bytes().get(0).unwrap_or(&b' ') == b'.'
    }
    else {
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
            })
        })
        .collect::<Vec<EntryData>>()
}

fn list_dir_entries(dir: fs::ReadDir) {
    // iterate and consume the entries, getting metadata for each entry
    let mut entries = get_children(dir, false);

    // sort them by their name
    entries.sort_by(|a, b| icompare_name(a, b));
    if !entries.is_empty() {
        println!("{}", tabulate::Tabulator::new(&entries));
    }
}

fn list_files(entries: &[EntryData]) {
    for file in entries {
        println!("{}", file.name);
    }
    if entries.len() > 0 {
        println!();
    }
}

pub fn list(inputs: &InputFiles) {
    list_files(&inputs.files);
    list_dirs(&inputs.dirs);
}
