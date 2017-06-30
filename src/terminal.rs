use std::io::{self, Write};
use term::terminfo::TermInfo;

#[derive(Debug)]
pub struct Terminal {
    terminfo: TermInfo,
}

impl Terminal {
    pub fn from_env() -> io::Result<Self> {
        let terminfo = TermInfo::from_env()?;
        Ok(Terminal { terminfo: terminfo })
    }

    pub fn enter_ca<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.terminfo.strings["smcup"])
    }

    pub fn exit_ca<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.terminfo.strings["rmcup"])
    }

    pub fn clear<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.terminfo.strings["clear"])
    }

    pub fn move_cursor<W: Write>(&self, mut w: W, x: i32, y: i32) -> io::Result<()> {
        w.write(&[0x1b])?;
        write!(w, "[{};{}H", y + 1, x + 1)
    }

    pub fn hide_cursor<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.terminfo.strings["civis"])
    }

    pub fn show_cursor<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.terminfo.strings["cnorm"])
    }

    pub fn put_char<W: Write>(&self, mut w: W, ch: char) -> io::Result<()> {
        let mut buf = [0; 4];
        w.write_all(ch.encode_utf8(&mut buf).as_bytes())
    }

    pub fn write<W: Write>(&self, w: W, command: Command) -> io::Result<()> {
        match command {
            Command::HideCursor => self.hide_cursor(w),
            Command::ShowCursor => self.show_cursor(w),
            Command::MoveCursor { x, y } => self.move_cursor(w, x, y),
            Command::PutChar(ch) => self.put_char(w, ch),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    HideCursor,
    ShowCursor,
    MoveCursor { x: i32, y: i32 },
    PutChar(char),
}
