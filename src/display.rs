use std::io::{self, Write};
use crossterm::{
    queue,
    terminal::{self, ClearType},
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor},
    cursor,
};

use crate::editor::{MicroHex, EditMode};

pub fn draw(editor: &mut MicroHex) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Use queue! to buffer commands without flushing
    queue!(stdout, cursor::MoveTo(0, 0))?;

    // Status line
    let file_size = editor.bytes.len();
    let percent = if file_size > 0 { 
        ((editor.cursor_pos + 1) as f64 / file_size as f64) * 100.0 
    } else { 
        0.0 
    };
    let mode_str = match editor.mode {
        EditMode::View => "VIEW",
        EditMode::EditHex => "EDIT HEX",
        EditMode::EditAscii => "EDIT ASCII",
    };
    writeln!(
        stdout,
        "File: {} ({} bytes) | {:.1}% | {} | ^E/Tab: mode | ^Q: quit | ^S: save | Backspace: Null\n",
        editor.filename, file_size, percent, mode_str
    )?;

    // Header
    queue!(stdout, SetForegroundColor(Color::AnsiValue(51)))?; // Bright cyan
    write!(stdout, "Offset    ")?;
    for i in 0..editor.bytes_per_line {
        if i == 8 { write!(stdout, " ")?; }
        write!(stdout, "{:02x} ", i)?;
    }
    writeln!(stdout, " ASCII")?;
    queue!(stdout, ResetColor)?; // Reset after header

    // Display lines
    let end_offset = (editor.offset + editor.bytes_per_line * editor.lines_per_page).min(editor.bytes.len());

    for line_start in (editor.offset..end_offset).step_by(editor.bytes_per_line) {
        write!(stdout, "{:08x}: ", line_start)?;

        let line_end = (line_start + editor.bytes_per_line).min(editor.bytes.len());
        let chunk = &editor.bytes[line_start..line_end];

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 { write!(stdout, " ")?; }
            let pos = line_start + j;
            let is_changed = editor.original_bytes.get(pos) != Some(byte);

            if pos == editor.cursor_pos {
                match editor.mode {
                    EditMode::EditHex => {
                        queue!(stdout, SetBackgroundColor(Color::AnsiValue(226)), SetForegroundColor(Color::AnsiValue(16)))?; // Yellow bg, black fg
                    }
                    EditMode::EditAscii => {
                        queue!(stdout, SetBackgroundColor(Color::AnsiValue(240)), SetForegroundColor(Color::AnsiValue(255)))?; // Grey bg, white fg
                    }
                    EditMode::View => {
                        queue!(stdout, SetBackgroundColor(Color::AnsiValue(15)), SetForegroundColor(Color::AnsiValue(16)))?; // White bg, black fg
                    }
                }
            }
            else if is_changed {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(208)))?; // Orange fg
            }
            else if *byte == 0 {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(242)))?; // Dark grey for zero
            } else if *byte < 0x20 || *byte >= 0x7f {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(33)))?; // Blue for control/non-printable
            } else if byte.is_ascii_graphic() || *byte == b' ' {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(34)))?; // Green for printable
            }

            write!(stdout, "{:02x}", byte)?;

            // Reset after each byte
            queue!(stdout, ResetColor)?;
            write!(stdout, " ")?;
        }

        // Padding
        for p in chunk.len()..16 {
            if p == 8 { write!(stdout, " ")?; }
            write!(stdout, "   ")?;
        }

        write!(stdout, " ")?;

        // ASCII
        for (j, byte) in chunk.iter().enumerate() {
            let pos = line_start + j;
            let c = if byte.is_ascii_graphic() || *byte == b' ' { *byte as char } else { '.' };
            let is_changed = editor.original_bytes.get(pos) != Some(byte);

            if pos == editor.cursor_pos {
                match editor.mode {
                    EditMode::EditAscii => {
                        queue!(stdout, SetBackgroundColor(Color::AnsiValue(226)), SetForegroundColor(Color::AnsiValue(16)))?; // Yellow bg, black fg
                    }
                    EditMode::EditHex => {
                        queue!(stdout, SetBackgroundColor(Color::AnsiValue(240)), SetForegroundColor(Color::AnsiValue(255)))?; // Grey bg, white fg
                    }
                    EditMode::View => {
                        queue!(stdout, SetBackgroundColor(Color::AnsiValue(15)), SetForegroundColor(Color::AnsiValue(16)))?; // White bg, black fg
                    }
                }
                write!(stdout, "{}", c)?;
                queue!(stdout, ResetColor)?;
            } else if is_changed {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(208)))?; // Orange fg
                write!(stdout, "{}", c)?;
                queue!(stdout, ResetColor)?;
            }
            else if *byte == 0 {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(242)))?; // Dark grey for zero
                write!(stdout, "{}", c)?;
                queue!(stdout, ResetColor)?;
            } else if *byte < 0x20 || *byte >= 0x7f {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(33)))?; // Blue for control/non-printable
                write!(stdout, "{}", c)?;
                queue!(stdout, ResetColor)?;
            } else if byte.is_ascii_graphic() || *byte == b' ' {
                queue!(stdout, SetForegroundColor(Color::AnsiValue(34)))?; // Green for printable
                write!(stdout, "{}", c)?;
                queue!(stdout, ResetColor)?;
            } else {
                write!(stdout, "{}", c)?;
            }
        }
        queue!(stdout, terminal::Clear(ClearType::UntilNewLine))?;
        writeln!(stdout)?;
    }
    
    queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;

    stdout.flush()?;
    Ok(())
} 