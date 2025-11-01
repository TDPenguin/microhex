//! Main editor state machine and event loop.
//!
//! Contains the `MicroHex` struct, which holds all editor state, and implements the main TUI loop (`run`).
//! Handles file I/O, mode management, user prompts, and dispatches navigation/edit/display actions.
//! All user input is processed here and routed to the appropriate module.

use std::fs;
use std::io::{self, Write};
use crossterm::queue;
use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, ClearType},
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
};

use crate::{display, navigation, edit, config::ColorConfig, search};

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
    pub search_state: Option<search::SearchState>, // Active search session, if any
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
            search_state: None,
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
                    if self.handle_key_event(key, colors)? {
                        break;
                    }
                }
            }
        }

        terminal::disable_raw_mode()?;
        execute!(io::stdout(), cursor::Show, LeaveAlternateScreen)?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent, colors: &ColorConfig) -> io::Result<bool> {
        match key.code {

            // FILE/MODE CONTROLS
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::ALT) => {
                if self.modified {
                    if let Some(ans) = self.prompt("File modified. Save before exit? (y/n/c): ")? {
                        match ans.to_lowercase().as_str() {
                            "y" => { self.save()?; return Ok(true); }
                            "n" => return Ok(true),
                            _ => return Ok(false),
                        }
                    }
                } else {
                    return Ok(true);
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) && self.modified => {
                if let Some(ans) = self.prompt("Really save changes? (y/n): ")? {
                    if ans.to_lowercase() == "y" {
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

            // SEARCH MODE
            KeyCode::Char('/') => {
                // Prompt user for search pattern (hex or ASCII)
                if let Some(pattern_str) = self.prompt("Search [0xHEX | text:ASCII | auto]: ")? {
                    // Convert input string to a byte pattern using search::parse_pattern
                    if let Some(pattern) = search::parse_pattern(&pattern_str) {
                        // Create a new search state by finding all matches
                        self.search_state = search::SearchState::new(&self.bytes, pattern);
                        
                        if let Some(ref state) = self.search_state {
                            self.cursor_pos = state.current_position();
                            navigation::scroll_to_cursor(self);
                            // Search info now displays persistently in help bar
                        } else {
                            display::show_message(
                                self,
                                "Pattern not found. Press any key to continue...",
                                colors,
                            )?;
                        }
                    } else {
                        display::show_message(
                            self,
                            "Invalid pattern. Use even-length hex or ASCII. Press any key to continue...",
                            colors,
                        )?;
                    }
                }
            }
            
            // Clear search
            KeyCode::Esc if self.search_state.is_some() => {
                self.search_state = None;
            }
            
            // Next search match
            KeyCode::Char('n') if self.search_state.is_some() => {
                if let Some(ref mut state) = self.search_state {
                    state.next_match();
                    self.cursor_pos = state.current_position();
                    navigation::scroll_to_cursor(self);
                    // Match info displays in help bar automatically
                }
            }
            
            // Previous search match (Shift+N)
            KeyCode::Char('N') if self.search_state.is_some() => {
                if let Some(ref mut state) = self.search_state {
                    state.prev_match();
                    self.cursor_pos = state.current_position();
                    navigation::scroll_to_cursor(self);
                    // Match info displays in help bar automatically
                }
            }

            _ => {}
        }
        Ok(false)
    }

    fn prompt(&self, message: &str) -> io::Result<Option<String>> {
        let mut stdout = io::stdout();
        let mut input = String::new();
        let prompt_row = (self.lines_per_page + 4) as u16;
        
        loop {
            queue!(stdout, cursor::MoveTo(0, prompt_row))?;
            queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            write!(stdout, "{}{}", message, input)?;
            stdout.flush()?;
            
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            return if input.is_empty() {
                                Ok(None)
                            } else {
                                Ok(Some(input))
                            };
                        }
                        KeyCode::Esc => {
                            return Ok(None);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        _ => {}
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
        // Update original_bytes and bytes to match the saved state
        self.original_bytes = data.clone();
        self.bytes = data;
        self.modified = false;
        Ok(())
    }
}