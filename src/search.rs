//! Search logic for microhex-tui.
//!
//! Provides functions for searching for byte patterns (hex or ASCII) within the file buffer.
//! Intended for use by the editor event loop to implement search mode, jump to match, and (in the future) search/replace features.
//!
//! All search operations are stateless and operate on slices of the file data.

/// Holds the state of an active search session.
/// Tracks all match positions and current position.
pub struct SearchState {
    pub matches: Vec<usize>,       // All positions where pattern was found
    pub current_index: usize,      // Index into matches (which match we're viewing)
}

impl SearchState {
    /// Create a new search state by finding all matches of pattern in data.
    pub fn new(data: &[u8], pattern: Vec<u8>) -> Option<Self> {
        let matches = search_all_bytes(data, &pattern);
        if matches.is_empty() {
            None
        } else {
            Some(Self {
                matches,
                current_index: 0,
            })
        }
    }

    /// Get the current match position.
    pub fn current_position(&self) -> usize {
        self.matches[self.current_index]
    }

    /// Move to the next match, wrapping around to the start if at the end.
    pub fn next_match(&mut self) {
        self.current_index = (self.current_index + 1) % self.matches.len();
    }

    /// Move to the previous match, wrapping around to the end if at the start.
    pub fn prev_match(&mut self) {
        if self.current_index == 0 {
            self.current_index = self.matches.len() - 1;
        } else {
            self.current_index -= 1;
        }
    }

    /// Get the total number of matches.
    pub fn total_matches(&self) -> usize {
        self.matches.len()
    }

    /// Get a user-friendly string describing the current match position.
    pub fn match_info(&self) -> String {
        format!("Match {} of {}", self.current_index + 1, self.matches.len())
    }
}

/// Parse a user input string as either a hex pattern or ASCII bytes.
/// 
/// Supports three formats:
/// - `0x` prefix: Forces hex interpretation (e.g., "0x4f", "0xFA12")
/// - `text:` prefix: Forces ASCII interpretation (e.g., "text:4f2a")
/// - Auto-detect: Even-length all-hex → hex bytes, otherwise → ASCII
/// 
/// Returns a Vec<u8> of the pattern, or None if the input is invalid.
pub fn parse_pattern(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    
    // Check for explicit hex prefix (0x or 0X)
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        let hex_part = &trimmed[2..];
        if hex_part.is_empty() {
            return None;
        }
        // Parse hex digits, allowing spaces (e.g., "0x4f 2a" or "0x4f2a")
        let hex_clean: String = hex_part.chars().filter(|c| !c.is_whitespace()).collect();
        
        // Pad with leading 0 if odd length (e.g., "0xf" -> "0x0f")
        let hex_padded = if hex_clean.len() % 2 == 1 {
            format!("0{}", hex_clean)
        } else {
            hex_clean
        };
        
        if !hex_padded.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        
        let bytes = (0..hex_padded.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_padded[i..i + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        return Some(bytes);
    }
    
    // Check for explicit ASCII/text prefix
    if trimmed.starts_with("text:") {
        let text_part = &trimmed[5..];
        return Some(text_part.as_bytes().to_vec());
    }
    
    // Auto-detect: Check if all characters are valid hex digits
    let all_hex = trimmed.chars().all(|c| c.is_ascii_hexdigit());
    
    // If all chars are hex AND even length, parse as hex bytes
    if all_hex && trimmed.len() % 2 == 0 {
        // For each pair of hex digits, convert to a u8 using base 16 (hexadecimal)
        // Example: "4F" -> 79, "fa" -> 250
        let bytes = (0..trimmed.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&trimmed[i..i + 2], 16)) // from_str_radix parses a string slice as a number in the given base
            .collect::<Result<Vec<_>, _>>() // Collects all results into a Vec<u8>, or returns error if any fail
            .ok()?; // If any conversion fails, return None
        Some(bytes)
    } else {
        // Otherwise, treat as ASCII: convert each char to its byte value
        Some(trimmed.as_bytes().to_vec())
    }
}

/// Search for ALL occurrences of "pattern" in given "data".
/// Returns a Vec of all starting indices where pattern is found.
pub fn search_all_bytes(data: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() || pattern.len() > data.len() {
        return Vec::new();
    }
    data.windows(pattern.len())
        .enumerate()
        .filter_map(|(i, window)| {
            if window == pattern {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}