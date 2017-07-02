extern crate festerm;

use festerm::Event;
use festerm::keys::*;

fn main() {
    let (mut f, rx) = festerm::hold().unwrap();
    let (mut cursor_x, mut cursor_y) = (0, 0);

    for ev in rx.iter() {
        let mut screen = f.lock_screen();
        match ev {
            Event::Char('q') | Event::Key(ESC) => break,
            Event::Char(ch) => screen.put_char(cursor_x, cursor_y, ch),
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
