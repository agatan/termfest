use num::FromPrimitive;

use terminal::Terminal;
use key::Key;

/// `Event` is an event of termfest, that contains special key pressed, character input, and window
/// resize.
#[derive(Debug, Clone)]
pub enum Event {
    /// `Key` is an event that notify a special key (e.g. ctrl-A, Space, Enter) is pressed.
    Key(Key),
    /// `Char` is an event that notify the input byte sequence is a non-special character.
    Char(char),
    Resize { width: usize, height: usize },
}

/// Parse event from buffer.
/// `None` means 'buffered bytes are not enough'.
pub fn parse(buf: &[u8], term: &Terminal) -> Option<(usize, Event)> {
    if buf.is_empty() {
        return None;
    }
    if buf[0] == b'\x1b' {
        // escape sequence
        if let Some(result) = parse_escape_sequence(buf, term) {
            return Some(result);
        }
    }

    if let Some(key) = key_from_byte(buf[0]) {
        // for single-byte keys like ctrl-A
        return Some((1, Event::Key(key)));
    }

    let (len_utf8, ch) = match decode_char(buf) {
        None => return None,
        Some(r) => r,
    };
    Some((len_utf8, Event::Char(ch)))
}

// Copy from core/str/mod.rs in Rust.
// Because `std::io::Chars` is unstable yet, we cannot use this in std library.
// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
    0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
    2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
    3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
    4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

#[inline]
fn utf8_char_width(b: u8) -> usize {
    return UTF8_CHAR_WIDTH[b as usize] as usize;
}

fn decode_char(buf: &[u8]) -> Option<(usize, char)> {
    let first_byte = match buf.first() {
        None => return None,
        Some(&byte) => byte,
    };
    let width = utf8_char_width(first_byte);
    if width == 1 {
        return Some((width, first_byte as char));
    }
    if width == 0 {
        return None;
    }
    if buf.len() < width {
        return None;
    }
    match ::std::str::from_utf8(&buf[0..width]) {
        Ok(s) => Some((width, s.chars().next().unwrap())),
        Err(_) => None,
    }
}

fn parse_escape_sequence(buf: &[u8], term: &Terminal) -> Option<(usize, Event)> {
    debug_assert!(buf[0] == b'\x1b');

    for &key in ESCAPE_KEYS.iter() {
        if let Some(keybytes) = term.escaped_key_bytes(key) {
            if buf.starts_with(keybytes) {
                return Some((keybytes.len(), Event::Key(key)));
            }
        }
    }
    None
}

fn key_from_byte(byte: u8) -> Option<Key> {
    if byte as isize <= Key::Backspace as isize {
        Key::from_u8(byte)
    } else {
        None
    }
}

static ESCAPE_KEYS: [Key; 4] = [
    Key::ArrowUp,
    Key::ArrowDown,
    Key::ArrowLeft,
    Key::ArrowRight,
];
