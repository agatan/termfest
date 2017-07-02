extern crate festival;

use festival::{Event, Key};

fn main() {
    let (mut f, rx) = festival::hold().unwrap();
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
                    Key::ArrowUp | Key::CtrlP => {
                        cursor_y -= 1;
                        f.move_cursor(cursor_x, cursor_y);
                    }
                    Key::ArrowDown | Key::CtrlN => {
                        cursor_y += 1;
                        f.move_cursor(cursor_x, cursor_y);
                    }
                    Key::ArrowLeft | Key::CtrlB => {
                        cursor_x -= 1;
                        f.move_cursor(cursor_x, cursor_y);
                    }
                    Key::ArrowRight | Key::CtrlF => {
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
