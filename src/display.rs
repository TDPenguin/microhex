use std::io::{self, Write};
use crossterm::{
    queue,
    terminal::{self, ClearType},
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor},
    cursor,
};

use crate::config::ColorConfig;
use crate::editor::{MicroHex, EditMode};

pub fn draw(editor: &mut MicroHex, colors: &ColorConfig) -> io::Result<()> {
    let mut stdout = io::stdout();
    let (cols, rows) = terminal::size()?;

    // Calculate minimum size: 
    // - Status bar (1) + blank (1) + header (1) + at least 4 lines of data (4) + help bar (1) = 8 rows minimum
    // - For columns: offset (10) + 16*3 (hex bytes + spaces) + 2 (ASCII margin) + 16 (ASCII) = 76 columns minimum for 16 bytes/line
    let min_lines = 8;
    let min_cols = 76;

    if cols < min_cols || rows < min_lines {
        queue!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;
        writeln!(stdout, "Terminal too small! Resize to at least {min_cols}x{min_lines}.")?;
        stdout.flush()?;
        return Ok(());
    }

    // Dynamically recalculate lines_per_page for current term size
    editor.lines_per_page = (rows as usize).saturating_sub(4).max(1);

    queue!(stdout, cursor::MoveTo(0, 0))?;
    draw_status_line(&mut stdout, editor, cols, colors)?;
    writeln!(stdout)?; // Blank line after status bar
    draw_header(&mut stdout, editor.bytes_per_line, cols, colors)?;

    let end_offset = (editor.offset + editor.bytes_per_line * editor.lines_per_page).min(editor.bytes.len());

    for line_start in (editor.offset..end_offset).step_by(editor.bytes_per_line) {
        draw_line(&mut stdout, editor, line_start, colors)?;
    }
    
    queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;
    draw_help_bar(&mut stdout, editor, cols, colors)?;
    stdout.flush()?;
    Ok(())
} 

// This function is generic over any writer that implements std::io::Write (such as Stdout, a file, or a buffer).
// Using W: Write allows us to reuse this function for testing, alternate outputs, or redirection if needed.
fn draw_status_line<W: Write>(stdout: &mut W, editor: &MicroHex, cols: u16, colors: &ColorConfig) -> io::Result<()> {
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
    let status = format!(
        "File: {} ({} bytes) | {:.1}% | {} | Cursor: 0x{:X} ({})",
        editor.filename, file_size, percent, mode_str, editor.cursor_pos, editor.cursor_pos
    );
    let mut line = status.chars().take(cols as usize).collect::<String>();
    if line.len() < cols as usize {
        line.push_str(&" ".repeat(cols as usize - line.len()));
    }
    queue!(
        stdout,
        SetBackgroundColor(Color::AnsiValue(colors.status_bg)),
        SetForegroundColor(Color::AnsiValue(colors.status_fg))
    )?;
    write!(stdout, "{line}")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn draw_help_bar<W: Write>(stdout: &mut W, editor: &MicroHex, cols: u16, colors: &ColorConfig) -> io::Result<()> {
    let help_row = (editor.lines_per_page + 2) as u16;
    let help_text = "^G Help   ^X Exit   ^S Save   ^E/Tab Mode   ^Z Undo";
    let mut line = help_text.chars().take(cols as usize).collect::<String>();
    if line.len() < cols as usize {
        line.push_str(&" ".repeat(cols as usize - line.len()));
    }
    queue!(
        stdout,
        cursor::MoveTo(0, help_row),
        SetBackgroundColor(Color::AnsiValue(colors.help_bg)),
        SetForegroundColor(Color::AnsiValue(colors.help_fg)),
    )?;
    write!(stdout, "{line}")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn draw_header<W: Write>(stdout: &mut W, bytes_per_line: usize, cols: u16, colors: &ColorConfig) -> io::Result<()> {
    queue!(stdout, SetForegroundColor(Color::AnsiValue(colors.header_fg)))?; // Configurable header color
    let mut header = String::from("Offset    ");
    for i in 0..bytes_per_line {
        if i == 8 { header.push(' '); }
        header.push_str(&format!("{:02x} ", i));
    }
    header.push_str(" ASCII");
    let mut line = header.chars().take(cols as usize).collect::<String>();
    if line.len() < cols as usize {
        line.push_str(&" ".repeat(cols as usize - line.len()));
    }
    writeln!(stdout, "{line}")?;
    queue!(stdout, ResetColor)?;
    Ok(())
}

fn draw_line<W: Write>(stdout: &mut W, editor: &MicroHex, line_start: usize, colors: &ColorConfig) -> io::Result<()> {
    write!(stdout, "{:08x}: ", line_start)?;

    let line_end = (line_start + editor.bytes_per_line).min(editor.bytes.len());
    let chunk = &editor.bytes[line_start..line_end];

    // Hex bytes
    for (j, byte) in chunk.iter().enumerate() {
        if j == 8 { write!(stdout, " ")?; }
        let pos = line_start + j;
        set_cell_color(stdout, editor, pos, *byte, EditMode::EditHex, colors)?;
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
        set_cell_color(stdout, editor, pos, *byte, EditMode::EditAscii, colors)?;
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
    colors: &ColorConfig,
) -> io::Result<()> {
    let is_changed = editor.original_bytes.get(pos) != Some(&byte);
    if pos == editor.cursor_pos {
        match &editor.mode {
            m if *m == active_mode => {
                // Configurable: active editing mode
                queue!(
                    stdout,
                    SetBackgroundColor(Color::AnsiValue(colors.cursor_active_bg)),
                    SetForegroundColor(Color::AnsiValue(colors.cursor_active_fg))
                )?
            }
            EditMode::EditHex | EditMode::EditAscii => {
                // Configurable: inactive editing mode
                queue!(
                    stdout,
                    SetBackgroundColor(Color::AnsiValue(colors.cursor_inactive_bg)),
                    SetForegroundColor(Color::AnsiValue(colors.cursor_inactive_fg))
                )?
            }
            EditMode::View => {
                // Configurable: view mode
                queue!(
                    stdout,
                    SetBackgroundColor(Color::AnsiValue(colors.help_bg)),
                    SetForegroundColor(Color::AnsiValue(colors.help_fg))
                )?
            }
        }
    } else if is_changed {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(colors.changed_fg)))?; // Changed byte
    } else if byte == 0 {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(colors.null_fg)))?; // Null byte
    } else if byte < 0x20 || byte >= 0x7f {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(colors.control_fg)))?; // Control/non-printable
    } else if byte.is_ascii_graphic() || byte == b' ' {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(colors.printable_fg)))?; // Printable
    }
    Ok(())
}