#![feature(io)]

extern crate nix;
extern crate term;
extern crate libc;
extern crate signal_notify;
extern crate unicode_width;
#[macro_use]
extern crate num_derive;
extern crate num;

use std::io::prelude::*;
use std::io;
use std::fs::{File, OpenOptions};
use std::ops::Drop;
use std::sync::{mpsc, Arc, Mutex};

use std::os::unix::io::AsRawFd;

use nix::sys::termios;
use signal_notify::{notify, Signal};

pub mod keys;
pub use keys::Key;
mod event;
pub use event::Event;
mod screen;
use screen::Screen;
mod terminal;
use terminal::Terminal;

pub struct Festerm {
    ttyout: File,
    orig_tios: termios::Termios,

    terminal: Arc<Terminal>,
    /// `screen` should be guraded with `Mutex` because SIGWINCH watcher thread will modify width
    /// and height of `screen`
    screen: Arc<Mutex<Screen>>,
    write_buffer: Vec<u8>,
}

pub fn hold() -> Result<(Festerm, mpsc::Receiver<Event>), io::Error> {
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

    let fest = Festerm {
        ttyout: ttyout,
        orig_tios: orig_tios,
        terminal: terminal,
        screen: screen,
        write_buffer: Vec::new(),
    };
    Ok((fest, rx))
}

fn setup_tios(fd: ::libc::c_int) -> io::Result<termios::Termios> {
    let orig_tios = termios::tcgetattr(fd)?;
    let mut tios = orig_tios;
    tios.c_iflag &= !(termios::IGNBRK | termios::BRKINT | termios::PARMRK | termios::ISTRIP |
                      termios::INLCR |
                      termios::IGNCR | termios::ICRNL | termios::IXON);
    tios.c_lflag &= !(termios::ECHO | termios::ECHONL | termios::ICANON | termios::ISIG |
                      termios::IEXTEN);
    tios.c_cflag &= !(termios::CSIZE | termios::PARENB);
    tios.c_cflag |= termios::CS8;
    tios.c_cc[termios::VMIN] = 1;
    tios.c_cc[termios::VTIME] = 0;
    termios::tcsetattr(fd, termios::SetArg::TCSANOW, &tios)?;
    Ok(orig_tios)
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

impl Festerm {
    fn flush_to_buffer(&mut self) -> io::Result<()> {
        for command in self.screen.lock().unwrap().flush_commands() {
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
        self.screen.lock().unwrap().clear();
        self.terminal.clear(&mut self.write_buffer).unwrap()
    }

    pub fn move_cursor(&mut self, x: i32, y: i32) {
        let mut screen = self.screen.lock().unwrap();
        screen.cursor.x = x;
        screen.cursor.y = y;
    }

    pub fn hide_cursor(&mut self) {
        self.screen.lock().unwrap().cursor.visible = false;
    }

    pub fn show_cursor(&mut self) {
        self.screen.lock().unwrap().cursor.visible = true;
    }

    pub fn print(&mut self, x: i32, y: i32, s: &str) {
        self.screen.lock().unwrap().print(x, y, s)
    }

    pub fn put_char(&mut self, x: i32, y: i32, ch: char) {
        self.screen.lock().unwrap().put_char(x, y, ch);
    }

    pub fn size(&self) -> (i32, i32) {
        terminal::size(self.ttyout.as_raw_fd())
    }
}

impl Drop for Festerm {
    fn drop(&mut self) {
        self.terminal.exit_keypad(&mut self.ttyout).unwrap();
        self.terminal.exit_ca(&mut self.ttyout).unwrap();
        termios::tcsetattr(self.ttyout.as_raw_fd(),
                           termios::SetArg::TCSANOW,
                           &self.orig_tios)
                .unwrap();
    }
}
