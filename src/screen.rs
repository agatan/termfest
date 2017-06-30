use std::iter::Iterator;
use std::collections::VecDeque;

use terminal::Command;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Cell {
    // `ch` is `None` if the left cell has wide character like 'あ'.
    pub ch: Option<char>,
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

    pub fn flush_commands(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut last_x = -1;
        let mut last_y = -1;
        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.index(x, y).unwrap();
                if self.painted_cells[index] == self.cells[index] {
                    continue;
                }
                if let Some(ch) = self.cells[index].ch {
                    if last_x != x || last_y != y {
                        commands.push(Command::MoveCursor { x: x, y: y });
                    }
                    commands.push(Command::PutChar(ch));
                    last_x = x;
                    last_y = y;
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
