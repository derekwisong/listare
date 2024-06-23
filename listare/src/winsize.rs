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
