extern crate festival;

use festival::{Event, Key};

fn main() {
    let (mut f, rx) = festival::hold().unwrap();
    let (mut cursor_x, mut cursor_y) = (0, 0);

    loop {
        let ev = rx.recv().unwrap();
        match ev {
            Event::Key(Key::Char(ch)) => {
                match ch {
                    'q' => break,
                    ch => f.put_char(cursor_x, cursor_y, ch),
                }
            }
            Event::Key(Key::ArrowUp) => {
                cursor_y -= 1;
                f.move_cursor(cursor_x, cursor_y);
            }
            Event::Key(Key::ArrowDown) => {
                cursor_y += 1;
                f.move_cursor(cursor_x, cursor_y);
            }
            Event::Key(Key::ArrowLeft) => {
                cursor_x -= 1;
                f.move_cursor(cursor_x, cursor_y);
            }
            Event::Key(Key::ArrowRight) => {
                cursor_x += 1;
                f.move_cursor(cursor_x, cursor_y);
            }
            Event::Resize { .. } => {}
        }
        f.flush().unwrap();
    }
}
