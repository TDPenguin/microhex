use std::fs;
use std::io::{self, Write};
use crossterm::queue;
use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, ClearType},
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
};

use crate::display;
use crate::navigation;
use crate::edit;
use crate::config::ColorConfig;

#[derive(PartialEq)]
pub enum EditMode {
    View,
    EditHex,
    EditAscii,
}

#[derive(Clone)]
pub struct UndoState {
    pub bytes: Vec<u8>,
    pub cursor_pos: usize,
    pub offset: usize,
    pub pending_nibble: Option<u8>,
}

pub struct MicroHex {
    pub original_bytes: Vec<u8>,
    pub bytes: Vec<u8>,
    pub undo_stack: Vec<UndoState>,
    pub filename: String,
    pub offset: usize, // Current view offset (which byte we start displaying from)
    pub cursor_pos: usize, // Which byte the cursor is on
    pub bytes_per_line: usize,
    pub lines_per_page: usize,
    pub mode: EditMode,
    pub modified: bool,
    pub pending_nibble: Option<u8>, // Stores the first hex digit if one has been entered
}

impl MicroHex {
    pub fn new(filename: String, bytes: Vec<u8>) -> io::Result<Self> {
        let (_, rows) = terminal::size()?;
        // Subtract rows for: status line (1) + blank line (1) + header (1) + blank line (1) + bottom margin (1) = 5 rows
        let lines_per_page = (rows as usize).saturating_sub(4).max(1);

        Ok(Self {
            original_bytes: bytes.clone(),
            bytes,
            undo_stack: Vec::new(),
            filename,
            offset: 0,
            cursor_pos: 0,
            bytes_per_line: 16,
            lines_per_page,
            mode: EditMode::View,
            modified: false,
            pending_nibble: None,
        })
    }

    pub fn run(&mut self, colors: &ColorConfig) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        loop {
            display::draw(self, colors)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.handle_key_event(key)? {
                        break;
                    }
                }
            }
        }

        terminal::disable_raw_mode()?;
        execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> io::Result<bool> {
        match key.code {

            // FILE/MODE CONTROLS
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::ALT) => {
                if self.modified {
                    if let Some(ans) = self.prompt("File modified. Save before exit? (y/n/c): ")? {
                        match ans {
                            'y' => { self.save()?; return Ok(true); }
                            'n' => return Ok(true),
                            _ => return Ok(false),
                        }
                    }
                } else {
                    return Ok(true);
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) && self.modified => {
                if let Some(ans) = self.prompt("Really save changes? (y/n): ")? {
                    if ans == 'y' {
                        self.save()?;
                    }
                }
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                edit::cycle_mode(self);
            }
            KeyCode::Tab => edit::cycle_mode(self),


            // UNDO CONTROLS
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                edit::undo(self);
            }


            // NAVIGATION CONTROLS
            KeyCode::Up => navigation::move_up(self),
            KeyCode::Down => navigation::move_down(self),
            KeyCode::Left => navigation::move_left(self),
            KeyCode::Right => navigation::move_right(self),
            KeyCode::PageUp => {
                let speed = if key.modifiers.contains(KeyModifiers::SHIFT) { 10 } else { 1 };
                navigation::page_up(self, speed);
            }
            KeyCode::PageDown => {
                let speed = if key.modifiers.contains(KeyModifiers::SHIFT) { 10 } else { 1 };
                navigation::page_down(self, speed);
            }
            KeyCode::Home => navigation::move_home(self),
            KeyCode::End => navigation::move_end(self),


            // EDITING CONTROLS
            KeyCode::Char(c) if !matches!(self.mode, EditMode::View) => {
                edit::edit_byte(self, c);
            }
            KeyCode::Delete if !matches!(self.mode, EditMode::View) => {
                edit::delete_prev_byte(self);
            }
            KeyCode::Backspace if !matches!(self.mode, EditMode::View) => {
                edit::backspace(self);
            }
            _ => {}
        }
        Ok(false)
    }

    fn prompt(&self, message: &str) -> io::Result<Option<char>> {
        let mut stdout = io::stdout();
        queue!(stdout, cursor::MoveTo(0, (self.lines_per_page + 4) as u16))?;
        queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
        write!(stdout, "{}", message)?;
        stdout.flush()?;
        loop {
            if let Event::Key(key) = event::read()? {
                // Only check for PRESS events, just like the main run loop
                // This prevents capturing the key *release* from the command that opened the prompt
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char(c) = key.code {
                        return Ok(Some(c.to_ascii_lowercase()));
                    } else if key.code == KeyCode::Esc {
                        return Ok(Some('c')); // Treat Esc as cancel
                    }
                }
            }
        }
    }

    fn save(&mut self) -> io::Result<()> {
        // Trim trailing null bytes (0x00) before saving, but always leave at least one byte
        let mut data = self.bytes.clone();
        while data.len() > 1 && data.last() == Some(&0) {
            data.pop();
        }
        fs::write(&self.filename, &data)?;
        // Update original_bytes and bytes to match trimmed data
        self.original_bytes = data.clone();
        self.bytes = data;
        self.modified = false;
        Ok(())
    }
}