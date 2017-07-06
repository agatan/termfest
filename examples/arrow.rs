extern crate termfest;

use termfest::{Event, TermFest, Cell};
use termfest::key::*;

fn main() {
    let (f, rx) = TermFest::hold().unwrap();
    let (mut cursor_x, mut cursor_y) = (0, 0);

    for ev in rx.iter() {
        let mut screen = f.lock();
        match ev {
            Event::Char('q') | Event::Key(ESC) => break,
            Event::Char(ch) => screen.put_cell(cursor_x, cursor_y, Cell::new(ch)),
            Event::Key(key) => {
                match key {
                    ArrowUp | CtrlP => {
                        cursor_y -= 1;
                        screen.move_cursor(cursor_x, cursor_y);
                    }
                    ArrowDown | CtrlN => {
                        cursor_y += 1;
                        screen.move_cursor(cursor_x, cursor_y);
                    }
                    ArrowLeft | CtrlB => {
                        cursor_x -= 1;
                        screen.move_cursor(cursor_x, cursor_y);
                    }
                    ArrowRight | CtrlF => {
                        cursor_x += 1;
                        screen.move_cursor(cursor_x, cursor_y);
                    }
                    _ => {}
                }
            }
            Event::Resize { .. } => {}
        }
    }
}
