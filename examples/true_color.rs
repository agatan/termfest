extern crate termfest;

use termfest::attr::*;
use termfest::{TermFest, Cell};

fn main() {
    let (fest, rx) = TermFest::hold().unwrap();

    let mut screen = fest.lock();
    screen.hide_cursor();
    let (w, h) = screen.size();
    for i in 0..h {
        for j in 0..w {
            let r = j * 255 / w;
            let g = if 2 * j < w {
                510 * j / w
            } else {
                510 - 510 * j / w
            };
            let b = 255 - j * 255 / w;
            screen.put_char(j,
                            i,
                            Cell::new(' ').bg(Color::Rgb(r as u8, g as u8, b as u8)));
        }
    }
    screen.flush().unwrap();

    rx.recv().unwrap();
}
