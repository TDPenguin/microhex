use crate::editor::{MicroHex, EditMode};

pub fn cycle_mode(editor: &mut MicroHex) {
    editor.mode = match editor.mode {
        EditMode::View => EditMode::EditHex,
        EditMode::EditHex => EditMode::EditAscii,
        EditMode::EditAscii => EditMode::View,
    };
    editor.pending_nibble = None; // Clear any pending nibble when switching modes
}

pub fn edit_byte(editor: &mut MicroHex, c: char) {
    match editor.mode {
        EditMode::EditAscii => {
            // ASCII editing mode
            if c.is_ascii() {
                editor.bytes[editor.cursor_pos] = c as u8;
                editor.modified = true;
                // Always auto-advance after entering a character
                editor.cursor_pos += 1;
                // If we're now at the end in edit mode, append a new null byte
                if editor.cursor_pos >= editor.bytes.len() {
                    editor.bytes.push(0);
                    editor.original_bytes.push(0);
                }
                // Scroll window if cursor goes below visible window
                if editor.cursor_pos >= editor.offset + (editor.bytes_per_line * editor.lines_per_page) {
                    editor.offset += editor.bytes_per_line;
                }
            }
        }
        EditMode::EditHex => {
            // Only accept hex digits (0-9, a-f, A-F)
            if let Some(d) = c.to_digit(16) {
                if editor.pending_nibble.is_none() {
                    // First nibble: set high nibble, keep low nibble
                    editor.bytes[editor.cursor_pos] = (editor.bytes[editor.cursor_pos] & 0x0F) | ((d as u8) << 4);
                    editor.pending_nibble = Some(d as u8);
                    editor.modified = true;
                } else {
                    // Second nibble: set low nibble, keep high nibble
                    editor.bytes[editor.cursor_pos] = (editor.bytes[editor.cursor_pos] & 0xF0) | (d as u8);
                    editor.pending_nibble = None;
                    editor.modified = true;
                    // Advance cursor after completing the byte
                    editor.cursor_pos += 1;
                    // If we're now at the end in edit mode, append a new null byte
                    if editor.cursor_pos >= editor.bytes.len() {
                        editor.bytes.push(0);
                        editor.original_bytes.push(0);
                    }
                    // Scroll window if needed
                    if editor.cursor_pos >= editor.offset + (editor.bytes_per_line * editor.lines_per_page) {
                        editor.offset += editor.bytes_per_line;
                    }
                }
            }
        }
        EditMode::View => {}
    }
}

pub fn backspace(editor: &mut MicroHex) {
    // Set the current byte to null (0x00), then move the cursor back one (if not at 0)
    if editor.cursor_pos < editor.bytes.len() {
        editor.bytes[editor.cursor_pos] = 0;
        editor.modified = true;
        if editor.cursor_pos > 0 {
            editor.cursor_pos -= 1;
        }
    }
}

pub fn delete_prev_byte(editor: &mut MicroHex) {
    // Completely remove the byte at the current cursor position, then move back
    // But never delete the last remaining byte
    if editor.cursor_pos < editor.bytes.len() && editor.bytes.len() > 1 {
        editor.bytes.remove(editor.cursor_pos);
        editor.original_bytes.remove(editor.cursor_pos);
        editor.modified = true;
        // Move cursor back after deletion (unless we're at position 0)
        if editor.cursor_pos > 0 {
            editor.cursor_pos -= 1;
        }
        // Adjust offset if needed
        if editor.cursor_pos < editor.offset {
            editor.offset = editor.offset.saturating_sub(editor.bytes_per_line);
        }
    }
}