extern crate festerm;
extern crate unicode_width;

use std::collections::VecDeque;

use unicode_width::UnicodeWidthChar;

use festerm::{Festerm, Event};
use festerm::keys::*;

struct Editor {
    before_cursor: VecDeque<char>,
    after_cursor: VecDeque<char>,
    festerm: Festerm,
    cursor: i32,
}

impl Editor {
    fn new(fest: Festerm) -> Self {
        Editor {
            before_cursor: VecDeque::new(),
            after_cursor: VecDeque::new(),
            festerm: fest,
            cursor: 0,
        }
    }

    fn insert(&mut self, ch: char) {
        self.before_cursor.push_back(ch);
        let mut screen = self.festerm.lock_screen();
        screen.put_char(self.cursor, 0, ch);
        self.cursor += ch.width_cjk().unwrap_or(1) as i32;
        let mut x = self.cursor;
        for &ch in self.after_cursor.iter() {
            screen.put_char(x, 0, ch);
            x += ch.width_cjk().unwrap_or(1) as i32;
        }
        screen.move_cursor(self.cursor, 0);
    }

    fn backspace(&mut self) {
        if let Some(ch) = self.before_cursor.pop_back() {
            let mut screen = self.festerm.lock_screen();
            self.cursor -= ch.width_cjk().unwrap_or(1) as i32;
            let mut x = self.cursor;
            for &ch in self.after_cursor.iter() {
                screen.put_char(x, 0, ch);
                x += ch.width_cjk().unwrap_or(1) as i32;
            }
            screen.put_char(x, 0, ' ');
            screen.move_cursor(self.cursor, 0);
        }
    }

    fn move_left(&mut self) {
        if let Some(ch) = self.before_cursor.pop_back() {
            self.after_cursor.push_front(ch);
            self.cursor -= ch.width_cjk().unwrap_or(1) as i32;
            self.festerm.lock_screen().move_cursor(self.cursor, 0);
        }
    }

    fn move_right(&mut self) {
        if let Some(ch) = self.after_cursor.pop_front() {
            self.before_cursor.push_back(ch);
            self.cursor += ch.width_cjk().unwrap_or(1) as i32;
            self.festerm.lock_screen().move_cursor(self.cursor, 0);
        }
    }
}

fn main() {
    let (fest, rx) = festerm::hold().unwrap();
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
