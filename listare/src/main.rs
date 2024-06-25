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
        by_lines: matches.get_flag("bylines"),
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
                .default_value(".")
                .num_args(1..),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Show hidden files (do not ignore entries starting with .)"),
        )
        .arg(
            Arg::new("bylines")
                .short('x')
                .action(ArgAction::SetTrue)
                .help("List entries by lines instead of by columns")
        );

    let args = parse_arguments(cmd);

    match listare::run(&args) {
        Ok(()) => {}, // do nothing on success
        Err(listare::ListareError::Generic(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        },
        Err(listare::ListareError::Unknown) => {
            eprintln!("An unknown error occurred");
            std::process::exit(1);
        },
    };
}
