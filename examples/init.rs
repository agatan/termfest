extern crate festival;

use festival::{Event, Key};

fn main() {
    let (mut f, rx) = festival::hold().unwrap();
    f.clear().unwrap();
    f.set_cursor(3, 3).unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.hide_cursor().unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));
    f.set_cursor(4, 4).unwrap();
    ::std::thread::sleep(::std::time::Duration::from_millis(500));

    loop {
        match rx.recv().unwrap() {
            Event::Key(Key::Char(ch)) => {
                if ch == 'q' {
                    break;
                } else {
                    f.set_cursor(4, 4).unwrap();
                    f.putchar(ch).unwrap();
                }
            }
        }
    }
}
