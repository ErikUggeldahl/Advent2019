use std::fs::File;
use std::io::prelude::*;
use std::io::{self};

use intcode::Intcode;

fn main() -> Result<(), io::Error> {
    let mut file = File::open("input.txt")?;
    let mut program = String::new();
    file.read_to_string(&mut program)?;

    let program = program
        .trim()
        .split(',')
        .map(|i| i.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    
    let mut computer = Intcode::new(program);

    let stdio = io::stdin();
    let input = stdio.lock();

    let output = io::stdout();

    computer.compute(input, output);

    Ok(())
}
