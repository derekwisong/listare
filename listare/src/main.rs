mod winsize;
use clap::{Arg, ArgAction, Command};

use listare;

fn get_line_length() -> usize {
    const DEFAULT: usize = 80;

    // first try using the ioctl system call
    if let Some(winsize) = crate::winsize::get_winsize() {
        return winsize.cols;
    }

    // if that fails, try using the COLUMNS environment variable
    if let Ok(val) = std::env::var("COLUMNS") {
        if let Ok(num) = val.parse::<usize>() {
            if num > 0 {
                return num;
            }
        }
    }

    // if all else fails, return the default value
    DEFAULT
}

fn parse_arguments(command: Command) -> listare::Arguments {
    let matches = command.get_matches();
    listare::Arguments {
        max_line_length: get_line_length(),
        inputs: listare::InputFiles::from_args(matches.get_many("files").unwrap().cloned().collect()),
        show_hidden: matches.get_flag("all"),
    }
}

fn main() {
    let cmd = Command::new("listare")
        .version("0.1.0")
        .author("Derek Wisong <derekwisong@gmail.com>")
        .about("My version of `ls`")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("The file(s) to list information about")
                .default_value("/etc")
                .num_args(1..),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Show hidden files (do not ignore entries starting with .)"),
        );
    let args = parse_arguments(cmd);
    listare::list(&args);
}
