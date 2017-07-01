use std::io::{self, Read};

#[derive(Debug, Clone)]
pub enum Event {
    Key(Key),
    Resize { width: i32, height: i32 },
}

#[derive(Debug, Clone)]
pub enum Key {
    Char(char),
}

impl Event {
    pub fn parse<R: Read>(buf: R) -> io::Result<Event> {
        let ch = match buf.chars().next().unwrap() {
            Ok(ch) => ch,
            Err(io::CharsError::NotUtf8) => panic!("not utf8"),
            Err(io::CharsError::Other(err)) => return Err(err.into()),
        };
        Ok(Event::Key(Key::Char(ch)))
    }
}
