#![feature(io)]

extern crate nix;
extern crate term;

use std::io::prelude::*;
use std::io;
use std::fs::{File, OpenOptions};
use std::ops::Drop;
use std::sync::mpsc;

use std::os::unix::io::AsRawFd;

use nix::sys::termios;
use term::terminfo::TermInfo;

mod event;
pub use event::{Event, Key};

mod buffer;
use buffer::Buffer;
mod terminal;
use terminal::Terminal;

struct Cursor {
    x: isize,
    y: isize,
}

pub struct Festival {
    ttyout: File,
    buffer: Buffer,
    orig_tios: termios::Termios,
    cursor: Option<Cursor>,
}

pub fn hold() -> Result<(Festival, mpsc::Receiver<Event>), io::Error> {
    let mut ttyin = OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open("/dev/tty")?;
    let (tx, rx) = mpsc::channel();
    ::std::thread::spawn(move || loop {
                             let ev = event::Event::parse(&mut ttyin).unwrap();
                             if tx.send(ev).is_err() {
                                 break;
                             }
                         });
    Festival::new().map(|fest| (fest, rx))
}

impl Festival {
    fn new() -> Result<Self, io::Error> {
        let mut ttyout = OpenOptions::new()
            .write(true)
            .read(false)
            .create(false)
            .open("/dev/tty")?;

        let orig_tios = termios::tcgetattr(ttyout.as_raw_fd())?;
        let mut tios = orig_tios;
        tios.c_iflag &=
            !(termios::IGNBRK | termios::BRKINT | termios::PARMRK | termios::ISTRIP |
              termios::INLCR | termios::IGNCR | termios::ICRNL | termios::IXON);
        tios.c_lflag &= !(termios::ECHO | termios::ECHONL | termios::ICANON | termios::ISIG |
                          termios::IEXTEN);
        tios.c_cflag &= !(termios::CSIZE | termios::PARENB);
        tios.c_cflag |= termios::CS8;
        tios.c_cc[termios::VMIN] = 1;
        tios.c_cc[termios::VTIME] = 0;
        termios::tcsetattr(ttyout.as_raw_fd(), termios::SetArg::TCSANOW, &tios)?;

        let terminal = Terminal::from_env()?;
        terminal.enter_ca(&mut ttyout)?;
        let buffer = Buffer::new(terminal, 80, 50);

        let mut fest = Festival {
            ttyout: ttyout,
            buffer: buffer,
            orig_tios: orig_tios,
            cursor: None,
        };

        Ok(fest)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush(&mut self.ttyout)
    }

    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    pub fn move_cursor(&mut self, x: i32, y: i32) {
        self.buffer.move_cursor(x, y)
    }

    pub fn hide_cursor(&mut self) {
        self.buffer.hide_cursor()
    }

    pub fn show_cursor(&mut self) {
        self.buffer.show_cursor()
    }

    pub fn put_char(&mut self, x: i32, y: i32, ch: char) {
        self.buffer.put_char(x, y, ch)
    }
}

impl Drop for Festival {
    fn drop(&mut self) {
        self.buffer.terminal.exit_ca(&mut self.ttyout).unwrap();
        termios::tcsetattr(self.ttyout.as_raw_fd(),
                           termios::SetArg::TCSANOW,
                           &self.orig_tios)
                .unwrap();
    }
}
