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
    queue!(stdout, cursor::MoveTo(0, 0))?;

    draw_status_line(&mut stdout, editor)?;
    draw_header(&mut stdout, editor.bytes_per_line)?;

    let end_offset = (editor.offset + editor.bytes_per_line * editor.lines_per_page).min(editor.bytes.len());

    for line_start in (editor.offset..end_offset).step_by(editor.bytes_per_line) {
        draw_line(&mut stdout, editor, line_start)?;
    }
    
    queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;
    stdout.flush()?;
    Ok(())
} 

// This function is generic over any writer that implements std::io::Write (such as Stdout, a file, or a buffer).
// Using W: Write allows us to reuse this function for testing, alternate outputs, or redirection if needed.
fn draw_status_line<W: Write>(stdout: &mut W, editor: &MicroHex) -> io::Result<()> {
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
    )
}

fn draw_header<W: Write>(stdout: &mut W, bytes_per_line: usize) -> io::Result<()> {
    queue!(stdout, SetForegroundColor(Color::AnsiValue(51)))?; // Bright cyan header
    write!(stdout, "Offset    ")?;
    for i in 0..bytes_per_line {
        if i == 8 { write!(stdout, " ")?; }
        write!(stdout, "{:02x} ", i)?;
    }
    writeln!(stdout, " ASCII")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn draw_line<W: Write>(stdout: &mut W, editor: &MicroHex, line_start: usize) -> io::Result<()> {
    write!(stdout, "{:08x}: ", line_start)?;

    let line_end = (line_start + editor.bytes_per_line).min(editor.bytes.len());
    let chunk = &editor.bytes[line_start..line_end];

    // Hex bytes
    for (j, byte) in chunk.iter().enumerate() {
        if j == 8 { write!(stdout, " ")?; }
        let pos = line_start + j;
        set_cell_color(stdout, editor, pos, *byte, EditMode::EditHex)?;
        write!(stdout, "{:02x}", byte)?;
        queue!(stdout, ResetColor)?;
        write!(stdout, " ")?;
    }

    // Padding
    for p in chunk.len()..editor.bytes_per_line {
        if p == 8 { write!(stdout, " ")?; }
        write!(stdout, "   ")?;
    }
    write!(stdout, " ")?;

    // ASCII
    for (j, byte) in chunk.iter().enumerate() {
        let pos = line_start + j;
        let c = if byte.is_ascii_graphic() || *byte == b' ' { *byte as char } else { '.' };
        set_cell_color(stdout, editor, pos, *byte, EditMode::EditAscii)?;
        write!(stdout, "{}", c)?;
        queue!(stdout, ResetColor)?;
    }
    queue!(stdout, terminal::Clear(ClearType::UntilNewLine))?;
    writeln!(stdout)?;
    Ok(())
}

fn set_cell_color<W: Write>(
    stdout: &mut W,
    editor: &MicroHex,
    pos: usize,
    byte: u8,
    active_mode: EditMode,
) -> io::Result<()> {
    let is_changed = editor.original_bytes.get(pos) != Some(&byte);
    if pos == editor.cursor_pos {
        match &editor.mode {
            m if *m == active_mode => {
                // Yellow bg, black fg for the active editing mode
                queue!(stdout, SetBackgroundColor(Color::AnsiValue(226)), SetForegroundColor(Color::AnsiValue(16)))?
            }
            EditMode::EditHex | EditMode::EditAscii => {
                // Grey bg, white fg for the other editing mode
                queue!(stdout, SetBackgroundColor(Color::AnsiValue(240)), SetForegroundColor(Color::AnsiValue(255)))?
            }
            EditMode::View => {
                // White bg, black fg for view mode
                queue!(stdout, SetBackgroundColor(Color::AnsiValue(15)), SetForegroundColor(Color::AnsiValue(16)))?
            }
        }
    } else if is_changed {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(208)))?; // Orange fg
    } else if byte == 0 {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(242)))?; // Dark grey for zero
    } else if byte < 0x20 || byte >= 0x7f {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(33)))?; // Blue for control/non-printable
    } else if byte.is_ascii_graphic() || byte == b' ' {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(34)))?; // Green for printable
    }
    Ok(())
}