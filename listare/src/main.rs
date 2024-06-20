mod display;

use clap::{Arg, ArgAction, Command};
use display::InputFiles;

fn main() {
    let matches = Command::new("listare")
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
        .get_matches();

    let files = InputFiles::from_args(matches.get_many("files").unwrap().cloned().collect());
    display::list(&files);
}
