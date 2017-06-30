extern crate festival;

use festival::{Event, Key};

fn main() {
    let (mut f, rx) = festival::hold().unwrap();
    f.clear();
    f.move_cursor(3, 3);
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.flush().unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.hide_cursor();
    f.flush().unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(1000));
    f.move_cursor(10, 10);
    f.show_cursor();
    f.flush().unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(1000));

    loop {
        match rx.recv().unwrap() {
            Event::Key(Key::Char(ch)) => {
                match ch {
                    'q' => break,
                    'f' => f.flush().unwrap(),
                    'h' => f.hide_cursor(),
                    's' => f.show_cursor(),
                    'u' => f.move_cursor(3, 3),
                    'd' => f.move_cursor(4, 4),
                    ch => f.put_char(5, 5, ch),
                }
            }
        }
    }
}
