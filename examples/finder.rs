extern crate libc;
extern crate termfest;

use std::io::BufRead;
use std::rc::Rc;

use termfest::{TermFest, Event, ScreenLock, DisplayWidth, Cell};
use termfest::keys::*;
use termfest::attr::*;

#[derive(Default)]
struct Finder {
    needle: String,
    cursor: usize,
    candidates: Vec<Rc<String>>,
    matches: Vec<Rc<String>>,
    select: usize,
}

impl Finder {
    fn new(cs: Vec<String>) -> Self {
        let cs: Vec<_> = cs.into_iter().map(Rc::new).collect();
        Finder {
            needle: String::new(),
            cursor: 0,
            candidates: cs.clone(),
            matches: cs,
            select: 0,
        }
    }

    fn find(&mut self) {
        self.matches = self.candidates
            .iter()
            .filter(|s| s.contains(&self.needle))
            .cloned()
            .collect();
        if self.matches.is_empty() {
            self.select = 0;
        } else if self.select >= self.matches.len() {
            self.select = self.matches.len() - 1;
        }
    }

    fn insert(&mut self, ch: char) {
        self.needle.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
        self.find();
    }

    fn backspace(&mut self) {
        if let Some(ch) = self.needle[0..self.cursor].chars().rev().next() {
            self.cursor -= ch.len_utf8();
            self.needle.remove(self.cursor);
            self.find();
        }
    }

    fn left(&mut self) {
        if let Some(ch) = self.needle[0..self.cursor].chars().rev().next() {
            self.cursor -= ch.len_utf8();
        }
    }

    fn right(&mut self) {
        if let Some(ch) = self.needle[self.cursor..].chars().next() {
            self.cursor += ch.len_utf8();
        }
    }

    fn up(&mut self) {
        if self.select > 0 {
            self.select -= 1;
        }
    }

    fn down(&mut self) {
        if self.select < self.matches.len() - 1 {
            self.select += 1;
        }
    }

    fn get(&self) -> Rc<String> {
        self.matches[self.select].clone()
    }

    fn show_needle(&self, screen: &mut ScreenLock) {
        screen.print(0, 0, &self.needle, Attribute::default());
        let w = self.needle.display_width() as i32;
        let (width, _) = screen.size();
        for i in w..width {
            screen.put_cell(i, 0, Cell::new(' '));
        }
        let x = self.needle[..self.cursor].display_width() as i32;
        screen.move_cursor(x as i32, 0);
    }

    fn show_candidates(&self, screen: &mut ScreenLock) {
        for (i, m) in self.matches.iter().enumerate() {
            let attr = if i == self.select {
                Attribute {
                    effect: BOLD,
                    ..Attribute::default()
                }
            } else {
                Attribute::default()
            };
            if i == self.select {
                screen.print(0, i as i32 + 1, "> ", attr);
            } else {
                screen.print(0, i as i32 + 1, "  ", attr);
            }
            let (before, mat, after) = match m.find(&self.needle) {
                None => ("", m.as_str(), ""),
                Some(i) => (&m[..i], &m[i..i + self.needle.len()], &m[i + self.needle.len()..]),
            };
            screen.print(2, i as i32 + 1, before, attr);
            screen.print(2 + before.display_width() as i32,
                         i as i32 + 1,
                         mat,
                         Attribute {
                             fg: Color::Red,
                             ..attr
                         });
            screen.print(2 + before.display_width() as i32 + mat.display_width() as i32,
                         i as i32 + 1,
                         after,
                         attr);
        }
    }

    fn show(&self, fest: &TermFest) {
        let mut screen = fest.lock();
        screen.clear();
        self.show_needle(&mut screen);
        self.show_candidates(&mut screen);
    }
}

fn main() {
    let stdin = ::std::io::stdin();
    let candidates = stdin.lock().lines().collect::<Result<_, _>>().unwrap();
    let mut finder = Finder::new(candidates);
    let (fest, events) = TermFest::hold().unwrap();
    finder.show(&fest);

    let mut result = None;

    for ev in events.iter() {
        match ev {
            Event::Char(ch) => {
                finder.insert(ch);
            }
            Event::Key(ESC) => break,
            Event::Key(Backspace) => {
                finder.backspace();
            }
            Event::Key(ArrowLeft) |
            Event::Key(CtrlB) => {
                finder.left();
            }
            Event::Key(ArrowRight) |
            Event::Key(CtrlF) => {
                finder.right();
            }
            Event::Key(ArrowUp) |
            Event::Key(CtrlP) => {
                finder.up();
            }
            Event::Key(ArrowDown) |
            Event::Key(CtrlN) => {
                finder.down();
            }
            Event::Key(ENTER) => {
                result = Some(finder.get());
                break;
            }
            _ => {}
        }
        finder.show(&fest);
    }
    ::std::mem::drop(fest);
    if let Some(result) = result {
        println!("{}", result);
    }
}
