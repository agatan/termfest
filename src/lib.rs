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

mod screen;
use screen::Screen;
mod terminal;
use terminal::Terminal;

pub struct Festival {
    ttyout: File,
    orig_tios: termios::Termios,

    terminal: Terminal,
    screen: Screen,
    write_buffer: Vec<u8>,
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
        terminal.clear(&mut ttyout)?;

        let mut fest = Festival {
            ttyout: ttyout,
            orig_tios: orig_tios,

            terminal: terminal,
            screen: Screen::new(80, 50),
            write_buffer: Vec::new(),
        };

        Ok(fest)
    }

    fn flush_to_buffer(&mut self) -> io::Result<()> {
        for command in self.screen.flush_commands() {
            self.terminal.write(&mut self.write_buffer, command)?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.flush_to_buffer()?;
        self.ttyout.write_all(&self.write_buffer)?;
        self.write_buffer.truncate(0);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.terminal.clear(&mut self.write_buffer).unwrap()
    }

    pub fn move_cursor(&mut self, x: i32, y: i32) {
        self.screen.cursor.x = x;
        self.screen.cursor.y = y;
    }

    pub fn hide_cursor(&mut self) {
        self.screen.cursor.visible = false;
    }

    pub fn show_cursor(&mut self) {
        self.screen.cursor.visible = true;
    }

    pub fn put_char(&mut self, x: i32, y: i32, ch: char) {
        if let Some(cell) = self.screen.cell_mut(x, y) {
            cell.ch = Some(ch);
        }
    }
}

impl Drop for Festival {
    fn drop(&mut self) {
        self.terminal.exit_ca(&mut self.ttyout).unwrap();
        termios::tcsetattr(self.ttyout.as_raw_fd(),
                           termios::SetArg::TCSANOW,
                           &self.orig_tios)
                .unwrap();
    }
}
