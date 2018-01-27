extern crate termfest;

use termfest::{Cell, Event, Termfest};
use termfest::key::*;

use std::cmp;

fn main() {
    let (f, rx) = Termfest::hold().unwrap();
    let (mut cursor_x, mut cursor_y) = (0, 0);
    let (mut width, mut height) = f.lock_screen().size();

    for ev in rx.iter() {
        let mut screen = f.lock_screen();
        match ev {
            Event::Char('q') | Event::Key(ESC) => break,
            Event::Char(ch) => screen.put_cell(cursor_x, cursor_y, Cell::new(ch)),
            Event::Key(key) => match key {
                ArrowUp | CtrlP => {
                    if cursor_y > 0 {
                        cursor_y -= 1;
                    }
                    screen.move_cursor(cursor_x, cursor_y);
                }
                ArrowDown | CtrlN => {
                    if cursor_y < height - 1 {
                        cursor_y += 1;
                    }
                    screen.move_cursor(cursor_x, cursor_y);
                }
                ArrowLeft | CtrlB => {
                    if cursor_x > 0 {
                        cursor_x -= 1;
                    }
                    screen.move_cursor(cursor_x, cursor_y);
                }
                ArrowRight | CtrlF => {
                    if cursor_x < width - 1 {
                        cursor_x += 1;
                    }
                    screen.move_cursor(cursor_x, cursor_y);
                }
                _ => {}
            },
            Event::Resize {
                width: w,
                height: h,
            } => {
                width = w;
                height = h;
                cursor_y = cmp::min(cursor_y, height - 1);
                cursor_x = cmp::min(cursor_x, width - 1);
            }
        }
    }
}
