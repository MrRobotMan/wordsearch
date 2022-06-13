use std::fs;

use rand::{distributions::Standard, prelude::Distribution, Rng};
use regex::Regex;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    AngledUpRight,
    AngledDownRight,
    AngledUpLeft,
    AngledDownLeft,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => write!(f, "UP"),
            Direction::Down => write!(f, "DOWN"),
            Direction::Left => write!(f, "RIGHT to LEFT"),
            Direction::Right => write!(f, "LEFT to RIGHT"),
            Direction::AngledUpRight => write!(f, "UP to the RIGHT"),
            Direction::AngledDownRight => write!(f, "DOWN to the RIGHT"),
            Direction::AngledUpLeft => write!(f, "UP to the LEFT"),
            Direction::AngledDownLeft => write!(f, "DOWN to the LEFT"),
        }
    }
}

pub struct Location {
    row: usize,
    column: usize,
}

impl Location {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.row, self.column)
    }
}

pub enum Color {
    Red,
    Green,
    Yellow,
    Magenta,
    Cyan,
    Reset,
    LightredEx,
    LightgreenEx,
    LightyellowEx,
    LightblueEx,
    LightmagentaEx,
    LightcyanEx,
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0..=10) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Yellow,
            3 => Color::Magenta,
            4 => Color::Cyan,
            5 => Color::LightredEx,
            6 => Color::LightgreenEx,
            7 => Color::LightyellowEx,
            8 => Color::LightblueEx,
            9 => Color::LightmagentaEx,
            _ => Color::LightcyanEx,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letter = match self {
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::Reset => 0,
            Color::LightredEx => 91,
            Color::LightgreenEx => 92,
            Color::LightyellowEx => 93,
            Color::LightblueEx => 94,
            Color::LightmagentaEx => 95,
            Color::LightcyanEx => 96,
        };
        write!(f, "\x1b[{letter}m")
    }
}

pub struct Grid {
    rows: Vec<Vec<char>>,
    columns: Vec<Vec<char>>,
    diag_up_right: Vec<Vec<char>>,
    diag_down_right: Vec<Vec<char>>,
    highlighted: Vec<Vec<String>>,
}

impl Grid {
    pub fn new(text: Vec<&str>) -> Self {
        let n_rows = text.len();
        let n_cols = text[0].len();
        let mut rows = vec![Vec::new(); n_rows];
        let mut columns = vec![Vec::new(); n_cols];
        let mut diag_up_right = vec![Vec::new(); n_rows + n_cols - 1];
        let mut diag_down_right = vec![Vec::new(); n_rows + n_cols - 1];
        let mut highlighted = vec![Vec::new(); n_rows];

        for (row, line) in text.iter().enumerate() {
            for (col, letter) in line.chars().enumerate() {
                rows[row].push(letter);
                columns[col].push(letter);
                diag_up_right[row + col].push(letter);
                diag_down_right[n_rows + col - row - 1].push(letter);
                highlighted[row].push(letter.to_string());
            }
        }
        for row in &mut diag_up_right {
            row.reverse();
        }

        Self {
            rows,
            columns,
            diag_up_right,
            diag_down_right,
            highlighted,
        }
    }
    pub fn from_str(text: &str) -> Self {
        let text = text.to_string().replace(" ", "");
        let text: Vec<&str> = text.split("\r\n").collect();
        Grid::new(text)
    }

    pub fn show_grid(&self) {
        for line in &self.rows {
            for letter in line {
                print!("{} ", letter)
            }
            print!("\n");
        }
    }

    pub fn show_solve(&self) {
        for line in &self.highlighted {
            for letter in line.iter() {
                print!("{} ", letter)
            }
            print!("\n");
        }
    }

    fn highlight(&mut self, start: &Location, dir: &Direction, len: usize, color: &Color) {
        let (row_off, col_off) = match dir {
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
            Direction::AngledUpRight => (-1, 1),
            Direction::AngledDownRight => (1, 1),
            Direction::AngledUpLeft => (-1, -1),
            Direction::AngledDownLeft => (1, -1),
        };
        for idx in 0..len {
            let row = (start.row as i32 + idx as i32 * row_off) as usize;
            let col = (start.column as i32 + idx as i32 * col_off) as usize;
            let letter = self.rows[row][col];
            self.highlighted[row][col] = format!("{}{}{}", color, letter, Color::Reset);
        }
    }
    pub fn find_word(&mut self, word: &str, color: &Color) -> Option<(Location, Direction)> {
        for (row, group) in self.rows.iter().enumerate() {
            if let Some((column, to_right)) = find_in_group(word, group) {
                let dir = if to_right {
                    Direction::Right
                } else {
                    Direction::Left
                };
                let start = Location { row, column };
                self.highlight(&start, &dir, word.len(), color);
                return Some((start, dir));
            }
        }
        for (column, group) in self.columns.iter().enumerate() {
            if let Some((row, is_down)) = find_in_group(word, group) {
                let dir = if is_down {
                    Direction::Down
                } else {
                    Direction::Up
                };
                let start = Location { row, column };
                self.highlight(&start, &dir, word.len(), color);
                return Some((start, dir));
            }
        }

        let num_rows = self.rows.len();
        for (diag, group) in self.diag_up_right.iter().enumerate() {
            if let Some((idx, is_forward)) = find_in_group(word, group) {
                let dir = if is_forward {
                    Direction::AngledUpRight
                } else {
                    Direction::AngledDownLeft
                };
                let row: usize;
                let column: usize;
                if diag < num_rows {
                    row = num_rows - diag;
                    column = idx;
                } else {
                    row = num_rows - idx - 1;
                    column = (diag - num_rows) + idx + 1;
                }
                let start = Location { row, column };
                self.highlight(&start, &dir, word.len(), color);
                return Some((start, dir));
            }
        }
        for (diag, group) in self.diag_down_right.iter().enumerate() {
            if let Some((idx, is_forward)) = find_in_group(word, group) {
                let dir = if is_forward {
                    Direction::AngledDownRight
                } else {
                    Direction::AngledUpLeft
                };
                let row: usize;
                let column: usize;
                if diag < num_rows {
                    row = num_rows - diag + idx - 1;
                    column = idx;
                } else {
                    row = idx;
                    column = diag - num_rows + idx + 1;
                }
                let start = Location { row, column };
                self.highlight(&start, &dir, word.len(), color);
                return Some((start, dir));
            }
        }

        None
    }
}

fn find_in_group(word: &str, group: &Vec<char>) -> Option<(usize, bool)> {
    let search_text: String = group.iter().collect();
    if let Some(pos) = search_text.find(word) {
        return Some((pos, true));
    };
    let reverse: String = group.iter().rev().collect();
    if let Some(pos) = reverse.find(word) {
        let last = reverse.len() - 1;
        return Some((last - pos, false));
    };
    None
}

pub fn get_words(text: &str) -> Vec<String> {
    let re = Regex::new(r"\s+").unwrap();
    let lines = re.split(&text);
    let mut res = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        res.push(line.to_string())
    }
    res
}

pub fn read_file(file: &str) -> (Grid, Vec<String>) {
    let text = fs::read_to_string(file).expect("Error reading the file");
    let text: Vec<&str> = text.split("\r\n\r\n\r\n").collect();
    (Grid::from_str(text[0]), get_words(text[1]))
}
