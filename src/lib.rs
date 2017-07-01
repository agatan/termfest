#![feature(io)]

extern crate nix;
extern crate term;
extern crate libc;
extern crate signal_notify;

use std::io::prelude::*;
use std::io;
use std::fs::{File, OpenOptions};
use std::ops::Drop;
use std::sync::{mpsc, Arc, Mutex};

use std::os::unix::io::AsRawFd;

use nix::sys::termios;
use signal_notify::{notify, Signal};

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
    /// `screen` should be guraded with `Mutex` because SIGWINCH watcher thread will modify width
    /// and height of `screen`
    screen: Arc<Mutex<Screen>>,
    write_buffer: Vec<u8>,
}

pub fn hold() -> Result<(Festival, mpsc::Receiver<Event>), io::Error> {
    let mut ttyout = OpenOptions::new()
        .write(true)
        .read(false)
        .create(false)
        .open("/dev/tty")?;

    let orig_tios = setup_tios(ttyout.as_raw_fd())?;

    let terminal = Terminal::from_env()?;
    terminal.enter_ca(&mut ttyout)?;
    terminal.clear(&mut ttyout)?;

    let (tx, rx) = mpsc::channel();

    spawn_ttyin_reader(tx.clone())?;

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

    let fest = Festival {
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

fn spawn_ttyin_reader(tx: mpsc::Sender<Event>) -> io::Result<()> {
    let mut ttyin = OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open("/dev/tty")?;
    ::std::thread::spawn(move || loop {
                             let ev = event::Event::parse(&mut ttyin).unwrap();
                             if tx.send(ev).is_err() {
                                 break;
                             }
                         });
    Ok(())
}

impl Festival {
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

    pub fn put_char(&mut self, x: i32, y: i32, ch: char) {
        self.screen.lock().unwrap().put_char(x, y, ch);
    }

    pub fn size(&self) -> (i32, i32) {
        terminal::size(self.ttyout.as_raw_fd())
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
