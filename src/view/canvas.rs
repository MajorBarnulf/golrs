use std::io::{stdout, Write};

use crate::{pos, Pos};

pub struct Canvas {
    lines: Vec<String>,
    width: usize,
    height: usize,
}

impl Canvas {
    pub fn from_screen() -> Self {
        let (width, height) = termion::terminal_size().unwrap();
        Self::new(width as usize, (height - 1) as usize)
    }

    pub fn new(width: usize, height: usize) -> Self {
        let lines = (0..height)
            .map(|_| (0..width).map(|_| ' '.to_string()).collect::<String>())
            .collect();
        Self {
            height,
            lines,
            width,
        }
    }

    pub fn layer(&mut self, f: impl Fn(Pos) -> Option<char>) {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(char) = f(pos!(x as i32, y as i32)) {
                    let line = &mut self.lines[y];
                    line.replace_range(
                        line.char_indices()
                            .nth(x)
                            .map(|(pos, ch)| (pos..pos + ch.len_utf8()))
                            .unwrap(),
                        &format!("{char}"),
                    );
                }
            }
        }
    }

    pub fn display(&self) {
        let clear = termion::clear::All;
        print!("{clear}");
        for (index, line) in self.lines.iter().enumerate() {
            let goto = termion::cursor::Goto(1, index as u16 + 1);
            println!("{goto}{line}");
        }
        stdout().flush().unwrap();
    }
}
