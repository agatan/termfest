use std::io::{self, Read};

#[derive(Debug, Clone)]
pub enum Event {
    Key(Key),
    Resize { width: i32, height: i32 },
}

#[derive(Debug, Clone)]
pub enum Key {
    Char(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

impl Event {
    pub fn parse(buf: &[u8]) -> io::Result<Option<(usize, Event)>> {
        let ch = match buf.chars().next() {
            None => return Ok(None),
            Some(Ok(ch)) => ch,
            Some(Err(io::CharsError::NotUtf8)) => return Ok(None),
            Some(Err(io::CharsError::Other(err))) => return Err(err.into()),
        };
        Ok(Some((ch.len_utf8(), Event::Key(Key::Char(ch)))))
    }
}
