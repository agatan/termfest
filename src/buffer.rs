use std::io::{self, Write};

use term::terminfo::TermInfo;

use terminal::Terminal;

#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    // `ch` is `None` if the left cell has wide character like 'あ'.
    pub ch: Option<char>,
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub width: i32,
    pub height: i32,
    // length of `cells` is `width * height`
    // accessing (x, y) is equal to `cells[x + y * width]`
    pub cells: Vec<Cell>,
}

impl Screen {
    pub fn new(width: i32, height: i32) -> Self {
        Screen {
            width: width,
            height: height,
            cells: vec![Cell::default(); (width * height) as usize],
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || self.width <= x || y < 0 || self.height <= y {
            None
        } else {
            Some((x + y * self.width) as usize)
        }
    }

    pub fn put_cell(&mut self, x: i32, y: i32, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
        }
    }

    pub fn cell(&self, x: i32, y: i32) -> Option<&Cell> {
        self.index(x, y).map(|i| &self.cells[i])
    }

    pub fn cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Cell> {
        match self.index(x, y) {
            Some(i) => Some(&mut self.cells[i]),
            None => None,
        }
    }
}

#[derive(Debug, )]
pub struct Buffer {
    pub screen_buffer: Screen,
    pub terminal: Terminal,
    write_buffer: Vec<u8>,
}

impl Buffer {
    pub fn new(terminal: Terminal, width: i32, height: i32) -> Self {
        Buffer {
            screen_buffer: Screen::new(width, height),
            terminal: terminal,
            write_buffer: Vec::new(),
        }
    }

    fn flush_to_buffer(&mut self) -> io::Result<()> {
        let mut last_x = -1;
        let mut last_y = -1;
        for y in 0..self.screen_buffer.height {
            for x in 0..self.screen_buffer.width {
                let cell = match self.screen_buffer.cell(x, y) {
                    None => continue,
                    Some(cell) => cell,
                };
                if let Some(ch) = cell.ch {
                    if last_x != x - 1 || last_y != y {
                        self.terminal.move_cursor(&mut self.write_buffer, x, y)?;
                    }
                    self.terminal.put_char(&mut self.write_buffer, ch)?;
                    last_x = x;
                    last_y = y;
                }
            }
        }
        Ok(())
    }

    pub fn flush<W: Write>(&mut self, mut w: W) -> io::Result<()> {
        self.flush_to_buffer()?;
        w.write_all(&self.write_buffer)?;
        self.write_buffer.truncate(0);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.terminal.clear(&mut self.write_buffer).unwrap()
    }

    pub fn move_cursor(&mut self, x: i32, y: i32) {
        self.terminal
            .move_cursor(&mut self.write_buffer, x, y)
            .unwrap()
    }

    pub fn hide_cursor(&mut self) {
        self.terminal.hide_cursor(&mut self.write_buffer).unwrap()
    }

    pub fn show_cursor(&mut self) {
        self.terminal.show_cursor(&mut self.write_buffer).unwrap()
    }

    pub fn put_char(&mut self, x: i32, y: i32, ch: char) {
        if let Some(cell) = self.screen_buffer.cell_mut(x, y) {
            cell.ch = Some(ch);
        }
    }
}
