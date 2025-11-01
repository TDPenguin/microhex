# microhex
A modern, lightweight TUI hex editor - Like [Nano](https://nano-editor.org/), but for binary files.

## Why
When you need to inspect or edit raw binary files—firmware, dumps, corrupted files, or any “bytes and bits” scenario—you shouldn’t need a heavy GUI tool.
microhex gives you:
- Terminal-native interface
- Fast startup and small footprint
- Intuitive keybindings and modes (inspired by Nano)
- Built in Rust for safety and performance
- Perfect for live installs, rescue environments, embedded systems

## Features
- Open and save binary files of any size
- Navigate via arrow keys, PageUp/PageDown, go-to offset
- Edit bytes (overwrite, insert, delete)
- Toggle between hex and ASCII view
- Cross-platform: Windows, Linux (buggy due to terminal issues at the moment)

## Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/TDPenguin/microhex.git
   cd microhex
   ```
2. Build via Cargo (requires Rust):
   ```bash
   cargo build --release
   ```
3. Copy the binary to your PATH (optional):
   ```
   cp target/release/microhex /usr/local/bin/
   ```
   or on Windows, place microhex.exe somewhere in your PATH, or append it to PATH.
4. Run:
   ```bash
   microhex path/to/file.bin
   ```
   *Note: doesn't have to be .bin, can be any format.*

## Usage
* Open a file: microhex myfile.bin
* Navigate: arrow keys, PageUp/PageDown, Home/End (partially implemented)
* Ctrl+E/Tab to switch modes, VIEW, EDIT (HEX), EDIT (ASCII).
* Edit mode: press i to insert, o to overwrite, d to delete byte(s) (WIP for all)
* Save: Ctrl+S/Ctrl+O
* Quit: Ctrl+Q (Prompts if unsaved changes)
* Help: Ctrl+G (WIP)

## License
Distributed under the MIT license. See `LICENSE` for details.

## Roadmap

### v0.8.x - Polishing and Essentials (Current phase)
- Search for hex/ASCII patterns (`/` key, highlight matches, jump to match)  
- Insert vs. overwrite toggle for hex and ASCII editing  
- Block selection and copy-paste (Shift + arrows)  
- Visual improvements: cursor flashing, pending nibble marker  
- Autosave/backup on crash; prompt for `.bak` on overwrite   
- Nano-like keybindings: Ctrl+S save, Ctrl+Q quit, Ctrl+G help, etc.  
- QoL improvements: Home/End, Delete, display cursor offset  

### v1.0.0 – Feature Complete Core
- Diff mode: compare two files side by side
- Export bytes: C array, Rust slice, hex dump  
- Color coding by byte type (changed bytes, nulls, data sections, somewhat implemented)
- Jump to address (`g`)  
- Undo/redo stack  
- User configuration: toggle null glyph (`.` vs `·`), configurable bytes per line  
- Multi-line bytes-per-line display: 8, 16, 32 bytes/line  

### v2.x – Expanded / GUI Transition (Will branch off)
- GUI version (egui, Slint, or similar) while keeping core TUI functionality  
- Scriptable macros or plugin support  
- File-specific views (e.g., WAV, PNG, or structured binary)  
- Advanced binary operations (XOR selection, arithmetic, etc.)  
- Bookmarks: mark positions and jump back  
- Advanced search (regex, ranges, multiple matches)  