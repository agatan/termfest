extern crate termfest;

use termfest::{Event, Termfest};
use termfest::key::*;
use termfest::attr::*;

fn main() {
    let (fest, rx) = Termfest::hold().unwrap();

    let mut screen = fest.lock_screen();
    let colors = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
    ];
    let mut i = 0;
    for c in colors.iter() {
        screen.print(
            0,
            i,
            "0123456789",
            Attribute {
                fg: *c,
                ..Attribute::default()
            },
        );
        i += 1;
        screen.print(
            0,
            i,
            "0123456789",
            Attribute {
                bg: *c,
                ..Attribute::default()
            },
        );
        i += 1;
    }

    let effects = [Effect::BOLD, Effect::DIM, Effect::UNDERLINE, Effect::BLINK, Effect::REVERSE];
    for e in effects.iter() {
        screen.print(
            0,
            i,
            "0123456789",
            Attribute {
                effect: *e,
                ..Attribute::default()
            },
        );
        i += 1;
    }

    screen.flush().unwrap();

    for ev in rx.iter() {
        if let Event::Key(ESC) = ev {
            break;
        }
    }
}
