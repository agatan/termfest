//! termfest is a thread-safe TUI library that provides simple APIs to render texts in terminal,
//! heavily inspired by [nsf/termbox-go](https://github.com/nsf/termbox-go).
//! Currently, termfest doesn't support windows because of my poor windows experience.
//!
//! termfest has internal buffer for efficient rendering.
//! Applications can render everything to the buffer every time, and termfest flushes the buffer
//! and renders only the difference between the terminal state and the buffer.
//!
//! ## Example
//!
//! ```no_run
//! use termfest::{Termfest, Event};
//! use termfest::attr::*;
//! use termfest::key::*;
//!
//! // first, initialize termfest.
//! let (fest, events) = Termfest::hold().unwrap();
//!
//! let mut y = 0;
//!
//! // events is a receiver of a channel that accepts terminal events like key input.
//! for ev in events.iter() {
//!     {
//!         // lock the screen.
//!         let mut screen = fest.lock_screen();
//!         // clear the buffer. you can render everything every time.
//!         // termfest can provide efficient rendering.
//!         screen.clear();
//!         // write to the buffer.
//!         let attr = Attribute { fg: Color::Red, ..Attribute::default() };
//!         screen.print(0, y, "Hello, world!", attr);
//!         // when the screen lock is released, the buffer is flushed.
//!         // (you can flush the buffer with explicit `flush` call.)
//!     }
//!     match ev {
//!         Event::Key(ESC) | Event::Char('q') => break,
//!         Event::Key(ArrowUp) => if y > 0 { y -= 1; },
//!         Event::Key(ArrowDown) => y += 1,
//!         _ => {}
//!     }
//! }
//! ```

#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate num;
#[macro_use]
extern crate num_derive;
extern crate signal_notify;
extern crate term;
extern crate unicode_width;

use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::fs::{File, OpenOptions};
use std::ops::Drop;
use std::sync::{mpsc, Arc, Mutex, MutexGuard};

use std::os::unix::io::{AsRawFd, RawFd};

use signal_notify::{notify, Signal};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub mod key;
mod event;
mod screen;
use screen::Screen;
mod terminal;
use terminal::Terminal;
pub mod attr;

use key::Key;
pub use event::Event;
pub use screen::Cell;
use attr::Attribute;

/// `Termfest` holds termfest states.
/// It is created by `Termfest::hold`.
/// When it is dropped, termfest finalizes and restores every terminal states.
pub struct Termfest {
    ttyout_fd: RawFd,
    ttyout: Mutex<BufWriter<File>>,
    orig_tios: libc::termios,

    terminal: Arc<Terminal>,
    /// `screen` should be guraded with `Mutex` because SIGWINCH watcher thread will modify width
    /// and height of `screen`
    screen: Arc<Mutex<Screen>>,
}

impl Termfest {
    /// `hold` initialize terminal state and termfest state.
    /// If succeeded, it returns a tuple of `Termfest` object and `Receiver<Event>`.
    /// When the returned `Termfest` object is dropped, the terminal state will be restored.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), std::io::Error> {
    /// use termfest::Termfest;
    /// let (fest, events) = Termfest::hold()?;
    /// // do something widht fest and events.
    /// # Ok(())
    /// # }
    /// ```
    pub fn hold() -> Result<(Termfest, mpsc::Receiver<Event>), io::Error> {
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
                }).is_err()
                {
                    break;
                }
            });
        }

        let fest = Termfest {
            ttyout_fd: ttyout.as_raw_fd(),
            ttyout: Mutex::new(BufWriter::new(ttyout)),
            orig_tios: orig_tios,
            terminal: terminal,
            screen: screen,
        };
        Ok((fest, rx))
    }

    /// acquire the lock of screen, and returns `ScreenLock`.
    /// It will block if the lock is already acquired.
    pub fn lock_screen(&self) -> ScreenLock {
        ScreenLock {
            flushed: false,
            screen: self.screen.lock().unwrap(),
            ttyout: &self.ttyout,
            terminal: &self.terminal,
        }
    }
}

impl Drop for Termfest {
    fn drop(&mut self) {
        // ignore errors in drop
        if let Ok(mut ttyout) = self.ttyout.lock() {
            let _ = self.terminal.show_cursor(&mut *ttyout);
            let _ = self.terminal.exit_keypad(&mut *ttyout);
            let _ = self.terminal.exit_ca(&mut *ttyout);
            let _ = self.terminal.reset_attr(&mut *ttyout);
            unsafe {
                libc::tcsetattr(self.ttyout_fd, libc::TCSANOW, &self.orig_tios);
            }
        }
    }
}

/// `ScreenLock` is a locked screen buffer, created by `Termfest::lock_screen`.
/// When it is dropped, the buffered state will be flushed to the terminal.
/// All rendering manipulation is implemented in `ScreenLock`.
pub struct ScreenLock<'a> {
    flushed: bool,
    screen: MutexGuard<'a, Screen>,
    ttyout: &'a Mutex<BufWriter<File>>,
    terminal: &'a Terminal,
}

impl<'a> ScreenLock<'a> {
    /// flushes the internal buffer states to the terminal.
    /// Even if this function is not called, the buffer will be flushed when `self` is dropped.
    pub fn flush(&mut self) -> io::Result<()> {
        let mut ttyout = self.ttyout.lock().unwrap();
        for command in self.screen.flush_commands() {
            self.terminal.write(&mut *ttyout, command)?;
        }
        ttyout.flush()?;
        self.flushed = true;
        Ok(())
    }

    /// clear the internal buffer states.
    /// If clear and flush is called, the terminal will be cleared (nothing will be rendered).
    pub fn clear(&mut self) {
        self.screen.clear();
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        self.screen.cursor.x = x;
        self.screen.cursor.y = y;
    }

    pub fn hide_cursor(&mut self) {
        self.screen.cursor.visible = false;
    }

    pub fn show_cursor(&mut self) {
        self.screen.cursor.visible = true;
    }

    /// print string with the given attribute.
    /// It is equal to `put_cell` calls with each character.
    pub fn print(&mut self, x: usize, y: usize, s: &str, attr: Attribute) {
        self.screen.print(x, y, s, attr)
    }

    pub fn put_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.screen.put_cell(x, y, cell);
    }

    /// returns the width and height of the terminal.
    pub fn size(&self) -> (usize, usize) {
        self.screen.size()
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
        tios.c_iflag &= !(libc::IGNBRK | libc::BRKINT | libc::PARMRK | libc::ISTRIP | libc::INLCR
            | libc::IGNCR | libc::ICRNL | libc::IXON);
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
        let r = libc::fcntl(
            ttyin.as_raw_fd(),
            libc::F_SETFL,
            libc::O_ASYNC | libc::O_NONBLOCK,
        );
        if r < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    if cfg!(linux) {
        unsafe {
            let r = libc::fcntl(ttyin.as_raw_fd(), libc::F_SETOWN, libc::getpid());
            if r < 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    let sigio = notify(&[Signal::IO]);
    ::std::thread::spawn(move || {
        let mut buf = Vec::new();
        for _ in sigio.iter() {
            let mut tmpbuf = [0; 64];
            match ttyin.read(&mut tmpbuf) {
                Ok(n) => buf.extend(&tmpbuf[..n]),
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock | io::ErrorKind::InvalidInput => continue,
                    _ => panic!("failed to read from tty: {}", e),
                },
            };
            let mut from = 0;
            loop {
                if let Some((read_byte, ev)) = event::parse(&buf[from..], &*term) {
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

/// `DisplayWidth` provides a way to determine display width of characters or strings.
///
/// ```
/// use termfest::DisplayWidth;
///
/// assert_eq!('あ'.display_width(), 2);
/// assert_eq!('a'.display_width(), 1);
///
/// assert_eq!("abc".display_width(), 3);
/// assert_eq!("あいう".display_width(), 6);
/// ```
pub trait DisplayWidth {
    fn display_width(&self) -> usize;
}

impl DisplayWidth for char {
    fn display_width(&self) -> usize {
        self.width().unwrap_or(1)
    }
}

impl DisplayWidth for str {
    fn display_width(&self) -> usize {
        self.width()
    }
}
