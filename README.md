## termfest

[![Build Status](https://travis-ci.org/agatan/termfest.svg?branch=master)](https://travis-ci.org/agatan/termfest)

termfest is a thread-safe TUI library that provides simple APIs to render texts in terminal, heavily inspired by [nsf/termbox-go](https://github.com/nsf/termbox-go).
Currently, termfest doesn't support windows because of my poor windows experience.

termfest has internal buffer for efficient rendering.
Applications can render everything to the buffer every time, and termfest flushes the buffer and renders only the difference between the terminal state and the buffer.

```rust
use termfest::{Termfest, Event};
use termfest::attr::*;
use termfest::key::*;

// first, initialize termfest.
let (fest, events) = Termfest::hold().unwrap();

let mut y = 0;

// events is a receiver of a channel that accepts terminal events like key input.
for ev in events.iter() {
    {
        // lock the screen.
        let mut screen = fest.lock_screen();
        // clear the buffer. you can render everything every time.
        // termfest can provide efficient rendering.
        screen.clear();
        // write to the buffer.
        let attr = Attribute { fg: Color::Red, ..Attribute::default() };
        screen.print(0, y, "Hello, world!", attr);
        // when the screen lock is released, the buffer is flushed.
        // (you can flush the buffer with explicit `flush` call.)
    }
    match ev {
        Event::Key(ESC) | Event::Char('q') => break,
        Event::Key(ArrowUp) => if y > 0 { y -= 1; },
        Event::Key(ArrowDown) => y += 1,
        _ => {}
    }
}
```

See `examples` for more detail.
