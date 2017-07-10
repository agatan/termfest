extern crate termfest;

use termfest::attr::*;
use termfest::Termfest;

fn main() {
    let (fest, rx) = Termfest::hold().unwrap();

    let mut screen = fest.lock();
    screen.hide_cursor();
    for i in 0..16 {
        for j in 0..16 {
            let v = i * 16 + j;
            screen.print(j * 3,
                         i,
                         &format!("{:02x}", v),
                         Attribute {
                             fg: Color::EightBit(v as u8),
                             ..Attribute::default()
                         });

        }
    }
    screen.flush().unwrap();

    rx.recv().unwrap();
}
