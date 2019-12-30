use std::fs::File;
use std::io::prelude::*;
use std::io::{self, empty, sink};

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

    let mut program_modified = program.clone();
    program_modified[1] = 12;
    program_modified[2] = 2;

    let mut computer = Intcode::new(program_modified);
    computer.compute(empty(), sink());

    println!("{}", computer.program[0]);

    for i in 0..99 {
        for j in 0..99 {
            let mut program = program.clone();
            program[1] = i;
            program[2] = j;
            let mut computer = Intcode::new(program);
            computer.compute(empty(), sink());

            if computer.program[0] == 19_690_720 {
                println!("{}", i * 100 + j);
                return Ok(());
            }
        }
    }

    Ok(())
}
