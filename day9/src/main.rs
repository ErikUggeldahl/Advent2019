use std::fs::File;
use std::io::prelude::*;

use intcode::Intcode;

fn main() {
    let mut file = File::open("input.txt").expect("Could not open input file.");
    let mut program = String::new();
    file.read_to_string(&mut program).expect("Could not read file to string.");

    let program = program
        .trim()
        .split(',')
        .map(|i| i.parse::<i64>().unwrap())
        .collect::<Vec<_>>();

    let mut computer = Intcode::new(program.clone());
    let mut output = Vec::new();
    computer.compute(&b"1"[..], &mut output);
    let result = String::from_utf8(output).expect("Could not stringify output.");
    println!("{}", result);

    let mut computer = Intcode::new(program);
    let mut output = Vec::new();
    computer.compute(&b"2"[..], &mut output);
    let result = String::from_utf8(output).expect("Could not stringify output.");
    println!("{}", result);
}
