extern crate festerm;
extern crate unicode_width;

use std::collections::VecDeque;

use unicode_width::UnicodeWidthChar;

use festerm::{Festival, Event};
use festerm::keys::*;

#[derive(Default)]
struct Editor {
    before_cursor: VecDeque<char>,
    after_cursor: VecDeque<char>,
}

impl Editor {
    fn insert(&mut self, ch: char) {
        self.before_cursor.push_back(ch);
    }

    fn backspace(&mut self) {
        self.before_cursor.pop_back();
    }

    fn move_left(&mut self) {
        if let Some(ch) = self.before_cursor.pop_back() {
            self.after_cursor.push_front(ch);
        }
    }

    fn move_right(&mut self) {
        if let Some(ch) = self.after_cursor.pop_front() {
            self.before_cursor.push_back(ch);
        }
    }

    fn show(&self, fest: &mut Festival) {
        fest.clear();
        fest.print(0,
                   0,
                   self.before_cursor
                       .iter()
                       .chain(self.after_cursor.iter())
                       .cloned()
                       .collect::<String>()
                       .as_str());
        fest.move_cursor(self.before_cursor
                             .iter()
                             .map(|ch| ch.width_cjk().unwrap_or(1) as i32)
                             .sum(),
                         0);
        fest.flush().unwrap();
    }
}

fn main() {
    let mut editor = Editor::default();
    let (mut fest, rx) = festerm::hold().unwrap();

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
        editor.show(&mut fest);
    }
}
