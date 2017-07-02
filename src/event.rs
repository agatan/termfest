use std::io::{self, Read};

use terminal::Terminal;

use num::FromPrimitive;

#[derive(Debug, Clone)]
pub enum Event {
    /// `Key` is an event that notify a special key (e.g. ctrl-A, Space, Enter) is pressed.
    Key(Key),
    /// `Char` is an event that notify the input byte sequence is a non-special character.
    Char(char),
    Resize { width: i32, height: i32 },
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum Key {
    CtrlA = 0x01,
    CtrlB = 0x02,
    CtrlC = 0x03,
    CtrlD = 0x04,
    CtrlE = 0x05,
    CtrlF = 0x06,
    CtrlG = 0x07,
    CtrlH = 0x08,
    CtrlI = 0x09,
    CtrlJ = 0x0a,
    CtrlK = 0x0b,
    CtrlL = 0x0c,
    CtrlM = 0x0d,
    CtrlN = 0x0e,
    CtrlO = 0x0f,
    CtrlP = 0x10,
    CtrlQ = 0x11,
    CtrlR = 0x12,
    CtrlS = 0x13,
    CtrlT = 0x14,
    CtrlU = 0x15,
    CtrlV = 0x16,
    CtrlW = 0x17,
    CtrlX = 0x18,
    CtrlY = 0x19,
    CtrlZ = 0x1a,
    ESC = 0x1b,
    Space = 0x20,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

pub use Key::*;

static ESCAPE_KEYS: [Key; 4] = [Key::ArrowUp,
                                Key::ArrowDown,
                                Key::ArrowLeft,
                                Key::ArrowRight];

impl Event {
    /// Parse event from buffer. Returns `Err` if any IO error occurs.
    /// `Ok(None)` means 'no error occurs, but buffered bytes is not enough'.
    pub fn parse(buf: &[u8], term: &Terminal) -> io::Result<Option<(usize, Event)>> {
        if buf.is_empty() {
            return Ok(None);
        }
        if buf[0] == b'\x1b' {
            // escape sequence
            if let Some(result) = Event::parse_escape_sequence(buf, term)? {
                return Ok(Some(result));
            }
        }

        if let Some(key) = Key::from_byte(buf[0]) {
            // for single-byte keys like ctrl-A
            return Ok(Some((1, Event::Key(key))));
        }

        let ch = match buf.chars().next() {
            None => return Ok(None),
            Some(Ok(ch)) => ch,
            Some(Err(io::CharsError::NotUtf8)) => return Ok(None),
            Some(Err(io::CharsError::Other(err))) => return Err(err.into()),
        };
        Ok(Some((ch.len_utf8(), Event::Char(ch))))
    }

    fn parse_escape_sequence(buf: &[u8], term: &Terminal) -> io::Result<Option<(usize, Event)>> {
        debug_assert!(buf[0] == b'\x1b');

        for &key in ESCAPE_KEYS.iter() {
            if let Some(keybytes) = term.escaped_key_bytes(key) {
                if buf.starts_with(keybytes) {
                    return Ok(Some((keybytes.len(), Event::Key(key))));
                }
            }
        }

        Ok(None)
    }
}

impl Key {
    fn from_byte(byte: u8) -> Option<Self> {
        if byte as isize <= Key::Space as isize {
            Key::from_u8(byte)
        } else {
            None
        }
    }
}
