use libc;

#[repr(C)]
#[derive(Debug)]
struct WinSizeInternal {
    ws_row: libc::c_ushort,    /* rows, in characters */
    ws_col: libc::c_ushort,    /* columns, in characters */
    ws_xpixel: libc::c_ushort, /* horizontal size, pixels */
    ws_ypixel: libc::c_ushort, /* vertical size, pixels */
}

#[derive(Debug)]
pub struct WinSize {
    pub rows: usize,
    pub cols: usize,
}

pub fn get_winsize() -> Option<WinSize> {
    let w = WinSizeInternal {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    match unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &w) } {
        0 => {
            if w.ws_col > 0 {
                Some(WinSize {
                    rows: w.ws_row as usize,
                    cols: w.ws_col as usize,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn strcoll(a: &str, b: &str) -> std::cmp::Ordering {
    let result = unsafe {
        libc::strcoll(
            a.as_ptr() as *const libc::c_char,
            b.as_ptr() as *const libc::c_char,
        )
    };

    if result < 0 {
        std::cmp::Ordering::Less
    } else if result > 0 {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

#[derive(Debug)]
pub enum LocaleError {
    NullByte,        // the provided input locale contains a null byte
    LocaleError,     // call to setlocale failed
    ConversionError, // error converting setlocale result to str
}

impl std::fmt::Display for LocaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LocaleError::NullByte => write!(f, "Input contains a null byte"),
            LocaleError::LocaleError => write!(f, "Could not set locale"),
            LocaleError::ConversionError => write!(f, "Could not convert locale to string"),
        }
    }
}

impl std::error::Error for LocaleError {}

pub enum Locale<'a> {
    UserPreferred,
    Named(&'a str),
}

pub fn setlocale(locale: Locale) -> Result<&str, LocaleError> {
    let locale = match locale {
        Locale::UserPreferred => "",
        Locale::Named(locale) => locale,
    };
    match std::ffi::CString::new(locale) {
        Err(_) => Err(LocaleError::NullByte),
        Ok(locale) => unsafe {
            let result = libc::setlocale(libc::LC_ALL, locale.as_ptr());
            if result.is_null() {
                Err(LocaleError::LocaleError)
            } else {
                let result_str = std::ffi::CStr::from_ptr(result);
                match result_str.to_str() {
                    Err(_) => Err(LocaleError::ConversionError),
                    Ok(result) => Ok(result),
                }
            }
        },
    }
}
