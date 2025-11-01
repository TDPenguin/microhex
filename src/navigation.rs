//! Cursor and window navigation logic.
//!
//! Implements movement commands (arrows, paging, Home/End, etc.) for the editor.
//! All navigation functions operate on a mutable reference to `MicroHex` and update cursor/offset as needed.

use crate::editor::{MicroHex, EditMode};

pub fn scroll_to_cursor(editor: &mut MicroHex) {
    // Scroll up if cursor is above the visible window
    if editor.cursor_pos < editor.offset {
        editor.offset = editor.cursor_pos - (editor.cursor_pos % editor.bytes_per_line);
    }
    // Scroll down if cursor is below the visible window
    let window_end = editor.offset + (editor.bytes_per_line * (editor.lines_per_page - 1));
    if editor.cursor_pos > window_end {
        editor.offset = editor.cursor_pos - (editor.cursor_pos % editor.bytes_per_line)
            - editor.bytes_per_line * (editor.lines_per_page - 1);
        // Clamp to 0 if underflow
        if editor.offset > editor.cursor_pos {
            editor.offset = 0;
        }
    }
}

pub fn move_up(editor: &mut MicroHex) {
    if editor.cursor_pos >= editor.bytes_per_line {
        editor.cursor_pos -= editor.bytes_per_line;
        scroll_to_cursor(editor);
    }
    editor.pending_nibble = None;
}

pub fn move_down(editor: &mut MicroHex) {
    if editor.cursor_pos + editor.bytes_per_line < editor.bytes.len() {
        editor.cursor_pos += editor.bytes_per_line;
        scroll_to_cursor(editor);
    }
    editor.pending_nibble = None;
}

pub fn move_left(editor: &mut MicroHex) {
    if editor.cursor_pos > 0 {
        editor.cursor_pos -= 1;
        scroll_to_cursor(editor);
    }
    editor.pending_nibble = None;
}

pub fn move_right(editor: &mut MicroHex) {
    if editor.cursor_pos < editor.bytes.len() - 1 {
        editor.cursor_pos += 1;
        scroll_to_cursor(editor);
    } else if editor.mode != EditMode::View {
        // In edit mode, allow expanding the file
        editor.bytes.push(0);
        editor.cursor_pos += 1;
        scroll_to_cursor(editor);
    }
    editor.pending_nibble = None;
}

pub fn page_up(editor: &mut MicroHex, factor: usize) {
    let jump = editor.bytes_per_line * editor.lines_per_page * factor;
    editor.offset = editor.offset.saturating_sub(jump);
    editor.cursor_pos = editor.offset;
    editor.pending_nibble = None;
}

pub fn page_down(editor: &mut MicroHex, factor: usize) {
    let jump = editor.bytes_per_line * editor.lines_per_page * factor;
    let new_offset = editor.offset + jump;

    // Clamp to last full line that can be displayed
    let max_offset = editor.bytes.len().saturating_sub(1);
    let max_line_start = (max_offset / editor.bytes_per_line) * editor.bytes_per_line;

    editor.offset = new_offset.min(max_line_start);
    editor.cursor_pos = editor.offset;
    editor.pending_nibble = None;
}

pub fn move_home(editor: &mut MicroHex) {
    editor.cursor_pos = 0;
    scroll_to_cursor(editor);
    editor.pending_nibble = None;
}

pub fn move_end(editor: &mut MicroHex) {
    if !editor.bytes.is_empty() { // If file exists
        editor.cursor_pos = editor.bytes.len() - 1; // Last valid index
        scroll_to_cursor(editor);
    }
    editor.pending_nibble = None;
}