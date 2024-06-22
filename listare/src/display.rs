use std::{
    fmt::{self, Display},
    fs::{self, Metadata},
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
    for dir in dirs {
        if let Ok(entries) = fs::read_dir(&dir.name) {
            // println!("{}:", &dir.name);
            list_dir_entries(entries);
            // println!();
        } else {
            eprintln!("Could not read directory: {}", &dir.name);
        }
    }
}

fn icompare_name(left: &EntryData, right: &EntryData) -> std::cmp::Ordering {
    left.name.to_lowercase().cmp(&right.name.to_lowercase())
}

fn list_dir_entries(entries: fs::ReadDir) {
    // iterate and consume the entries, getting metadata for each entry
    let mut details = entries
        .into_iter()
        .filter_map(|e| {
            let entry = e.ok()?;
            if entry.file_name().is_empty() {
                eprintln!("Could not read file name");
                return None;
            }
            if entry.file_name().to_str()?.starts_with(".") {
                // hidden file
                return None;
            }
            Some(EntryData {
                name: entry.file_name().to_string_lossy().to_string(),
                metadata: entry.metadata().ok()?,
            })
        })
        .collect::<Vec<EntryData>>();

    // sort them by their name
    details.sort_by(|a, b| icompare_name(a, b));
    if !details.is_empty() {
        println!("{}", tabulate::Tabulator::new(&details));
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
