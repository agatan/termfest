extern crate festerm;

use festerm::Event;
use festerm::keys::*;

fn main() {
    let (mut f, rx) = festerm::hold().unwrap();
    let (mut cursor_x, mut cursor_y) = (0, 0);

    loop {
        let ev = rx.recv().unwrap();
        match ev {
            Event::Char(ch) => {
                match ch {
                    'q' => break,
                    ch => f.put_char(cursor_x, cursor_y, ch),
                }
            }
            Event::Key(key) => {
                match key {
                    ArrowUp | CtrlP => {
                        cursor_y -= 1;
                        f.move_cursor(cursor_x, cursor_y);
                    }
                    ArrowDown | CtrlN => {
                        cursor_y += 1;
                        f.move_cursor(cursor_x, cursor_y);
                    }
                    ArrowLeft | CtrlB => {
                        cursor_x -= 1;
                        f.move_cursor(cursor_x, cursor_y);
                    }
                    ArrowRight | CtrlF => {
                        cursor_x += 1;
                        f.move_cursor(cursor_x, cursor_y);
                    }
                    _ => {}
                }
            }
            Event::Resize { .. } => {}
        }
        f.flush().unwrap();
    }
}
