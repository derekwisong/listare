use std::fs::{self, Metadata};

#[derive(Debug)]
struct DirType {
    entry: String,
    metadata: Metadata,
}

#[derive(Debug)]
struct FileType {
    entry: String,
    metadata: Metadata,
}

pub struct InputFiles {
    pub files: Vec<FileType>,
    pub dirs: Vec<DirType>,
}

impl From<Vec<String>> for InputFiles {
    fn from(value: Vec<String>) -> Self {
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for file in value {
            if let Ok(metadata) = fs::metadata(&file) {
                if metadata.is_dir() {
                    dirs.push(DirType { entry: file , metadata: metadata });
                } else {
                    files.push(FileType { entry: file, metadata: metadata });
                }
            }
            else {
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

fn list_dirs(dirs: &[DirType]) {
    for dir in dirs {
        if let Ok(entries) = fs::read_dir(&dir.entry) {
            println!("{}:", dir.entry);
            list_dir_entries(entries);
            println!();
        }
        else {
            eprintln!("Could not read directory: {}", dir.entry);
        }
    }

}

fn get_line_length() -> usize {
    // get the environment variable COLUMNS
    // if it is not greater-than 0, return 80
    // otherwise, return the value of COLUMNS
    let default: usize = 80;

    if let Ok(val) = std::env::var("COLUMNS") {
        if let Ok(num) = val.parse::<usize>() {
            if num > 0 {
                return num;
            }
        }
    }
    default
}

fn list_dir_entries(entries: fs::ReadDir) {
    let line_length = get_line_length();
    let min_column_width: usize = 3;  // 1 char for name 2 separating white space
    let max_columns = std::cmp::max(1, line_length / min_column_width);

    for entry in entries.filter_map(|e| e.ok()) {
        let display = match entry.file_type() {
            Ok(ft) => {
                if ft.is_dir() {
                    format!("{}/", entry.file_name().to_string_lossy())
                } 
                else {
                    entry.file_name().to_string_lossy().to_string()
                }
            
            },
            Err(_) => continue,
        };
        println!("{}",  display);
    }
}

fn list_files(files: &[FileType]) {
    for file in files {
        println!("{}", file.entry);
    }
    if files.len() > 0 {
        println!();
    }
}

pub fn list(inputs: &InputFiles) {
    list_files(&inputs.files);
    list_dirs(&inputs.dirs);
}