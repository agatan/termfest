use std::io::{self, Write};

use term::terminfo::TermInfo;
use libc;

use keys::Key;
use attr::{Color, Effect, BOLD, DIM, UNDERLINE, BLINK, REVERSE};

#[derive(Debug)]
pub struct Terminal {
    terminfo: TermInfo,
}

impl Terminal {
    pub fn from_env() -> io::Result<Self> {
        let terminfo = TermInfo::from_env()?;
        Ok(Terminal { terminfo: terminfo })
    }

    fn write_if_exists<W: Write>(&self, mut w: W, typ: &str) -> io::Result<()> {
        if let Some(bytes) = self.terminfo.strings.get(typ) {
            w.write_all(bytes)
        } else {
            Ok(())
        }
    }

    pub fn enter_ca<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "smcup")
    }

    pub fn exit_ca<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "rmcup")
    }

    pub fn enter_keypad<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "smkx")
    }

    pub fn exit_keypad<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "rmkx")
    }

    pub fn clear<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "clear")
    }

    pub fn move_cursor<W: Write>(&self, mut w: W, x: i32, y: i32) -> io::Result<()> {
        w.write(&[0x1b])?;
        write!(w, "[{};{}H", y + 1, x + 1)
    }

    pub fn hide_cursor<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "civis")
    }

    pub fn show_cursor<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "cnorm")
    }

    pub fn put_char<W: Write>(&self, mut w: W, ch: char) -> io::Result<()> {
        let mut buf = [0; 4];
        w.write_all(ch.encode_utf8(&mut buf).as_bytes())
    }

    pub fn reset_attr<W: Write>(&self, w: W) -> io::Result<()> {
        self.write_if_exists(w, "sgr0")
    }

    pub fn fg<W: Write>(&self, mut w: W, color: Color) -> io::Result<()> {
        let bytes = match color {
            Color::Default => "\u{1b}[39m",
            Color::Black => "\u{1b}[30m",
            Color::Red => "\u{1b}[31m",
            Color::Green => "\u{1b}[32m",
            Color::Yellow => "\u{1b}[33m",
            Color::Blue => "\u{1b}[34m",
            Color::Magenta => "\u{1b}[35m",
            Color::Cyan => "\u{1b}[36m",
            Color::White => "\u{1b}[37m",
        };
        w.write_all(bytes.as_bytes())
    }

    pub fn bg<W: Write>(&self, mut w: W, color: Color) -> io::Result<()> {
        let bytes = match color {
            Color::Default => "\u{1b}[49m",
            Color::Black => "\u{1b}[40m",
            Color::Red => "\u{1b}[41m",
            Color::Green => "\u{1b}[42m",
            Color::Yellow => "\u{1b}[43m",
            Color::Blue => "\u{1b}[44m",
            Color::Magenta => "\u{1b}[45m",
            Color::Cyan => "\u{1b}[46m",
            Color::White => "\u{1b}[47m",
        };
        w.write_all(bytes.as_bytes())
    }

    pub fn effect<W: Write>(&self, mut w: W, effect: Effect) -> io::Result<()> {
        if effect.contains(BOLD) {
            self.write_if_exists(&mut w, "bold")?;
        }
        if effect.contains(DIM) {
            self.write_if_exists(&mut w, "dim")?;
        }
        if effect.contains(UNDERLINE) {
            self.write_if_exists(&mut w, "smul")?;
        }
        if effect.contains(BLINK) {
            self.write_if_exists(&mut w, "blink")?;
        }
        if effect.contains(REVERSE) {
            self.write_if_exists(&mut w, "rev")?;
        }
        Ok(())
    }

    pub fn write<W: Write>(&self, w: W, command: Command) -> io::Result<()> {
        match command {
            Command::HideCursor => self.hide_cursor(w),
            Command::ShowCursor => self.show_cursor(w),
            Command::MoveCursor { x, y } => self.move_cursor(w, x, y),
            Command::PutChar(ch) => self.put_char(w, ch),
            Command::ResetAttr => self.reset_attr(w),
            Command::Fg(c) => self.fg(w, c),
            Command::Bg(c) => self.bg(w, c),
            Command::Effect(a) => self.effect(w, a),
        }
    }

    pub fn escaped_key_bytes(&self, key: Key) -> Option<&Vec<u8>> {
        match key {
            Key::ArrowUp => self.terminfo.strings.get("kcuu1"),
            Key::ArrowDown => self.terminfo.strings.get("kcud1"),
            Key::ArrowLeft => self.terminfo.strings.get("kcub1"),
            Key::ArrowRight => self.terminfo.strings.get("kcuf1"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    HideCursor,
    ShowCursor,
    MoveCursor { x: i32, y: i32 },
    PutChar(char),
    ResetAttr,
    Fg(Color),
    Bg(Color),
    Effect(Effect),
}

pub fn size(fd: libc::c_int) -> (i32, i32) {
    unsafe {
        let mut wsz: libc::winsize = ::std::mem::uninitialized();
        let n = libc::ioctl(fd, libc::TIOCGWINSZ, &mut wsz as *mut _);
        if n < 0 {
            libc::perror("get window size".as_ptr() as *const _);
            panic!();
        }
        (wsz.ws_col as i32, wsz.ws_row as i32)
    }
}
