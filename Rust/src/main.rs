use finder;
use rand;
use std::{
    env,
    io::{stdin, stdout, Write},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let (mut grid, words) = finder::read_file(filename);
    grid.show_grid();
    println!("Press 'Enter' to reveal solution.");
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let reset = finder::Color::Reset;
    for word in words {
        let color: finder::Color = rand::random();
        if let Some(found) = grid.find_word(&word, &color) {
            let (loc, dir) = found;
            println!("Found {color}{word}{reset} at {loc} going {dir}.")
        } else {
            println!("Did not find {word}")
        }
    }
    grid.show_solve();
}
