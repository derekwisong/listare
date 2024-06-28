// mod posix;
use clap::{Arg, ArgAction, Command};
use listare;

fn get_terminal_width() -> Option<usize> {
    if let Some(winsize) = listare::posix::get_winsize() {
        Some(winsize.cols)
    } else if let Ok(val) = std::env::var("COLUMNS") {
        if let Ok(num) = val.parse::<usize>() {
            if num > 0 {
                Some(num)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn build_command() -> Command {
    Command::new("listare")
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
                .help("List entries by lines instead of by columns"),
        )
}

fn parse_args() -> listare::Arguments {
    let command = build_command();
    let matches = command.get_matches();

    listare::Arguments {
        max_line_length: get_terminal_width().unwrap_or(80),
        paths: matches.get_many("files").unwrap().cloned().collect(),
        // inputs: listare::InputFiles::from_args(
        //     matches.get_many("files").unwrap().cloned().collect(),
        // ),
        show_hidden: matches.get_flag("all"),
        by_lines: matches.get_flag("bylines"),
    }
}

fn main() {
    let args = parse_args();

    // sorting by name is done with strcoll, which is locale-aware
    let _ = listare::posix::setlocale(listare::posix::Locale::UserPreferred);

    match listare::run(&args) {
        Err(listare::ListareError::Generic(msg)) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
        Err(listare::ListareError::Unknown) => {
            eprintln!("An unknown error occurred");
            std::process::exit(1);
        },
        Ok(_) => {}
    };
}
