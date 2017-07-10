extern crate termfest;
extern crate unicode_width;

use termfest::{Termfest, Event, DisplayWidth, Attribute};
use termfest::key::*;

struct Editor {
    contents: String,
    cursor: usize,
}

impl Editor {
    fn new() -> Self {
        Editor {
            contents: String::new(),
            cursor: 0,
        }
    }

    fn insert(&mut self, ch: char) {
        self.contents.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    fn backspace(&mut self) {
        if let Some(ch) = self.contents[..self.cursor].chars().rev().next() {
            self.cursor -= ch.len_utf8();
            self.contents.remove(self.cursor);
        }
    }

    fn move_left(&mut self) {
        if let Some(ch) = self.contents[..self.cursor].chars().rev().next() {
            self.cursor -= ch.len_utf8();
        }
    }

    fn move_right(&mut self) {
        if let Some(ch) = self.contents[self.cursor..].chars().next() {
            self.cursor += ch.len_utf8();
        }
    }

    fn show(&self, fest: &Termfest) {
        let mut screen = fest.lock_screen();
        screen.clear();
        screen.print(0, 0, &self.contents, Attribute::default());
        let cursor_x = self.contents[..self.cursor].display_width();
        screen.move_cursor(cursor_x, 0);
    }
}

fn main() {
    let (fest, rx) = Termfest::hold().unwrap();
    let mut editor = Editor::new();
    editor.show(&fest);

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
        editor.show(&fest);
    }
}
