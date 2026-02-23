use core::fmt;
use core::ptr::{read_volatile, write_volatile};

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER: usize = 0xb8000;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

static mut CURSOR_COL: usize = 0;
static mut CURSOR_ROW: usize = 0;
static mut CURRENT_COLOR: ColorCode = ColorCode(0x0f); // 白 on 黒

fn vga_ptr() -> *mut ScreenChar {
    VGA_BUFFER as *mut ScreenChar
}

fn index(row: usize, col: usize) -> usize {
    row * BUFFER_WIDTH + col
}

fn put_char(row: usize, col: usize, ch: ScreenChar) {
    unsafe {
        let ptr = vga_ptr().add(index(row, col));
        write_volatile(ptr, ch);
    }
}

fn get_char(row: usize, col: usize) -> ScreenChar {
    unsafe {
        let ptr = vga_ptr().add(index(row, col));
        read_volatile(ptr)
    }
}

fn clear_row(row: usize) {
    let blank = ScreenChar {
        ascii_character: b' ',
        color_code: unsafe { CURRENT_COLOR },
    };
    for col in 0..BUFFER_WIDTH {
        put_char(row, col, blank);
    }
}

fn scroll_up() {
    for row in 1..BUFFER_HEIGHT {
        for col in 0..BUFFER_WIDTH {
            let ch = get_char(row, col);
            put_char(row - 1, col, ch);
        }
    }
    clear_row(BUFFER_HEIGHT - 1);
}

fn new_line() {
    unsafe {
        if CURSOR_ROW < BUFFER_HEIGHT - 1 {
            CURSOR_ROW += 1;
            CURSOR_COL = 0;
        } else {
            scroll_up();
            CURSOR_COL = 0;
        }
    }
}

fn write_byte(byte: u8) {
    unsafe {
        match byte {
            b'\n' => new_line(),
            byte => {
                if CURSOR_COL >= BUFFER_WIDTH {
                    new_line();
                }
                let row = CURSOR_ROW;
                let col = CURSOR_COL;
                let ch = ScreenChar {
                    ascii_character: byte,
                    color_code: CURRENT_COLOR,
                };
                put_char(row, col, ch);
                CURSOR_COL += 1;
            }
        }
    }
}

fn write_str_impl(s: &str) {
    for b in s.bytes() {
        match b {
            0x20..=0x7e | b'\n' => write_byte(b),
            _ => write_byte(0xfe),
        }
    }
}

pub fn init() {
    unsafe {
        CURRENT_COLOR = ColorCode::new(Color::White, Color::Black);
        CURSOR_COL = 0;
        CURSOR_ROW = 0;
    }
    for row in 0..BUFFER_HEIGHT {
        clear_row(row);
    }
}

struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str_impl(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut w = Writer;
    let _ = w.write_fmt(args); // エラーは握りつぶす（panic させない）
}
