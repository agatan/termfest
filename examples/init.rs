extern crate festival;

use std::io::{Write, stderr};

use festival::{Event, Key};

fn main() {
    let (mut f, rx) = festival::hold().unwrap();
    let (w, h) = f.size();

    loop {
        match rx.recv().unwrap() {
            Event::Key(Key::Char(ch)) => {
                match ch {
                    'q' => break,
                    'f' => f.flush().unwrap(),
                    'k' => f.move_cursor(w / 2, 0),
                    'j' => f.move_cursor(w / 2, h - 1),
                    'l' => f.move_cursor(w - 1, h / 2),
                    'h' => f.move_cursor(0, h / 2),
                    ch => f.put_char(5, 5, ch),
                }
            }
            Event::Resize { width, height } => {
                panic!(width);
                writeln!(stderr(), "w: {}, h: {}", width, height).unwrap();
            }
        }
    }
}
