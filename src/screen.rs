use std::default::Default;

use unicode_width::UnicodeWidthChar;

use terminal::Command;
use attr::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub bg: Color,
    pub fg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: ' ',
            bg: Color::default(),
            fg: Color::default(),
        }
    }
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Cell {
            ch: ch,
            bg: Color::default(),
            fg: Color::default(),
        }
    }

    pub fn fg(self, attr: Color) -> Self {
        Cell { fg: attr, ..self }
    }

    pub fn bg(self, attr: Color) -> Self {
        Cell { bg: attr, ..self }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub x: i32,
    pub y: i32,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub width: i32,
    pub height: i32,
    // length of `cells` is `width * height`
    // accessing (x, y) is equal to `cells[x + y * width]`
    pub cells: Vec<Cell>,
    pub cursor: Cursor,

    painted_cells: Vec<Cell>,
    painted_cursor: Cursor,
}

impl Screen {
    pub fn new(width: i32, height: i32) -> Self {
        Screen {
            width: width,
            height: height,
            cells: vec![Cell::default(); (width * height) as usize],
            cursor: Cursor {
                x: 0,
                y: 0,
                visible: true,
            },

            painted_cells: vec![Cell::default(); (width * height) as usize],
            painted_cursor: Cursor {
                x: 0,
                y: 0,
                visible: true,
            },
        }
    }

    fn copy_cells(&self, original: &[Cell], width: i32, height: i32) -> Vec<Cell> {
        let mut new_cells = vec![Cell::default(); (width * height) as usize];
        use std::cmp;
        let min_height = cmp::min(height, self.height);
        let min_width = cmp::min(width, self.width);
        for y in 0..min_height {
            let orig_start = (y * self.width) as usize;
            let orig_end = min_width as usize + orig_start;
            let start = (y * width) as usize;
            let end = min_width as usize + start;
            (&mut new_cells[start..end]).copy_from_slice(&original[orig_start..orig_end]);
        }
        new_cells
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.cells = self.copy_cells(&self.cells, width, height);
        self.painted_cells = self.copy_cells(&self.painted_cells, width, height);
        self.width = width;
        self.height = height;
    }

    pub fn clear(&mut self) {
        for cell in self.painted_cells.iter_mut() {
            cell.ch = ' ';
        }
        for cell in self.cells.iter_mut() {
            cell.ch = ' ';
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || self.width <= x || y < 0 || self.height <= y {
            None
        } else {
            Some((x + y * self.width) as usize)
        }
    }

    pub fn print(&mut self, mut x: i32, y: i32, s: &str, fg: Color, bg: Color) {
        let mut cell = Cell::default().fg(fg).bg(bg);
        for c in s.chars() {
            cell.ch = c;
            self.put_char(x, y, cell);
            x += c.width().unwrap_or(1) as i32;
        }
    }

    pub fn put_char(&mut self, x: i32, y: i32, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = cell;
        }
    }

    pub fn flush_commands(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut last_x = -1;
        let mut last_y = -1;
        let mut last_fg = None;
        let mut last_bg = None;
        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.index(x, y).unwrap();
                if self.painted_cells[index] == self.cells[index] {
                    continue;
                }
                let cell = self.cells[index];
                if Some(cell.fg) != last_fg {
                    commands.push(Command::Fg(cell.fg));
                    last_fg = Some(cell.fg);
                }
                if Some(cell.bg) != last_bg {
                    commands.push(Command::Bg(cell.bg));
                    last_bg = Some(cell.bg);
                }
                let ch = cell.ch;
                if last_x != x || last_y != y {
                    commands.push(Command::MoveCursor { x: x, y: y });
                }
                commands.push(Command::PutChar(ch));
                last_x = x + 1;
                last_y = y;
                if ch.width() == Some(2) {
                    last_x += 1;
                    if let Some(right) = self.index(x + 1, y) {
                        self.cells[right] = Cell { ch: ' ', ..cell };
                    }
                }
                self.painted_cells[index] = self.cells[index];
            }
        }
        if self.cursor.visible && !self.painted_cursor.visible {
            commands.push(Command::ShowCursor);
        } else if !self.cursor.visible && self.painted_cursor.visible {
            commands.push(Command::HideCursor);
        }
        commands.push(Command::MoveCursor {
                          x: self.cursor.x,
                          y: self.cursor.y,
                      });
        self.painted_cursor = self.cursor;
        commands
    }
}
