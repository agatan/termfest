#![feature(io)]

extern crate term;
extern crate libc;
extern crate signal_notify;
extern crate unicode_width;
#[macro_use]
extern crate num_derive;
extern crate num;
#[macro_use]
extern crate bitflags;

use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::fs::{File, OpenOptions};
use std::ops::Drop;
use std::sync::{mpsc, Arc, Mutex, MutexGuard};

use std::os::unix::io::{RawFd, AsRawFd};

use signal_notify::{notify, Signal};

pub mod keys;
mod event;
mod screen;
use screen::Screen;
mod terminal;
use terminal::Terminal;
pub mod attr;

pub use keys::Key;
pub use event::Event;
pub use screen::Cell;
pub use attr::{Attribute, Color, Effect};

pub struct TermFest {
    ttyout_fd: RawFd,
    ttyout: Mutex<BufWriter<File>>,
    orig_tios: libc::termios,

    terminal: Arc<Terminal>,
    /// `screen` should be guraded with `Mutex` because SIGWINCH watcher thread will modify width
    /// and height of `screen`
    screen: Arc<Mutex<Screen>>,
}

impl TermFest {
    pub fn hold() -> Result<(TermFest, mpsc::Receiver<Event>), io::Error> {
        let mut ttyout = OpenOptions::new()
            .write(true)
            .read(false)
            .create(false)
            .open("/dev/tty")?;

        let orig_tios = setup_tios(ttyout.as_raw_fd())?;

        let terminal = Arc::new(Terminal::from_env()?);
        terminal.enter_ca(&mut ttyout)?;
        terminal.enter_keypad(&mut ttyout)?;
        terminal.clear(&mut ttyout)?;

        let (tx, rx) = mpsc::channel();

        spawn_ttyin_reader(tx.clone(), terminal.clone())?;

        let (width, height) = terminal::size(ttyout.as_raw_fd());
        let screen = Arc::new(Mutex::new(Screen::new(width, height)));
        {
            let ttyout_fd = ttyout.as_raw_fd();
            let screen = screen.clone();
            let tx = tx.clone();
            let sigwinch = notify(&[Signal::WINCH]);
            ::std::thread::spawn(move || loop {
                                     if sigwinch.recv().is_err() {
                                         break;
                                     }
                                     let (w, h) = terminal::size(ttyout_fd);
                                     let mut screen = screen.lock().unwrap();
                                     screen.resize(w, h);
                                     if tx.send(Event::Resize {
                                                    width: w,
                                                    height: h,
                                                })
                                            .is_err() {
                                         break;
                                     }
                                 });
        }

        let fest = TermFest {
            ttyout_fd: ttyout.as_raw_fd(),
            ttyout: Mutex::new(BufWriter::new(ttyout)),
            orig_tios: orig_tios,
            terminal: terminal,
            screen: screen,
        };
        Ok((fest, rx))
    }

    pub fn size(&self) -> (i32, i32) {
        terminal::size(self.ttyout_fd)
    }

    pub fn lock(&self) -> ScreenLock {
        ScreenLock {
            flushed: false,
            screen: self.screen.lock().unwrap(),
            ttyout_fd: self.ttyout_fd,
            ttyout: &self.ttyout,
            terminal: &self.terminal,
        }
    }
}

impl Drop for TermFest {
    fn drop(&mut self) {
        // ignore errors in drop
        if let Ok(mut ttyout) = self.ttyout.lock() {
            let _ = self.terminal.exit_keypad(&mut *ttyout);
            let _ = self.terminal.exit_ca(&mut *ttyout);
            let _ = self.terminal.reset_attr(&mut *ttyout);
            unsafe {
                libc::tcsetattr(self.ttyout_fd, libc::TCSANOW, &self.orig_tios);
            }
        }
    }
}

pub struct ScreenLock<'a> {
    flushed: bool,
    screen: MutexGuard<'a, Screen>,
    ttyout_fd: RawFd,
    ttyout: &'a Mutex<BufWriter<File>>,
    terminal: &'a Terminal,
}

impl<'a> ScreenLock<'a> {
    pub fn flush(&mut self) -> io::Result<()> {
        let mut ttyout = self.ttyout.lock().unwrap();
        for command in self.screen.flush_commands() {
            self.terminal.write(&mut *ttyout, command)?;
        }
        ttyout.flush()?;
        self.flushed = true;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.screen.clear();
        self.terminal
            .clear(&mut *self.ttyout.lock().unwrap())
            .unwrap()
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

    pub fn print(&mut self, x: i32, y: i32, s: &str, attr: Attribute) {
        self.screen.print(x, y, s, attr)
    }

    pub fn put_char(&mut self, x: i32, y: i32, cell: Cell) {
        self.screen.put_char(x, y, cell);
    }

    pub fn size(&self) -> (i32, i32) {
        terminal::size(self.ttyout_fd)
    }
}

impl<'a> Drop for ScreenLock<'a> {
    fn drop(&mut self) {
        if self.flushed {
            return;
        }
        let _ = self.flush();
        self.flushed = true;
    }
}

fn setup_tios(fd: ::libc::c_int) -> io::Result<libc::termios> {
    unsafe {
        let mut orig_tios: libc::termios = ::std::mem::uninitialized();
        if libc::tcgetattr(fd, &mut orig_tios as *mut _) < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut tios = orig_tios;
        tios.c_iflag &= !(libc::IGNBRK | libc::BRKINT | libc::PARMRK | libc::ISTRIP |
                          libc::INLCR | libc::IGNCR | libc::ICRNL |
                          libc::IXON);
        tios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
        tios.c_cflag &= !(libc::CSIZE | libc::PARENB);
        tios.c_cflag |= libc::CS8;
        tios.c_cc[libc::VMIN] = 1;
        tios.c_cc[libc::VTIME] = 0;
        if libc::tcsetattr(fd, libc::TCSANOW, &tios) < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(orig_tios)
    }
}

fn spawn_ttyin_reader(tx: mpsc::Sender<Event>, term: Arc<Terminal>) -> io::Result<()> {
    let mut ttyin = OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open("/dev/tty")?;
    unsafe {
        libc::fcntl(ttyin.as_raw_fd(),
                    libc::F_SETFL,
                    libc::O_ASYNC | libc::O_NONBLOCK);
    }
    let sigio = notify(&[Signal::IO]);
    ::std::thread::spawn(move || {
        let mut buf = Vec::new();
        for _ in sigio.iter() {
            let mut tmpbuf = [0; 64];
            match ttyin.read(&mut tmpbuf) {
                Ok(n) => buf.extend(&tmpbuf[..n]),
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock |
                        io::ErrorKind::InvalidInput => continue,
                        _ => panic!(e),
                    }
                }
            };
            let mut from = 0;
            loop {
                if let Some((read_byte, ev)) = event::Event::parse(&buf[from..], &*term).unwrap() {
                    from += read_byte;
                    if tx.send(ev).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
            buf = buf[from..].to_vec();
        }
    });
    Ok(())
}
