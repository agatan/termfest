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

struct Cursor {
    x: isize,
    y: isize,
}

pub struct Festival {
    ttyout: File,
    terminfo: TermInfo,
    orig_tios: termios::Termios,
    cursor: Option<Cursor>,
}

pub fn hold() -> Result<(Festival, mpsc::Receiver<Vec<u8>>), io::Error> {
    let mut ttyin = OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open("/dev/tty")?;
    let (tx, rx) = mpsc::channel();
    ::std::thread::spawn(move || {
        let mut buf = vec![0; 128];
        loop {
            match ttyin.read(&mut buf) {
                Ok(n) => tx.send(buf.iter().take(n).cloned().collect()).unwrap(),
                Err(e) => panic!(e),
            }
        }
    });
    Festival::new().map(|fest| (fest, rx))
}

impl Festival {
    fn new() -> Result<Self, io::Error> {
        let ttyout = OpenOptions::new()
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

        let terminfo = TermInfo::from_env()?;

        let mut fest = Festival {
            ttyout: ttyout,
            terminfo: terminfo,
            orig_tios: orig_tios,
            cursor: None,
        };

        fest.enter_ca()?;

        Ok(fest)
    }

    fn enter_ca(&mut self) -> Result<(), io::Error> {
        let s = &self.terminfo.strings["smcup"];
        self.ttyout.write_all(&s)
    }

    fn exit_ca(&mut self) -> Result<(), io::Error> {
        let s = &self.terminfo.strings["rmcup"];
        self.ttyout.write_all(&s)
    }

    pub fn clear(&mut self) -> Result<(), io::Error> {
        self.ttyout.write_all(&self.terminfo.strings["clear"])
    }

    pub fn set_cursor(&mut self, x: isize, y: isize) -> Result<(), io::Error> {
        if self.cursor.is_none() {
            self.show_cursor()?;
        }
        self.cursor = Some(Cursor { x: x, y: y });
        self.ttyout.write(&[0x1b])?;
        write!(self.ttyout, "[{};{}H", y + 1, x + 1)
    }

    fn show_cursor(&mut self) -> Result<(), io::Error> {
        self.ttyout.write_all(&self.terminfo.strings["cnorm"])
    }

    pub fn hide_cursor(&mut self) -> Result<(), io::Error> {
        self.cursor = None;
        self.ttyout.write_all(&self.terminfo.strings["civis"])
    }

    pub fn putchar(&mut self, ch: char) -> Result<(), io::Error> {
        let mut buf = [0; 4];
        self.ttyout.write_all(ch.encode_utf8(&mut buf).as_bytes())
    }

    pub fn putbyte(&mut self, byte: u8) -> Result<(), io::Error> {
        self.ttyout.write_all(&[byte])
    }
}

impl Drop for Festival {
    fn drop(&mut self) {
        self.exit_ca().unwrap();
        termios::tcsetattr(self.ttyout.as_raw_fd(),
                           termios::SetArg::TCSANOW,
                           &self.orig_tios)
                .unwrap();
    }
}
