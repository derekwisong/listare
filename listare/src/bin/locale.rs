// testing the locale functions

use listare::posix::{setlocale, Locale};

fn main() {
    let locales = [
        "",
        "C",
        "en_US.UTF-8",
        "fr_FR.UTF-8",
        "de_DE.UTF-8",
        "ja_JP.UTF-8",
    ];

    for locale in &locales {
        match setlocale(Locale::Named(locale)) {
            Ok(current_locale) => {
                println!("setlocale('{}') -> '{}'", locale, current_locale);
            }
            Err(error) => {
                eprintln!("Error setting locale to '{}': {}", locale, error);
            }
        }
    }
}
