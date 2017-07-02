use std::io::{self, Read};

use terminal::Terminal;

#[derive(Debug, Clone)]
pub enum Event {
    Key(Key),
    Resize { width: i32, height: i32 },
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Char(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

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

        let ch = match buf.chars().next() {
            None => return Ok(None),
            Some(Ok(ch)) => ch,
            Some(Err(io::CharsError::NotUtf8)) => return Ok(None),
            Some(Err(io::CharsError::Other(err))) => return Err(err.into()),
        };
        Ok(Some((ch.len_utf8(), Event::Key(Key::Char(ch)))))
    }

    fn parse_escape_sequence(buf: &[u8], term: &Terminal) -> io::Result<Option<(usize, Event)>> {
        debug_assert!(buf[0] == b'\x1b');

        let keys = [Key::ArrowUp,
                    Key::ArrowDown,
                    Key::ArrowLeft,
                    Key::ArrowRight];

        for &key in keys.iter() {
            if let Some(keybytes) = term.key_bytes(key) {
                if buf.starts_with(keybytes) {
                    return Ok(Some((keybytes.len(), Event::Key(key))));
                }
            }
        }

        Ok(None)
    }
}
