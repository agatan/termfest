extern crate termfest;

use termfest::{Event, TermFest,  Color};
use termfest::keys::*;

fn main() {
    let (fest, rx) = TermFest::hold().unwrap();

    let mut screen = fest.lock();
    screen.print(0, 0, "Foreground Black", Color::Black, Color::default());
    screen.print(0, 1, "Foreground Red", Color::Red, Color::default());
    screen.print(0, 2, "Foreground Green", Color::Green, Color::default());
    screen.print(0, 3, "Foreground Yellow", Color::Yellow, Color::default());
    screen.print(0, 4, "Foreground Blue", Color::Blue, Color::default());
    screen.print(0, 5, "Foreground Magenta", Color::Magenta, Color::default());
    screen.print(0, 6, "Foreground Cyan", Color::Cyan, Color::default());
    screen.print(0, 7, "Foreground White", Color::White, Color::default());
    screen.print(0, 8, "Background Black", Color::default(), Color::Black);
    screen.print(0, 9, "Background Red", Color::default(), Color::Red);
    screen.print(0, 10, "Background Green", Color::default(), Color::Green);
    screen.print(0, 11, "Background Yellow", Color::default(), Color::Yellow);
    screen.print(0, 12, "Background Blue", Color::default(), Color::Blue);
    screen.print(0, 13, "Background Magenta", Color::default(), Color::Magenta);
    screen.print(0, 14, "Background Cyan", Color::default(), Color::Cyan);
    screen.print(0, 15, "Background White", Color::default(), Color::White);
    screen.flush().unwrap();

    for ev in rx.iter() {
        if let Event::Key(ESC) = ev {
            break;
        }
    }
}
