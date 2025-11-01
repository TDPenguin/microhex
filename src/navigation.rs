use crate::editor::{MicroHex, EditMode};

pub fn move_up(editor: &mut MicroHex) {
    // If the cursor is not already on the first line, move it up by one line (subtract bytes_per_line)
    if editor.cursor_pos >= editor.bytes_per_line {
        editor.cursor_pos -= editor.bytes_per_line;
        // If moving the cursor up would put it above the visible window, scroll the window up by one line
        if editor.cursor_pos < editor.offset {
            // .saturating_sub prevents underflow (so offset never goes below 0)
            editor.offset = editor.offset.saturating_sub(editor.bytes_per_line);
        }
    }
}

pub fn move_down(editor: &mut MicroHex) {
    // If moving the cursor down by one line would stay within the file, move it down (add bytes_per_line)
    if editor.cursor_pos + editor.bytes_per_line < editor.bytes.len() {
        editor.cursor_pos += editor.bytes_per_line;
        // If moving the cursor down would put it below the visible window, scroll the window down by one line
        if editor.cursor_pos >= editor.offset + (editor.bytes_per_line * (editor.lines_per_page - 1)) {
            editor.offset += editor.bytes_per_line;
        }
    }
}

pub fn move_left(editor: &mut MicroHex) {
    // If cursor position is greater than 0 (it's not on the very first byte), move it left by one byte
    if editor.cursor_pos > 0 {
        editor.cursor_pos -= 1; // Move cursor one byte to the left
        // If moving left puts the cursor before the visible window, scroll the window left (up) by one line
        if editor.cursor_pos < editor.offset {
            // .saturating_sub prevents underflow (so offset never goes below 0)
            editor.offset = editor.offset.saturating_sub(editor.bytes_per_line);
        }
    }
}

pub fn move_right(editor: &mut MicroHex) {
    // If the cursor is not on the last byte, move it right by one byte
    if editor.cursor_pos < editor.bytes.len() - 1 {
        editor.cursor_pos += 1;
        // If moving right puts the cursor past the visible window, scroll the window down by one line
        if editor.cursor_pos >= editor.offset + (editor.bytes_per_line * (editor.lines_per_page - 1)) {
            editor.offset += editor.bytes_per_line;
        }
    } else if editor.mode != EditMode::View {
        // If in edit mode and at the end, append a new byte and move cursor
        editor.bytes.push(0);
        editor.original_bytes.push(0);
        editor.cursor_pos += 1;
        // Scroll if needed (this was missing!)
        if editor.cursor_pos >= editor.offset + (editor.bytes_per_line * editor.lines_per_page) {
            editor.offset += editor.bytes_per_line;
        }
    }
}

pub fn page_up(editor: &mut MicroHex, factor: usize) {
    // Move the window up by multiple pages (factor times the normal page size), but never below 0
    let jump = editor.bytes_per_line * editor.lines_per_page * factor; // Calculate how many bytes to jump (factor pages)
    editor.offset = editor.offset.saturating_sub(jump); // .saturating_sub prevents underflow (offset never goes below 0)
    editor.cursor_pos = editor.offset; // Move the cursor to the first byte of the new window
}

pub fn page_down(editor: &mut MicroHex, factor: usize) {
    // Move the window down by multiple pages (factor times the normal page size)
    let jump = editor.bytes_per_line * editor.lines_per_page * factor; // Calculate how many bytes to jump (factor pages)
    let new_offset = editor.offset + jump; // Add jump to current offset

    // Don't go past the last full line that can be displayed
    let max_offset = editor.bytes.len().saturating_sub(1); // Last valid byte index
    let max_line_start = (max_offset / editor.bytes_per_line) * editor.bytes_per_line; // Start of last full line

    editor.offset = new_offset.min(max_line_start); // Clamp offset so we don't scroll past the end
    editor.cursor_pos = editor.offset; // Move the cursor to the first byte of the new window
}