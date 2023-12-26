use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

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

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,      // Tracks the current column position
    color_code: ColorCode,       // Tracks the current color code
    buffer: &'static mut Buffer, // A mutable reference to the VGA buffer
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // If the byte is a newline character, call the new_line method
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    // If the current column position is out of bounds, call the new_line method
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1; // Set the row to the last row
                let col = self.column_position; // Set the column to the current column position

                let color_code = self.color_code; // Set the color code to the current color code
                self.buffer.chars[row][col].write(ScreenChar {
                    // Set the character at the current row and column to a new ScreenChar
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1; // Increment the column position
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read(); // Get the character at the current row and column
                self.buffer.chars[row - 1][col].write(char); // Set the character at the current row and column to the character at the previous row and column
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            // Iterate over each column in the row
            self.buffer.chars[row][col].write(ScreenChar {
                // Set the character at the current row and column to a new ScreenChar
                ascii_character: b' ',
                color_code: self.color_code,
            });
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            // Iterate over each byte in the string
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte), // If the byte is in the printable ASCII range or a newline character, write the byte
                _ => self.write_byte(0xfe),                   // Otherwise, write a ■ character
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Call the write_string method
        self.write_string(s);
        Ok(())
    }
}

#[cfg(target_arch = "x86_64")]
const VGA_BUFFER: usize = 0xb8000;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(VGA_BUFFER as *mut Buffer) },
    }); // Create a static Writer
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*))); // Define the print macro
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n")); // Define the println macro
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*))); // Define the println macro
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap(); // Lock the writer and write the arguments
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
