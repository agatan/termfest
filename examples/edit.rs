extern crate termfest;
extern crate unicode_width;

use std::collections::VecDeque;

use unicode_width::UnicodeWidthChar;

use termfest::{TermFest, Event, Cell};
use termfest::keys::*;

struct Editor {
    before_cursor: VecDeque<char>,
    after_cursor: VecDeque<char>,
    termfest: TermFest,
    cursor: i32,
}

impl Editor {
    fn new(fest: TermFest) -> Self {
        Editor {
            before_cursor: VecDeque::new(),
            after_cursor: VecDeque::new(),
            termfest: fest,
            cursor: 0,
        }
    }

    fn insert(&mut self, ch: char) {
        self.before_cursor.push_back(ch);
        let mut screen = self.termfest.lock();
        screen.put_char(self.cursor, 0, Cell::new(ch));
        self.cursor += ch.width().unwrap_or(1) as i32;
        let mut x = self.cursor;
        for &ch in self.after_cursor.iter() {
            screen.put_char(x, 0, Cell::new(ch));
            x += ch.width().unwrap_or(1) as i32;
        }
        screen.move_cursor(self.cursor, 0);
    }

    fn backspace(&mut self) {
        if let Some(ch) = self.before_cursor.pop_back() {
            let mut screen = self.termfest.lock();
            self.cursor -= ch.width().unwrap_or(1) as i32;
            let mut x = self.cursor;
            for &ch in self.after_cursor.iter() {
                screen.put_char(x, 0, Cell::new(ch));
                x += ch.width().unwrap_or(1) as i32;
            }
            screen.put_char(x, 0, Cell::new(' '));
            screen.put_char(x + 1, 0, Cell::new(' '));
            screen.move_cursor(self.cursor, 0);
        }
    }

    fn move_left(&mut self) {
        if let Some(ch) = self.before_cursor.pop_back() {
            self.after_cursor.push_front(ch);
            self.cursor -= ch.width().unwrap_or(1) as i32;
            self.termfest.lock().move_cursor(self.cursor, 0);
        }
    }

    fn move_right(&mut self) {
        if let Some(ch) = self.after_cursor.pop_front() {
            self.before_cursor.push_back(ch);
            self.cursor += ch.width().unwrap_or(1) as i32;
            self.termfest.lock().move_cursor(self.cursor, 0);
        }
    }
}

fn main() {
    let (fest, rx) = TermFest::hold().unwrap();
    let mut editor = Editor::new(fest);

    for ev in rx.iter() {
        match ev {
            Event::Char(ch) => {
                editor.insert(ch);
            }
            Event::Key(key) => {
                match key {
                    ArrowLeft | CtrlB => editor.move_left(),
                    ArrowRight | CtrlF => editor.move_right(),
                    CtrlH | Backspace => editor.backspace(),
                    ESC => break,
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
